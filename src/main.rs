// extern
use anyhow::Result;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use log::trace;

// internal
mod db;
use db::{init_db, open_connection};

mod service;
use service::shortner_service;

fn main() -> Result<()> {
    pretty_env_logger::init();
    init_db("./urls.db_3")?;
    smol::run(async {
        let addr = ([127, 0, 0, 1], 3000).into();
        let service = make_service_fn(move |_| async {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let db_conn = open_connection("./urls.db_3").unwrap();
                shortner_service(req, db_conn)
            }))
        });
        let server = Server::bind(&addr).serve(service);
        trace!("Listening on http://{}", addr);
        server.await.unwrap();
        Ok(())
    })
}
