// start the web server and handle websocket connections
mod handlers;
mod actor;

use actix_web::{web, App, HttpServer};
use handlers::connect_server::connect_to_server;
use anyhow;
use handlers::websocket::websocket;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let client_server = HttpServer::new(move || {
        App::new()
            .route("/", web::get().to(connect_to_server))
            .route("/connect", web::get().to(websocket))
    });
    client_server.bind("localhost:8082")?
        .run()
        .await?;
    Ok(())
}