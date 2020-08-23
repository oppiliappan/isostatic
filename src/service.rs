use anyhow::{Context, Result};
use hyper::header::CONTENT_TYPE;
use hyper::{Body, Method, Request, Response, StatusCode};
use log::{debug, error, info, trace};
use multer::Multipart;
use nanoid::nanoid;
use rusqlite::{params, Connection};
use url::form_urlencoded;

use std::collections::HashMap;

use crate::db::init_db;

fn respond_with_shortlink<S: AsRef<str>>(shortlink: S) -> Response<Body> {
    info!("Successfully generated shortlink");
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/html")
        .body(Body::from(shortlink.as_ref().to_string()))
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

fn get_link<S: AsRef<str>>(url: S, conn: &mut Connection) -> Result<Option<String>> {
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
    body: Body,
    boundary: String,
    conn: &mut Connection,
) -> Result<Response<Body>> {
    let mut m = Multipart::new(body, boundary);
    if let Some(field) = m.next_field().await? {
        if field.name() == Some("shorten") {
            trace!("Recieved valid multipart request");
            let content = field
                .text()
                .await
                .with_context(|| format!("Expected field name"))?;

            let shortlink = shorten(content, conn)?;
            return Ok(respond_with_shortlink(shortlink));
        }
    }
    trace!("Unprocessable multipart request!");
    Ok(respond_with_status(StatusCode::UNPROCESSABLE_ENTITY))
}

pub async fn shortner_service(req: Request<Body>) -> Result<Response<Body>> {
    let mut conn = init_db("./urls.db_3").unwrap();

    match req.method() {
        &Method::POST => {
            let boundary = req
                .headers()
                .get(CONTENT_TYPE)
                .and_then(|ct| ct.to_str().ok())
                .and_then(|ct| multer::parse_boundary(ct).ok());

            if boundary.is_none() {
                let b = hyper::body::to_bytes(req)
                    .await
                    .with_context(|| format!("Failed to stream request body!"))?;

                let params = form_urlencoded::parse(b.as_ref())
                    .into_owned()
                    .collect::<HashMap<String, String>>();

                if let Some(n) = params.get("shorten") {
                    trace!("POST: {}", &n);
                    let s = shorten(n, &mut conn)?;
                    return Ok(respond_with_shortlink(s));
                } else {
                    error!("Invalid form");
                    return Ok(respond_with_status(StatusCode::UNPROCESSABLE_ENTITY));
                }
            }

            trace!("Attempting to parse multipart request");
            return process_multipart(req.into_body(), boundary.unwrap(), &mut conn).await;
        }
        &Method::GET => {
            trace!("GET: {}", req.uri());
            let shortlink = req.uri().path().to_string();
            let link = get_link(&shortlink[1..], &mut conn);
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
