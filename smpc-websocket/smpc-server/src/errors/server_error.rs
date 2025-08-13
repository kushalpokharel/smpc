use std::fmt::Display;

use crate::errors::websocket_error::WebsocketError;

#[derive(Debug)]
pub enum ServerError{
    InitializationError(WebsocketError),
}

impl ServerError {
    
}