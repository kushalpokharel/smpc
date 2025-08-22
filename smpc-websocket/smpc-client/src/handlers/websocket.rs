use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use crate::actor::client_actor::ClientActor;
pub async fn websocket(req:HttpRequest, stream:web::Payload)-> Result<HttpResponse, actix_web::Error> {
    // This function will handle the websocket connection
    println!("WebSocket connection established");
    Ok(ws::start(ClientActor::new(), &req, stream)?)
}