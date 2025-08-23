use std::arch::aarch64::uint32x4_t;
use std::collections::HashMap;
use crate::actor::server_message::{InitializeParameters, RegisterClient};
use crate::errors::error_close::ErrorClose;

use actix_codec::Framed;
use actix_web::web::Bytes;
use futures::io::Sink;
use futures::stream::{select_all, SplitSink};
use serde::Serialize;
use serde_json::value::Index;
use serde_json::Value;
use actix::io::{SinkWrite, WriteHandler};
use actix::prelude::*;
use awc::error::WsProtocolError;
use awc::ws::{CloseCode, CloseReason, Codec, Frame, Message};
use futures::future::try_join_all;
use futures::StreamExt;
use kzen_paillier::{KeyGeneration, Paillier};
use awc::{BoxedSocket, Client};
use crate::errors::websocket_error::WebsocketError;
use crate::errors::server_error::ServerError;
use shared::types::{ClientMessage, InitializeProtocol, WebsocketMessage};

#[derive(PartialEq)]
enum State{
    ClientConnection, 
    FirstRound, 
    SecondRound
}


pub struct ServerActor{
    // maps client from a sequence number to their URLs.
    clients: HashMap<u32, String>,
    total_clients: u32,
    state: State,
    key_pair: Option<kzen_paillier::Keypair>,
    sinks: Option<Vec<SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>>>
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
            key_pair: None,
            sinks:None
        }
    }

    pub fn handle_websocket_message(&mut self, msg: WebsocketMessage, client_index: usize, ctx: &mut <Self as Actor>::Context) {
        
        match msg {
            WebsocketMessage::Unicast(response) => {
                let wmsg = response.get_value();
                // Handle the first round response
                println!("Received Unicast from client {}: {:?}", client_index, response);
                self.send_json(&wmsg, response.to, ctx);
            }
            WebsocketMessage::Broadcast(response) => {
                // Handle the first round response
                println!("Received SecondRound from client {}: {:?}", client_index, response);
                // send the response to all clients except the one broadcasting it.
            }
            _ => {
                eprintln!("Received unexpected message type: {:?}", msg);
            }
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
    fn schedule_send(self:&mut ServerActor, _ctx: &mut <ServerActor as Actor>::Context, times:u64) {
        if times > 5 {
            eprintln!("Failed to send InitializeParameters message after 5 retries. Giving up.");
            return;
        }
        let _ = _ctx.address().try_send(InitializeParameters)
        .map_err(|e| {
            eprintln!("Failed to send InitializeParameters message Retrying in 10 seconds: {}", e);
            self.schedule_send( _ctx, times+1);
        });
    }

    #[inline]
    fn write_raw(&mut self, client_index: usize, message: Message) {
        eprintln!("Sending message to client {}: {:?} ", client_index, message);

        // Get the sink, trapping any out of bound errors (Should NOT happen)
        if let Some(ref mut sinks) = self.sinks{
            let sink = sinks.get_mut(client_index);
            if sink.is_none() {
                return eprintln!(
                "Invalid client {} (Num client = {})",
                client_index + 1,
                self.total_clients
                );
            }
            let sink = sink.unwrap();
            if let Err(_) = sink.write(message) {
                return eprintln!("Error writing message: Sink is closed or closing");
            }
        }
        
    }

    /// Send a JSON response back to a given websocket, handling any serialization errors
    ///
    /// Returns "false" if this failed due to an error, meaning the actor should stop any processing immediately
    fn send_json<T>(&mut self, data: &T, client_index: usize, ctx: &mut <Self as Actor>::Context) -> bool
    where
        T: ?Sized + Serialize,
    {
        let serialized = serde_json::to_string(data);
        match serialized {
            Ok(ref json) => self.text(client_index, json),
            Err(ref e) => self.error_close((CloseCode::Error, format!("{}", e)), ctx),
        }

        serialized.is_ok()
    }

    /// Send text frame
    #[inline]
    fn text<T: Into<String>>(&mut self, client_index: usize, text: T) {
        self.write_raw(client_index, Message::Text(text.into().into()));
    }

    /// Send binary frame
    #[inline]
    fn binary<B: Into<Bytes>>(&mut self, client_index: usize, data: B) {
        self.write_raw(client_index, Message::Binary(data.into()));
    }

    /// Send ping frame
    #[inline]
    fn ping(&mut self, client_index: usize, message: &[u8]) {
        self.write_raw(client_index, Message::Ping(Bytes::copy_from_slice(message)));
    }

    /// Send pong frame
    #[inline]
    fn pong(&mut self, client_index: usize, message: &[u8]) {
        self.write_raw(client_index, Message::Pong(Bytes::copy_from_slice(message)));
    }

    /// Send close frame
    #[inline]
    fn close(&mut self, client_index: usize, reason: Option<CloseReason>) {
        self.write_raw(client_index, Message::Close(reason));
    }

    /// Close the mediator actor due to a fatal error
    fn error_close(&mut self, error: impl Into<ErrorClose>, ctx: &mut <Self as Actor>::Context) {
        let ErrorClose(code, description) = error.into();

        if let Some(ref description) = description {
            eprintln!("Closing mediator: {} (Code {:#?})", description, code);
        } else {
            eprintln!("Closing mediator: code {:#?}", code);
        }

        // Send the close data to all websockets and stop the actor
        self.close_all_websockets(&Some(CloseReason { code, description }));
        ctx.stop();
    }

    fn close_all_websockets(&mut self, close_reason: &Option<CloseReason>) {
        for index in 0..self.total_clients as usize {
            self.close(index, close_reason.clone());
        }
    }

}



impl Handler<RegisterClient> for ServerActor {
    type Result = ();



    fn handle(&mut self, msg: RegisterClient, ctx: &mut Self::Context) {
        if self.state != State::ClientConnection {
            eprintln!("Cannot register client, server is not in ClientConnection state.");
            return;
        }
        self.clients.insert(self.total_clients, msg.url.clone());
        println!("Registered client {} with URL: {}", self.total_clients, msg.url);
        self.total_clients += 1;
        
        if self.total_clients == 1 && self.state == State::ClientConnection {
            println!("Here i am");
            //schedule a task to send a message to the first client after 120 seconds
            ctx.run_later(std::time::Duration::from_secs(10), |act, ctx|{
                println!("Sending InitializeParameters message to start the protocol.");
                act.schedule_send(ctx, 0);
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
                            let (streams, sinks):(Vec<_>, Vec<_>) = websockets
                                .into_iter()
                                .map(|(id, connection)| {
                                    let (_sink, source) = connection.split();
                                    (source.map(move |item| (id, item)), SinkWrite::new(_sink, ctx))
                                })
                                .unzip();

                            // unify all the streams/sources into single stream where any message from any websocket client 
                            // contains client's sequence id and the message(frame).
                            ctx.add_stream(select_all(streams));
                            act.state = State::FirstRound;
                            act.sinks = Some(sinks);
                            println!("Transitioned to FirstRound state.");

                            let client_params: ClientMessage = ClientMessage::InitializeProtocol((InitializeProtocol{
                                bits_security: 2048,
                                num_parties: act.total_clients as usize,
                                sid: 0

                            }));

                            act.send_json(&client_params, 0, ctx);
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
                        let msg = match serde_json::from_slice::<WebsocketMessage>(&text){
                            Ok(message) => message,
                            Err(e) => {
                                println!("Failed to parse message: {}", e);
                                return;
                            }
                        };
                        self.handle_websocket_message(msg, id as usize, ctx);
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
