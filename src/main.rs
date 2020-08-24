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

mod cli;
use cli::{CONFIG, HELP_TEXT};

fn main() -> Result<()> {
    pretty_env_logger::init();

    if CONFIG.help {
        println!("{}", HELP_TEXT);
        return Ok(());
    }

    init_db(&CONFIG.db_path)?;
    smol::run(async {
        let addr = ([127, 0, 0, 1], CONFIG.port).into();
        let service = make_service_fn(|_| async {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let db_conn = open_connection(&CONFIG.db_path).unwrap();
                shortner_service(req, db_conn)
            }))
        });
        let server = Server::bind(&addr).serve(service);
        trace!("Listening on http://{}", addr);
        server.await.unwrap();
        Ok(())
    })
}
