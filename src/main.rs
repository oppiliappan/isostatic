use anyhow::Result;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;

mod db;
mod service;
use service::shortner_service;

fn main() -> Result<()> {
    smol::run(async {
        let addr = ([127, 0, 0, 1], 3000).into();
        let service =
            make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(shortner_service)) });
        let server = Server::bind(&addr).serve(service);
        println!("Listening on http://{}", addr);
        server.await.unwrap();
        Ok(())
    })
}
