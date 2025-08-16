use actix::prelude::*;
use actix_http::ws::CloseCode;

/// Helpful type to close the WebSocket connection due to an error
#[derive(Message)]
#[rtype(result = "()")]
pub struct ErrorClose(pub CloseCode, pub Option<String>);

impl From<CloseCode> for ErrorClose {
  fn from(code: CloseCode) -> Self {
    Self(code, None)
  }
}

impl<T> From<(CloseCode, T)> for ErrorClose
where
  T: Into<String>,
{
  fn from((code, description): (CloseCode, T)) -> Self {
    Self(code, Some(description.into()))
  }
}
