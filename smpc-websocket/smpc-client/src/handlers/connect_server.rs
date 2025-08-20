use std::str::FromStr;

use actix_web::{web, HttpRequest, HttpResponse};
use awc::{ClientBuilder};
use actix_http::header::{HeaderName, HeaderValue};
use actix_http::Method;



pub async fn connect_to_server(
    _req:HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
        // Client::new()
        //     .connect("http://localhost:8080")
        //     .header("Origin", "http://localhost:8081")
        //     .await
        //     .map_err(|e| {
        //         eprintln!("Failed to connect to server: {}", e);
        //         actix_web::error::ErrorInternalServerError("Failed to connect to server")
        //     })?;
        let header_name = HeaderName::from_str("Origin");
        let header_value = HeaderValue::from_str("ws://localhost:8082");

       let _resp = ClientBuilder::new()
            .add_default_header((header_name.unwrap(), header_value.unwrap()))
            .finish()
            .request(Method::GET, "http://localhost:8080")
            .send()
            .await
            .map_err(|e| {
                eprintln!("Failed to connect to server: {}", e);
                actix_web::error::ErrorInternalServerError("Failed to connect to server")
            })?;
        Ok(HttpResponse::Ok().body("Connected to server!"))
        
}