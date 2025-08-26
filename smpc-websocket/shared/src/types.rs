use kzen_paillier::BigInt;
use serde::{Deserialize, Serialize};
use actix::prelude::*;
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub enum WebsocketMessage {
    Unicast(UnicastMessage<Value>),
    Broadcast(BroadcastMessage<Value>),
    Relayer(RelayerMessage<Value>),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage{
    InitializeProtocol(InitializeProtocol),
    FirstRoundResponse(FirstRoundResponse),
    SecondRoundResponse(SecondRoundResponse),
}

// Sent from server to the first  client to initialize the protocol. the sid will always be 0 in this msg type.
#[derive(Debug, Serialize, Deserialize)]
pub struct InitializeProtocol{
    pub bits_security: usize,
    pub num_parties: usize,
    pub sid: usize,
}

// Sent from one cient to other clients. Every client will add 1 to its sid and send it to the next client. Server just relays this message.
// Every client will check if it is the last client in the protocol by checking if its sid is equal to num_parties - 1.
// If it is the last client, it will send the SecondRoundResponse message to the server.
#[derive(Debug, Serialize, Deserialize)]
pub struct FirstRoundResponse{
    pub computed_value: BigInt,
    pub num_parties: usize,
    pub sid: usize,
    // used to get the publick key of the first  
    pub n_squared: BigInt,
    pub n: BigInt,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecondRoundResponse{
    pub computed_value: BigInt,
    pub n_squared: BigInt,
    pub num_parties: usize,
    pub sid: usize,
    pub n: BigInt,
}


/// Messages that have a "from" field
pub trait OriginMessage {
  /// Extract the source of the message
  fn get_from(&self) -> usize;
}


///
/// Message to send to or receive from a specific websocket.
///
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[serde(rename_all = "camelCase")]
#[rtype(result = "()")]
pub struct UnicastMessage<T> {
  pub from: usize,
  pub to: usize,
  pub data: T,
}

impl<T> UnicastMessage<T> {
  pub fn new(from: usize, to: usize, data: T) -> Self {
    Self { from, to, data }
  }

  pub fn into_inner(self) -> T {
    self.data
  }

  pub fn get_value(&self) -> &T {
    &self.data
  }
}

impl<T> OriginMessage for UnicastMessage<T> {
  fn get_from(&self) -> usize {
    self.from
  }
}

///
/// Message to send to the mediator
///
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[serde(rename_all = "camelCase")]
#[rtype(result = "()")]
pub struct RelayerMessage<T> {
  pub from: usize,
  pub to: (),
  pub data: T,
}

impl<T> RelayerMessage<T> {
  pub fn new(from: usize, data: T) -> Self {
    Self { from, to: (), data }
  }

  pub fn into_inner(self) -> T {
    self.data
  }
}

impl<T> OriginMessage for RelayerMessage<T> {
  fn get_from(&self) -> usize {
    self.from
  }
}

///
/// Message to send to or receive from ALL websockets
///
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[serde(rename_all = "camelCase")]
#[rtype(result = "()")]
pub struct BroadcastMessage<T> {
  pub from: usize,
  pub data: T,
}

impl<T> OriginMessage for BroadcastMessage<T> {
  fn get_from(&self) -> usize {
    self.from
  }
}

impl<T> BroadcastMessage<T> {
  pub fn new(from: usize, data: T) -> Self {
    Self { from, data }
  }

  pub fn into_inner(self) -> T {
    self.data
  }
}


