use crate::errors::websocket_error::WebsocketError;

pub enum ServerError{
    InitializationError(WebsocketError),
}

impl ServerError {
    
}