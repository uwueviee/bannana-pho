#[macro_use]
extern crate num_derive;

use std::io::Error;

use dotenv::dotenv;
use std::env;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};

use futures_util::{future, SinkExt, StreamExt, TryStreamExt};
use tokio_tungstenite::tungstenite::Message;
use crate::OpCode::{HEARTBEAT_ACK, HELLO, READY};
use crate::opcodes::{get_opcode, MessageData, OpCode, SocketMessage};
use crate::infoops::get_infotype;

use rand::prelude::*;
use rand::distributions::Alphanumeric;

use serde_json::Value::Array;

mod opcodes;
mod infoops;
mod socket_decode;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    let addr = env::var("LISTEN_ADDR").unwrap_or("0.0.0.0:3621".to_string());

    let socket = TcpListener::bind(&addr).await.expect("Failed to bind to address!");
    println!("Listening on {}!", &addr);

    while let Ok((stream, _)) = socket.accept().await {
        let peer = stream.peer_addr().expect("Failed to connect to peer, missing address?");
        println!("Connecting to peer {}...", &peer);

        tokio::spawn(accept_conn(peer, stream));
    }

    Ok(())
}

async fn accept_conn(peer: SocketAddr, stream: TcpStream) {
    if let Err(e) = handle_conn(peer, stream).await {
        match e {
            tokio_tungstenite::tungstenite::Error::ConnectionClosed | tokio_tungstenite::tungstenite::Error::Protocol(_) | tokio_tungstenite::tungstenite::Error::Utf8 => (),
            err => eprintln!("Error accepting connection from {}!", &peer),
        }
    }
}

async fn handle_conn(peer: SocketAddr, stream: TcpStream) -> tokio_tungstenite::tungstenite::Result<()> {
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
                                        println!("IDENTIFY from {}", &peer);
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
                                        println!("INFO from {}", &peer);

                                        let info = get_infotype(msg).unwrap();
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