use std::collections::HashMap;
use crate::actor::server_message::RegisterClient;

use actix::prelude::*;


pub struct ServerActor{
    // maps client from a sequence number to their URLs.
    clients: HashMap<u32, String>,
    total_clients: u32,
}

impl Actor for ServerActor{
    type Context = Context<Self>;
}

impl ServerActor{
    pub fn new()->Self{
        ServerActor{
            clients: HashMap::new(),
            total_clients: 0,
        }
    }
}

impl Handler<RegisterClient> for ServerActor {
    type Result = ();

    fn handle(&mut self, msg: RegisterClient, _: &mut Self::Context) {
        self.total_clients += 1;
        self.clients.insert(self.total_clients, msg.url.clone());
        println!("Registered client {} with URL: {}", self.total_clients, msg.url);
    }
}

