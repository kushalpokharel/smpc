use std::collections::HashMap;
use crate::actor::server_message::{InitializeParameters, RegisterClient};

use actix::prelude::*;
use futures::future::try_join_all;
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


pub struct ServerActor{
    // maps client from a sequence number to their URLs.
    clients: HashMap<u32, String>,
    total_clients: u32,
    state: State,
    key_pair: Option<kzen_paillier::Keypair>
}

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
        if self.total_clients > 1 {
            let kp = Paillier::keypair_with_modulus_size(2048);
            self.key_pair = Some(kp);
            let clients: Vec<(u32, String)> = self.clients.iter().map(|(&seq, url)| (seq, url.clone())).collect();
            ctx.spawn(
                actix::fut::wrap_future(async move {
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
                })
                .map(|result, act:&mut ServerActor, _ctx| {
                    match result {
                        Ok(_websockets) => {
                            act.state = State::FirstRound;
                            println!("Transitioned to FirstRound state.");
                        }
                        Err(_) => {
                            eprintln!("Error connecting to clients: ");
                        }
                    }
                })
            );
        } else {
            eprintln!("Not enough clients registered to start the protocol. At least 2 clients needed");
        }
    }
}

