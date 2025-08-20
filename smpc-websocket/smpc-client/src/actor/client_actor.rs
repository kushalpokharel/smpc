use actix::{Actor, StreamHandler};
use actix_web_actors::ws;
use actix::ActorContext;

use crate::actor::client_message::WebsocketMessage;

pub struct ClientActor;

impl Actor for ClientActor {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<actix_http::ws::Message, ws::ProtocolError>> for ClientActor {
    fn handle(&mut self, msg: Result<actix_http::ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(actix_http::ws::Message::Text(text)) => {
                println!("Received text message: {}", text);
                let str = match serde_json::from_str::<WebsocketMessage>(&text){
                    Ok(message) => message,
                    Err(e) => {
                        println!("Failed to parse message: {}", e);
                        // this is basically not being able to parse the result sent by one of the clients. which is fatal
                        // should be handled more gracefully.
                        return;
                    }
                };
            }
            Ok(actix_http::ws::Message::Binary(bin)) => {
                println!("Received binary message");
                ctx.binary(bin);
            }
            Ok(actix_http::ws::Message::Close(reason)) => {
                println!("WebSocket closed: {:?}", reason);
                ctx.close(reason);
            }
            Ok(_) => {
                println!("Received unsupported message type");
            }
            Err(e) => {
                println!("WebSocket error: {:?}", e);
                ctx.stop();
            }
        }
    }
}

