use std::marker::PhantomData;

use actix::{Actor, StreamHandler};
use actix_web_actors::ws;
use actix::ActorContext;
use kzen_paillier::*;
use shared::types::{ClientMessage, FirstRoundResponse, InitializeProtocol, SecondRoundResponse, UnicastMessage, WebsocketMessage};
use crate::actor::consts::SETUP;
use curv::arithmetic::{BigInt, Modulo};
use shared::utils::{EncodedCiphertextRepr, get_bigint_from_encoded_ciphertext};

pub struct ClientActor{
    // if this is the first client, it will generate and store the decryption key which will be used to decrypt and obtain the final result
    decryption_key: Option<DecryptionKey>,
}


impl Actor for ClientActor {
    type Context = ws::WebsocketContext<Self>;
}

impl ClientActor{
    pub fn new() -> Self {
        ClientActor{
            decryption_key: None,
        }
    }

    pub fn send_unicast<T>(&self, from: usize, to: usize, data: T, ctx: &mut ws::WebsocketContext<Self>) where T: serde::Serialize {
        let data_value = serde_json::to_value(data).unwrap_or_else(|e| {
            eprintln!("Failed to convert UnicastMessage to value: {}", e);
            serde_json::Value::Null
        });
        let uni_msg = UnicastMessage::new(from, to, data_value);

        let msg: WebsocketMessage = WebsocketMessage::Unicast(uni_msg);
        let json_str = serde_json::to_string(&msg).unwrap_or_else(|e| {
            eprintln!("Failed to serialize message: {}", e);
            "".to_string()
        });
        ctx.text(json_str);
    }

    pub fn _send_json<T>(&self, msg: &T, ctx: &mut ws::WebsocketContext<Self>) where T: serde::Serialize {
        let json_str = serde_json::to_string(msg).unwrap_or_else(|e| {
            eprintln!("Failed to serialize message: {}", e);
            "".to_string()
        });
        ctx.text(json_str);

    }

    pub fn start_protocol(&mut self, init: InitializeProtocol,  ctx: &mut ws::WebsocketContext<Self>) {
        // Start the protocol by sending an initialization message or any other setup
        let num_bits = init.bits_security;
        // generate the private and public Paillier keys here
        let kp = Paillier::keypair_with_modulus_size(num_bits).keys();
        self.decryption_key = Some(kp.1.clone());
        let start_value =  SETUP.private_input;
        println!("Private input chosen, {}", start_value);
        let encrypted_value = Paillier::encrypt(&kp.0, start_value);
        let encrypted_value_bigint = get_bigint_from_encoded_ciphertext(&encrypted_value);
        
        if init.sid == init.num_parties - 2 {
            // if this is the second last client, send the SecondRoundResponse to the last client
            let new_msg = SecondRoundResponse{
                computed_value: encrypted_value_bigint,
                num_parties: init.num_parties,
                sid: init.sid + 1,
                n_squared: kp.0.nn,
                n: kp.0.n
            };
            self.send_unicast(init.sid, init.sid + 1, ClientMessage::SecondRoundResponse(new_msg), ctx);

        }
        else{
            let new_msg = FirstRoundResponse{
                computed_value: encrypted_value_bigint,
                num_parties: init.num_parties,
                sid: init.sid + 1,
                n_squared:kp.0.nn,
                n: kp.0.n
            };
            self.send_unicast(init.sid, init.sid+1, ClientMessage::FirstRoundResponse(new_msg), ctx);

        }
        
        
    }

