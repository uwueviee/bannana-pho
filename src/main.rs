#[macro_use]
extern crate num_derive;

use std::io::Error;

use dotenv::dotenv;
use std::env;
use std::net::SocketAddr;
use std::time::Duration;
use fred::client::RedisClient;
use tokio::net::{TcpListener, TcpStream};

use futures_util::{future, SinkExt, StreamExt, TryStreamExt};
use tokio_tungstenite::tungstenite::{client, Message};
use crate::OpCode::{HEARTBEAT_ACK, HELLO, READY};
use crate::opcodes::{get_opcode, IDENTIFY, MessageData, OpCode, SocketMessage};

use crate::infoops::get_infotype;

use rand::prelude::*;
use rand::distributions::Alphanumeric;

use serde_json::Value::Array;
use crate::util::verify_token;

mod opcodes;
mod infoops;
mod util;
mod redis;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    let shared_secret = env::var("SECRET").expect("No secret present in environment!");

    let addr = env::var("LISTEN_ADDR").unwrap_or("0.0.0.0:3621".to_string());
    let redis_addr: (String, u16) = (
        env::var("REDIS_HOST").unwrap_or("127.0.0.1:6379".to_string()),
        env::var("REDIS_PORT").unwrap_or("6379".to_string()).parse::<u16>().expect("Failed to get Redis port!")
    );

    let redis = redis::connect_redis(redis_addr.0, redis_addr.1).await.expect("Failed to connect to Redis!");

    let socket = TcpListener::bind(&addr).await.expect("Failed to bind to address!");
    println!("Listening on {}!", &addr);

    while let Ok((stream, _)) = socket.accept().await {
        let peer = stream.peer_addr().expect("Failed to connect to peer, missing address?");
        println!("Connecting to peer {}...", &peer);

        tokio::spawn(accept_conn(peer, stream, redis.clone(), shared_secret.clone()));
    }

    Ok(())
}

async fn accept_conn(peer: SocketAddr, stream: TcpStream, redis: RedisClient, shared_secret: String) {
    if let Err(e) = handle_conn(peer, stream, redis, shared_secret).await {
        match e {
            tokio_tungstenite::tungstenite::Error::ConnectionClosed | tokio_tungstenite::tungstenite::Error::Protocol(_) | tokio_tungstenite::tungstenite::Error::Utf8 => (),
            err => eprintln!("Error accepting connection from {}!", &peer),
        }
    }
}

async fn handle_conn(peer: SocketAddr, stream: TcpStream, redis: RedisClient, shared_secret: String) -> tokio_tungstenite::tungstenite::Result<()> {
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Failed to complete the websocket handshake!");
    println!("Connected to peer: {}!", &peer);

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let mut heartbeat = tokio::time::interval(Duration::from_millis(1000));

    let mut nonce: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();

    let _: () = redis.set(format!("{}_nonce", peer), &nonce, None, None, false)
        .await.expect("Failed to insert nonce!");

    println!("HELLO to {}", &peer);
    ws_sender.send(Message::Text(
        serde_json::to_string(
            &SocketMessage {
                op: HELLO,
                d: MessageData::HELLO {
                    heartbeat_interval: env::var("HEARTBEAT_INTERVAL").
                        unwrap_or("1".to_string())
                        .parse::<i32>()
                        .unwrap_or(1),
                    nonce
                }
            }
        ).unwrap().to_owned()
    )).await?;

    loop {
        tokio::select! {
            msg = ws_receiver.next() => {
                match msg {
                    Some(msg) => {
                        let msg = msg?;

                        if msg.is_text() {
                            let op = get_opcode(msg.clone());
                            if op.is_ok() {
                                let op = op.unwrap();
                                match op.0 {
                                    OpCode::IDENTIFY => {
                                        if let MessageData::IDENTIFY(dn) = op.1 {
                                            println!("IDENTIFY from {}", &peer);

                                            let nonce: Option<String> = redis.get(format!("{}_nonce", peer)).await.expect("Failed to get nonce from Redis!");

                                            if verify_token(shared_secret.clone(), nonce, dn.token).await {
                                                println!("READY to {}", &peer);
                                                ws_sender.send(Message::Text(
                                                    serde_json::to_string(
                                                        &SocketMessage {
                                                            op: READY,
                                                            d: MessageData::READY {
                                                                health: 1.0 // trust
                                                            }
                                                        }
                                                    ).unwrap().to_owned()
                                                )).await?;
                                            } else {
                                                ws_sender.send(Message::Text((opcodes::ErrorCode::AUTH as i32).to_string())).await?;
                                            }
                                        } else {
                                            ws_sender.send(Message::Text((opcodes::ErrorCode::DECODE as i32).to_string())).await?;
                                        }
                                    }

                                    OpCode::RESUME => {
                                        println!("RESUME from {}", &peer);
                                        unimplemented!()
                                    }

                                    OpCode::HEARTBEAT => {
                                        println!("HEARTBEAT from {}", &peer);
                                        println!("HEARTBEAT_ACK to {}", &peer);
                                        ws_sender.send(Message::Text(
                                            serde_json::to_string(
                                                &SocketMessage {
                                                    op: HEARTBEAT_ACK,
                                                    d: MessageData::HEARTBEAT_ACK {
                                                        health: 1.0 // trust
                                                    }
                                                }
                                            ).unwrap().to_owned()
                                        )).await?;
                                    }

                                    OpCode::INFO => {
                                        let info_data = get_infotype(msg.clone());

                                        if info_data.is_ok() {
                                            println!("INFO from {} with type {}", &peer,  info_data.unwrap().0 as u8);
                                        } else {
                                            ws_sender.send(Message::Text((opcodes::ErrorCode::DECODE as i32).to_string())).await?;
                                        }
                                    },

                                    _ => {
                                        ws_sender.send(Message::Text((opcodes::ErrorCode::DECODE as i32).to_string())).await?;
                                    }
                                }
                            } else {
                                 ws_sender.send(Message::Text((opcodes::ErrorCode::DECODE as i32).to_string())).await?;
                            }
                        } else if msg.is_close() {
                            break;
                        }
                    },
                    None => break,
                }
            },
            _ = heartbeat.tick() => {
                //ws_sender.send(Message::Text("deez".to_owned())).await?;
            }
        }
    }

    Ok(())
}