// extern
use anyhow::{Context, Result};
use hyper::header::{HeaderValue, CONTENT_TYPE};
use hyper::{Body, Method, Request, Response, StatusCode};
use log::{error, info, trace};
use multer::Multipart;
use nanoid::nanoid;
use rusqlite::{params, Connection};
use url::form_urlencoded;

// std
use std::collections::HashMap;

fn get_host(req: &Request<Body>) -> Option<HeaderValue> {
    let host = req.headers().get("host").map(|h| h.clone())?;
    return Some(host);
}

fn welcome(req: Request<Body>) -> Response<Body> {
    let _h = get_host(&req);
    let host = _h.as_ref().map(|h| h.as_bytes()).unwrap_or(b"");
    let text = format!(
        "
This URL shortening service is powered by hedge.

    github.com/nerdypepper/hedge

To shorten urls:

    curl -F'shorten=https://shorten.some/long/url' {}\n",
        String::from_utf8_lossy(host)
    );
    return Response::builder()
        .header("content-type", "text/plain")
        .body(Body::from(text))
        .unwrap();
}

fn respond_with_shortlink<S: AsRef<[u8]>>(shortlink: S, host: &[u8]) -> Response<Body> {
    let url = [b"https://", host, b"/", shortlink.as_ref(), b"\n"].concat();
    info!("Successfully generated shortlink");
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/html")
        .body(Body::from(url))
        .unwrap()
}

fn respond_with_status(s: StatusCode) -> Response<Body> {
    Response::builder().status(s).body(Body::empty()).unwrap()
}

fn shorten<S: AsRef<str>>(url: S, conn: &mut Connection) -> Result<String> {
    let mut stmt = conn.prepare("select * from urls where link = ?1")?;
    let mut rows = stmt.query(params![url.as_ref().to_string()])?;
    if let Some(row) = rows.next()? {
        return Ok(row.get(1)?);
    } else {
        let new_id = nanoid!(4);
        conn.execute(
            "insert into urls (link, shortlink) values (?1, ?2)",
            params![url.as_ref().to_string(), new_id],
        )?;
        return Ok(new_id);
    }
}

fn get_link<S: AsRef<str>>(url: S, conn: &Connection) -> Result<Option<String>> {
    let url = url.as_ref();
    let mut stmt = conn.prepare("select * from urls where shortlink = ?1")?;
    let mut rows = stmt.query(params![url.to_string()])?;
    if let Some(row) = rows.next()? {
        return Ok(row.get(0)?);
    } else {
        return Ok(None);
    }
}

async fn process_multipart(
    req: Request<Body>,
    boundary: String,
    conn: &mut Connection,
) -> Result<Response<Body>> {
    let _h = get_host(&req);
    let host = _h.as_ref().map(|h| h.as_bytes()).unwrap_or(b"");
    let mut m = Multipart::new(req.into_body(), boundary);
    if let Some(field) = m.next_field().await? {
        if field.name() == Some("shorten") {
            trace!("Recieved valid multipart request");
            let content = field
                .text()
                .await
                .with_context(|| format!("Expected field name"))?;

            let shortlink = shorten(content, conn)?;
            return Ok(respond_with_shortlink(shortlink, host));
        }
    }
    trace!("Unprocessable multipart request!");
    Ok(respond_with_status(StatusCode::UNPROCESSABLE_ENTITY))
}

async fn process_form(req: Request<Body>, conn: &mut Connection) -> Result<Response<Body>> {
    let _h = get_host(&req);
    let host = _h.as_ref().map(|h| h.as_bytes()).unwrap_or(b"");
    let b = hyper::body::to_bytes(req)
        .await
        .with_context(|| format!("Failed to stream request body!"))?;

    let params = form_urlencoded::parse(b.as_ref())
        .into_owned()
        .collect::<HashMap<String, String>>();

    if let Some(n) = params.get("shorten") {
        trace!("POST: {}", &n);
        let s = shorten(n, conn)?;
        return Ok(respond_with_shortlink(s, host));
    } else {
        error!("Invalid form");
        return Ok(respond_with_status(StatusCode::UNPROCESSABLE_ENTITY));
    }
}

pub async fn shortner_service(req: Request<Body>, mut conn: Connection) -> Result<Response<Body>> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/") => {
            let boundary = req
                .headers()
                .get(CONTENT_TYPE)
                .and_then(|ct| ct.to_str().ok())
                .and_then(|ct| multer::parse_boundary(ct).ok());

            if boundary.is_none() {
                return process_form(req, &mut conn).await;
            }

            trace!("Attempting to parse multipart request");
            return process_multipart(req, boundary.unwrap(), &mut conn).await;
        }
        (&Method::GET, "/") => Ok(welcome(req)),
        (&Method::GET, _) => {
            trace!("GET: {}", req.uri());
            let shortlink = req.uri().path().to_string();
            let link = get_link(&shortlink[1..], &conn);
            if let Some(l) = link.unwrap() {
                trace!("Found in database, redirecting ...");
                Ok(Response::builder()
                    .header("Location", &l)
                    .header("content-type", "text/html")
                    .status(StatusCode::MOVED_PERMANENTLY)
                    .body(Body::from(format!(
                        "You will be redirected to: {}. If not, click the link.",
                        &l
                    )))?)
            } else {
                error!("Resource not found");
                Ok(respond_with_status(StatusCode::NOT_FOUND))
            }
        }
        _ => Ok(respond_with_status(StatusCode::NOT_FOUND)),
    }
}
