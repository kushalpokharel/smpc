use actix::{Actor, StreamHandler};
use actix_web_actors::ws;
use actix::ActorContext;
use kzen_paillier::*;
use shared::types::{FirstRoundResponse, InitializeProtocol, UnicastMessage, WebsocketMessage};


pub struct ClientActor;

impl Actor for ClientActor {
    type Context = ws::WebsocketContext<Self>;
}

impl ClientActor{
    pub fn _new() -> Self {
        ClientActor{}
    }

    pub fn send_unicast<T>(&self, from: usize, to: usize, data: T, ctx: &mut ws::WebsocketContext<Self>) where T: serde::Serialize {
        let msg = UnicastMessage::new(from, to, data);
        let json_str = serde_json::to_string(&msg).unwrap_or_else(|e| {
            eprintln!("Failed to serialize message: {}", e);
            "".to_string()
        });
        ctx.text(json_str);
    }

    pub fn _send_json<T>(&self, msg: &T, ctx: &mut ws::WebsocketContext<Self>) where T: serde::Serialize {
        let json_str = serde_json::to_string(msg).unwrap_or_else(|e| {
            eprintln!("Failed to serialize message: {}", e);
            "".to_string()
        });
        ctx.text(json_str);

    }

    pub fn start_protocol(&self, init: InitializeProtocol,  ctx: &mut ws::WebsocketContext<Self>) {
        // Start the protocol by sending an initialization message or any other setup
        let num_bits = init.bits_security;
        // generate the private and public Paillier keys here
        let kp = Paillier::keypair_with_modulus_size(num_bits).keys();
        let start_value =  10;
        let encrypted_value = Paillier::encrypt(&kp.0, start_value);
        serde_json::to_string(&encrypted_value).map(|msg| {
            let n_squared = kp.0.nn;
            let new_msg = FirstRoundResponse{
                computed_value: msg,
                n_squared
            };
            self.send_unicast(0, 1, new_msg, ctx);
        }).unwrap_or_else(|e| {
            eprintln!("Failed to serialize encrypted value: {}", e);
        });
    }
}

impl StreamHandler<Result<actix_http::ws::Message, ws::ProtocolError>> for ClientActor {
    fn handle(&mut self, msg: Result<actix_http::ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(actix_http::ws::Message::Text(text)) => {
                println!("Received text message: {}", text);
                let msg = match serde_json::from_str::<WebsocketMessage>(&text){
                    Ok(message) => message,
                    Err(e) => {
                        println!("Failed to parse message: {}", e);
                        // this is basically not being able to parse the result sent by one of the clients. which is fatal
                        // should be handled more gracefully.
                        return;
                    }
                };
                match msg {
                    WebsocketMessage::InitializeProtocol(init) => {
                        self.start_protocol(init, ctx);
                    }
                    _ => {
                        println!("Received unsupported WebsocketMessage variant");
                    }

                }
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

