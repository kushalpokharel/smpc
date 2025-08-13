use std::arch::aarch64::uint32x4_t;
use std::collections::HashMap;
use crate::actor::server_message::{BroadcastMessage, InitializeParameters, RegisterClient, UnicastMessage, WebsocketInitResult};

use futures::stream::select_all;
use serde_json::Value;
use actix::io::{SinkWrite, WriteHandler};
use actix::prelude::*;
use awc::error::WsProtocolError;
use awc::ws::Frame;
use futures::future::try_join_all;
use futures::StreamExt;
use kzen_paillier::{KeyGeneration, Paillier};
use awc::{Client};
use crate::errors::websocket_error::WebsocketError;
use crate::errors::server_error::ServerError;

#[derive(PartialEq)]
enum State{
    ClientConnection, 
    FirstRound, 
    SecondRound
}

enum WebsocketMessages{
    UnicastMessage(UnicastMessage<Value>),
    BroadcastMessage(BroadcastMessage<Value>)
}


pub struct ServerActor{
    // maps client from a sequence number to their URLs.
    clients: HashMap<u32, String>,
    total_clients: u32,
    state: State,
    key_pair: Option<kzen_paillier::Keypair>,
}

// Message to handle websocket initialization result


impl Actor for ServerActor{
    type Context = Context<Self>;
}

impl ServerActor{
    pub fn new()->Self{
        ServerActor{
            clients: HashMap::new(),
            total_clients: 0,
            state: State::ClientConnection,
            key_pair: None
        }
    }

    // fn error_close(&mut self, error: impl Into<ErrorClose>, ctx: &mut <Self as Actor>::Context) {
    //     let ErrorClose(code, description) = error.into();

    //     if let Some(ref description) = description {
    //         eprintln!("Closing mediator: {} (Code {:#?})", description, code);
    //     } else {
    //         eprintln!("Closing mediator: code {:#?}", code);
    //     }

    //     // Send the close data to all websockets and stop the actor
    //     self.close_all_websockets(&Some(CloseReason { code, description }));
    //     ctx.stop();
    // }
}

fn schedule_send(f:&mut ServerActor, _ctx: &mut <ServerActor as Actor>::Context, times:u64) {
    if times > 5 {
        eprintln!("Failed to send InitializeParameters message after 5 retries. Giving up.");
        return;
    }
    let _ = _ctx.address().try_send(InitializeParameters)
    .map_err(|e| {
        eprintln!("Failed to send InitializeParameters message Retrying in 10 seconds: {}", e);
        schedule_send(f, _ctx, times+1);
    });
}

impl Handler<RegisterClient> for ServerActor {
    type Result = ();



    fn handle(&mut self, msg: RegisterClient, ctx: &mut Self::Context) {
        if self.state != State::ClientConnection {
            eprintln!("Cannot register client, server is not in ClientConnection state.");
            return;
        }
        self.total_clients += 1;
        self.clients.insert(self.total_clients, msg.url.clone());
        println!("Registered client {} with URL: {}", self.total_clients, msg.url);
        
        if self.total_clients >= 1 && self.state == State::ClientConnection {
            //schedule a task to send a message to the first client after 120 seconds
            ctx.run_later(std::time::Duration::from_secs(10), |act, ctx|{
                println!("Sending InitializeParameters message to start the protocol.");
                schedule_send(act, ctx, 0);
            });
            
        }
    }
}
 

impl Handler<InitializeParameters> for ServerActor {
    type Result = ();

    fn handle(&mut self, _msg: InitializeParameters, ctx: &mut Self::Context) {
        if self.state != State::ClientConnection {
            eprintln!("Cannot initialize parameters, server is not in ClientConnection state.");
            return;
        }
        // call the function to separate the source with id from each client and sink will be combined.

        if self.total_clients > 1 {
            let kp = Paillier::keypair_with_modulus_size(2048);
            self.key_pair = Some(kp);
            let clients: Vec<(u32, String)> = self.clients.iter().map(|(&seq, url)| (seq, url.clone())).collect();
            // Spawn a future to connect to all clients, then send a message to self with the result
            let myself = ctx.address();
            // let arb = Arbiter::new();
            ctx.spawn(actix::fut::wrap_future(
                async move {
                    let websockets = try_join_all(clients.into_iter().map(|(seq, url)| {
                        async move {
                            println!("Connecting to client {} at URL: {}", seq, url);
                            let request = Client::builder().finish().ws(&url);
                            let connection_stream = WebsocketError::connect(request)
                                .await
                                .map_err(|e| ServerError::InitializationError(e))?;
                            Result::<_, ServerError>::Ok((seq, connection_stream))
                        }
                        
                    })).await;
                    websockets
                }).map(|websockets, act:&mut ServerActor, ctx: &mut <ServerActor as Actor>::Context| {
                    match websockets {
                        Ok(websockets) => {
                            println!("Successfully connected to all clients.");
                            // split the websocket channel into two streams, source and sink.
                            let streams: Vec<_> = websockets
                                .into_iter()
                                .map(|(id, stream)| {
                                    let (_sink, source) = stream.split();
                                    source.map(move |item| (id, item))
                                })
                                .collect();

                            // unify all the streams/sources into single stream where any message from any websocket client 
                            // contains client's sequence id and the message(frame).
                            ctx.add_stream(select_all(streams));
                            act.state = State::FirstRound;
                            println!("Transitioned to FirstRound state.");

                        }
                        Err(e) => {
                            eprintln!("Error connecting to clients");
                            return;
                        }
                    }
                })
            );
        } else {
            eprintln!("Not enough clients registered to start the protocol. At least 2 clients needed");
        }
    }
}

impl StreamHandler<(u32, Result<Frame, WsProtocolError>)> for ServerActor{
    fn handle(&mut self, item: (u32, Result<Frame, WsProtocolError>), ctx: &mut Self::Context) {
        match item {
            (id, Ok(frame)) => {
                println!("Received frame from client {}: {:?}", id, frame);
                // Handle the frame as needed
                match frame {
                    Frame::Text(text) => {
                        println!("Text frame from client {}: {}", id, String::from_utf8_lossy(&text));
                        // Process text frame

                    }
                    Frame::Binary(data) => {
                        println!("Binary frame from client {}: {:?}", id, data);
                        // Process binary frame
                    }
                    Frame::Close(_) => {
                        println!("Client {} has closed the connection.", id);
                        // Handle client disconnection if needed
                    }
                    _ => {
                    eprintln!("Unexpected frame type from client {}: {:?}", id, frame);
                    }
                }
            
            }
            (id, Err(e)) => {
                eprintln!("Error receiving frame from client {}", id, );
                // Handle the error as needed
            }
        }
    }
}



impl WriteHandler<WsProtocolError> for ServerActor {
  fn error(&mut self, error: WsProtocolError, ctx: &mut Self::Context) -> Running {
    // Send a message to close the actor due to a websocket error
    // self.error_close(
    //   (CloseCode::Error, format!("Error writing websocket message: {}", error)),
    //   ctx,
    // );

    Running::Stop
  }
}