    pub fn second_round_response(&self, response: SecondRoundResponse, ctx: &mut ws::WebsocketContext<Self>) {
        // Handle the second round response
        let data = response;
        let resp = data.computed_value;
        println!("Received second round response: {:?}", resp);
        
        // if this the first client, decrypt the final result and print it
        if data.sid == 0 {
            if let Some(dec_key) = &self.decryption_key {
                let rct = RawCiphertext::from(resp);
                let decrypted_result = Paillier::decrypt(dec_key, &rct);
                println!("Final decrypted result: {}", decrypted_result.0.into_owned());
            } else {
                eprintln!("Decryption key not found for the first client");
            }
            return;
        }
        let enc_key:EncryptionKey  = EncryptionKey {
            n: data.n.clone(),
            nn: data.n_squared.clone(),
        };
        let ct = Paillier::encrypt(&enc_key, SETUP.random_value);
        println!("Random value chosen, {}", SETUP.random_value);

        let ct_raw = get_bigint_from_encoded_ciphertext(&ct);
        BigInt::mod_inv(&ct_raw, &enc_key.n)
            .map(|inv| {
                let new_ct = BigInt::mod_mul(&resp, &inv, &enc_key.nn);
                // get_bigint_from_encoded_ciphertext(new)
                let new_response = SecondRoundResponse{
                    computed_value: new_ct,
                    n_squared: enc_key.nn,
                    num_parties: data.num_parties,
                    sid: data.sid - 1,
                    n: enc_key.n,
                };
                self.send_unicast(data.sid, data.sid-1, ClientMessage::SecondRoundResponse(new_response), ctx);
            }).unwrap_or_else(|| {
                eprintln!("Failed to compute modular inverse");
            });
    }

    pub fn first_round_response(&self, response: FirstRoundResponse, ctx: &mut ws::WebsocketContext<Self>) {
        // Handle the first round response
        println!("Received first round response: {:?}", response);
        // get the computed value from the response and raise it to the power of 
        let data = response;
        let computed_value = data.computed_value;
        let new_ct = BigInt::mod_pow(&computed_value, &BigInt::from(SETUP.private_input), &data.n_squared);
        println!("Private input chosen, {}", SETUP.private_input);

        if data.sid == data.num_parties - 2 {
            // if this is the second last client, send the SecondRoundResponse to the last client
            let new_msg = SecondRoundResponse{
                computed_value: new_ct,
                num_parties: data.num_parties,
                sid: data.sid + 1,
                n_squared: data.n_squared,
                n: data.n
            };
            self.send_unicast(data.sid, data.sid + 1, ClientMessage::SecondRoundResponse(new_msg), ctx);

        }
        else{
            let new_msg = FirstRoundResponse{
                computed_value: new_ct,
                num_parties: data.num_parties,
                sid: data.sid + 1,
                n_squared: data.n_squared,
                n: data.n
            };
            self.send_unicast(data.sid, data.sid + 1, ClientMessage::FirstRoundResponse(new_msg), ctx);
        }
          
    }
}

impl StreamHandler<Result<actix_http::ws::Message, ws::ProtocolError>> for ClientActor {
    fn handle(&mut self, msg: Result<actix_http::ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(actix_http::ws::Message::Text(text)) => {
                println!("Received text message: {}", text);
                let msg = match serde_json::from_str::<ClientMessage>(&text){
                    Ok(message) => message,
                    Err(e) => {
                        println!("Failed to parse message: {}", e);
                        // this is basically not being able to parse the result sent by one of the clients. which is fatal
                        // should be handled more gracefully.
                        return;
                    }
                };
                match msg {
                    ClientMessage::InitializeProtocol(init) => {
                        self.start_protocol(init, ctx);
                    }
                    ClientMessage::FirstRoundResponse(msg) => {
                        self.first_round_response(msg, ctx);
                    }
                    ClientMessage::SecondRoundResponse(msg) => {
                        self.second_round_response(msg, ctx);
                    }
                    _ => {
                        println!("Received unsupported WebsocketMessage variant: {:?}", msg);
                        // Handle other message types as needed
                        // For example, you could send a response back to the client
                        ctx.text("Unsupported message type received");
                    }

                }
            }
            Ok(actix_http::ws::Message::Binary(bin)) => {
                println!("Received binary message");
                ctx.binary(bin);
            }
            Ok(actix_http::ws::Message::Close(reason)) => {
                println!("WebSocket closed: {:?}", reason);
                ctx.close(reason);
            }
            Ok(_) => {
                println!("Received unsupported message type");
            }
            Err(e) => {
                println!("WebSocket error: {:?}", e);
                ctx.stop();
            }
        }
    }
}


