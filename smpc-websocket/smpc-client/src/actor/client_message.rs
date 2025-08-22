use kzen_paillier::BigInt;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum WebsocketMessage {
    InitializeProtocol(InitializeProtocol),
    
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitializeProtocol{
    pub bits_security: usize,
}

pub struct FirstRoundResponse{
    pub computed_value: String,
    pub n_squared: BigInt
}