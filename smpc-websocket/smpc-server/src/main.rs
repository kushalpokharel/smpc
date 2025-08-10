mod actor;
mod handlers;

use actor::server_actor::ServerActor;
use actix::prelude::*;
use actix_web::{web, App, HttpServer};
use anyhow;

#[actix_web::main]
async fn main() -> anyhow::Result<()>{
    // make a server actor here that will be global. Pass the address as the webdata in the server.
    let server_addr = ServerActor::new().start();
    // define an endpoint to which different clients can connect
    let server= HttpServer::new(move || {
        App::new()
        .app_data(web::Data::new(server_addr.clone()))
        
        .route("/", web::get().to(handlers::connect_websocket::connect_websocket))
    });
    server.bind("127.0.0.1:8080")?
        .run()
        .await?;
    Ok(())
}