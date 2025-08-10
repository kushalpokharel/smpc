use actix::prelude::*;

#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterClient{
    pub url: String,
}