use awc::{error::{WsClientError, WsProtocolError}, http::StatusCode, ws::{Codec, Frame}, BoxedSocket};
use awc::ws::{WebsocketsRequest};
use actix_codec::Framed;

#[derive(Debug)]
pub enum WebsocketError {
  RequestError(actix_web::Error),
  ClientError(WsClientError),
  ProtocolError(WsProtocolError),
  JSONError(serde_json::Error),
  UnexpectedFrame(Frame),
  WebsocketClosed,
  UnknownError(StatusCode),
}

impl WebsocketError{
    pub async fn connect(request: WebsocketsRequest) -> Result<Framed<BoxedSocket, Codec>, Self> {
        let (response, frame) = request
        .connect()
        .await
        .map_err(|e| WebsocketError::ClientError(e))?;
        
        if response.status() == StatusCode::SWITCHING_PROTOCOLS {
            Ok(frame)
        }else{
            Err(Self::UnknownError(response.status()))
        }
    }
    
    pub fn from_client_error(error: WsClientError) -> Self {
        WebsocketError::ClientError(error)
    }
    
    pub fn from_protocol_error(error: WsProtocolError) -> Self {
        WebsocketError::ProtocolError(error)
    }
}

