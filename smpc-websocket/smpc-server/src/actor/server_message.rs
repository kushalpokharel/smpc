use actix::prelude::*;
use actix_codec::Framed;
use awc::{ws::Codec, BoxedSocket};
use kzen_paillier::BigInt;
use serde::{Serialize, Deserialize};

use crate::errors::server_error::ServerError;

#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterClient{
    pub url: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct InitializeParameters;


/// Websocket messages

/// Messages that have a "from" field
pub trait OriginMessage {
  /// Extract the source of the message
  fn get_from(&self) -> usize;
}

/// Message to send to or receive from a specific websocket.
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

// #[derive(Serialize, Deserialize)]
// pub struct InitializeProtocol {
//     pub bits_security: usize,
// }
