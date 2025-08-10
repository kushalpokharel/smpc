use actix_web::{HttpResponse};

pub async fn connect_websocket() -> HttpResponse {
    HttpResponse::Ok().body("WebSocket endpoint")
}