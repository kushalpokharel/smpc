use actix::prelude::*;
use serde::{Serialize, Deserialize};


#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterClient{
    pub url: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct InitializeParameters;


