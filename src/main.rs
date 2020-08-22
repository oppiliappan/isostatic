use futures_util::TryStreamExt;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use nanoid::nanoid;
use rusqlite::{params, Connection, OpenFlags, Result, NO_PARAMS};
use url::form_urlencoded;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::from_utf8;

fn shorten<S: AsRef<str>>(url: S, conn: &mut Connection) -> Result<String> {
    let mut stmt = conn.prepare("select * from urls where link = ?1")?;
    let mut rows = stmt.query(params![url.as_ref().to_string()])?;
    if let Some(row) = rows.next()? {
        return row.get(1);
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
    eprintln!("{}", url.as_ref());
    let url = url.as_ref();
    let mut stmt = conn.prepare("select * from urls where shortlink = ?1")?;
    let mut rows = stmt.query(params![url.to_string()])?;
    if let Some(row) = rows.next()? {
        return Ok(row.get(0)?);
    } else {
        return Ok(None);
    }
}

async fn shortner_service(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let mut conn = init_db("./urls.db_3").unwrap();

    match req.method() {
        &Method::POST => {
            let b = hyper::body::to_bytes(req).await?;
            let params = form_urlencoded::parse(b.as_ref())
                .into_owned()
                .collect::<HashMap<String, String>>();

            if let Some(n) = params.get("shorten") {
                let shortlink = shorten(n, &mut conn).unwrap();
                return Ok(Response::new(Body::from(shortlink)));
            } else {
                return Ok(Response::new(Body::from("invalid form")));
            };
        }
        &Method::GET => {
            let shortlink = req.uri().path().to_string();
            let link = get_link(&shortlink[1..], &mut conn);
            if let Some(l) = link.unwrap() {
                return Ok(Response::new(Body::from(l)));
            } else {
                return Ok(Response::new(Body::from("not found!")));
            }
        }
        _ => unimplemented!(),
    }
}

fn init_db<P: AsRef<Path>>(p: P) -> Result<Connection> {
    let conn = Connection::open_with_flags(
        p,
        OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_READ_WRITE,
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS urls (
            link TEXT PRIMARY KEY,
            shortlink TEXT NOT NULL
        )",
        params![],
    )?;
    return Ok(conn);
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = ([127, 0, 0, 1], 3000).into();

    let service =
        make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(shortner_service)) });

    let server = Server::bind(&addr).serve(service);

    println!("Listening on http://{}", addr);

    server.await.unwrap();
    Ok(())
}
