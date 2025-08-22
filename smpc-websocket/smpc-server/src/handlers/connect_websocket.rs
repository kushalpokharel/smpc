use actix_web::{web, HttpRequest, HttpResponse};

use crate::actor::{server_actor::ServerActor, server_message::RegisterClient};

pub async fn connect_websocket(
    req:HttpRequest,
    data: web::Data<actix::Addr<ServerActor>>,
) -> Result<HttpResponse, actix_web::Error> {
    let addr = data.get_ref();
    let url = req.headers().get("Origin").and_then(|v| v.to_str().ok()).ok_or(actix_web::error::ErrorBadRequest("Port not found in request"))?;
    
    println!("{}", url);
    addr.try_send(RegisterClient{
        url: url.to_string()+ "/connect",
    }).map_err(|e| {
        eprintln!("Failed to register client: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to register client")
    })?;
    Ok(HttpResponse::Ok()
        .json(format!("Connected to server at {}", url))
    )
}