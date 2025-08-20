use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum WebsocketMessage {
    InitializeProtocol(InitializeProtocol),
    
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitializeProtocol{
    pub bits_security: usize,
}