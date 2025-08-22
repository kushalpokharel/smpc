use kzen_paillier::BigInt;
use serde::{de, Deserialize, Serialize};
use actix::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub enum WebsocketMessage {
    InitializeProtocol(InitializeProtocol),
    FirstRoundResponse(UnicastMessage<FirstRoundResponse>),
    Broadcast(BroadcastMessage<String>),
    Relayer(RelayerMessage<String>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitializeProtocol{
    pub bits_security: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FirstRoundResponse{
    pub computed_value: String,
    pub n_squared: BigInt
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


