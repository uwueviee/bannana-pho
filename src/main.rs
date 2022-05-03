#[macro_use]extern crate num_derive;

#[macro_use] extern crate log;

use std::collections::HashSet;
use std::io::Error;

use dotenv::dotenv;
use std::env;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};

use futures_util::{future, SinkExt, StreamExt, TryStreamExt};
use tokio_tungstenite::tungstenite::{client, Message};
use crate::OpCode::{HEARTBEAT_ACK, HELLO, READY};
use crate::opcodes::{get_opcode, IDENTIFY, MessageData, OpCode, SocketMessage};

use crate::infoops::{get_infotype, InfoData, InfoType};

use rand::prelude::*;
use rand::distributions::Alphanumeric;
use redis::{Client, Connection, RedisConnectionInfo};

use serde_json::Value::Array;
use crate::util::verify_token;

use redis::Commands;

mod opcodes;
mod infoops;
mod util;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    pretty_env_logger::init();

    let shared_secret = env::var("SECRET").expect("No secret present in environment!");

    let addr = env::var("LISTEN_ADDR").unwrap_or("0.0.0.0:3621".to_string());

    let redis_client = redis::Client::open(env::var("REDIS_ADDR").unwrap_or("redis://127.0.0.1:6379".to_string())).expect("Failed to connect to Redis server!");

    let socket = TcpListener::bind(&addr).await.expect("Failed to bind to address!");
    info!("Listening on {}!", &addr);

    while let Ok((stream, _)) = socket.accept().await {
        let peer = stream.peer_addr().expect("Failed to connect to peer, missing address?");
        info!(target: "initial", "Connecting to peer {}...", &peer);

        tokio::spawn(accept_conn(peer, stream, redis_client.clone(), shared_secret.clone()));
    }

    Ok(())
}

async fn accept_conn(peer: SocketAddr, stream: TcpStream, redis_client: Client, shared_secret: String) {
    if let Err(e) = handle_conn(peer, stream, redis_client, shared_secret).await {
        match e {
            tokio_tungstenite::tungstenite::Error::ConnectionClosed | tokio_tungstenite::tungstenite::Error::Protocol(_) | tokio_tungstenite::tungstenite::Error::Utf8 => (),
            err => error!(target: "initial", "Error accepting connection from {}!", &peer),
        }
    }
}

async fn handle_conn(peer: SocketAddr, stream: TcpStream, redis_client: Client, shared_secret: String) -> tokio_tungstenite::tungstenite::Result<()> {
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await;

    if ws_stream.is_err() {
        warn!(target: "initial", "Failed to complete the websocket handshake! Dropping {}!", peer);

        return Ok(());
    }

    let ws_stream = ws_stream.unwrap();

    info!(target: "socket", "Connected to peer: {}!", &peer);

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let mut heartbeat = tokio::time::interval(Duration::from_millis(1000));

    let mut redis = redis_client.get_connection().expect("Failed to get Redis connection!");

    let mut nonce: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();

    let _: () = redis.set(format!("{}_nonce", peer), &nonce).expect("Failed to insert nonce!");

    debug!(target: "socket", "HELLO to {}", &peer);
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

    let mut identified: bool = false;

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

                                // Check if identified
                                if !identified && !(op.0 == OpCode::IDENTIFY) {
                                    ws_sender.send(Message::Text((opcodes::ErrorCode::AUTH as i32).to_string())).await?;

                                    continue;
                                }

                                match op.0 {
                                    OpCode::IDENTIFY => {
                                        if let MessageData::IDENTIFY(dn) = op.1 {
                                            debug!(target: "socket", "IDENTIFY from {}", &peer);

                                            let nonce: Option<String> = redis.get(format!("{}_nonce", peer)).expect("Failed to get nonce from Redis!");

                                            if verify_token(shared_secret.clone(), nonce, dn.token).await {
                                                debug!(target: "socket", "READY to {}", &peer);
                                                ws_sender.send(Message::Text(
                                                    serde_json::to_string(
                                                        &SocketMessage {
                                                            op: READY,
                                                            d: MessageData::READY {
                                                                health: 6.9 // trust
                                                            }
                                                        }
                                                    ).unwrap().to_owned()
                                                )).await?;

                                                identified = true;
                                            } else {
                                                ws_sender.send(Message::Text((opcodes::ErrorCode::AUTH as i32).to_string())).await?;
                                            }
                                        } else {
                                            ws_sender.send(Message::Text((opcodes::ErrorCode::DECODE as i32).to_string())).await?;
                                        }
                                    }

                                    OpCode::RESUME => {
                                        debug!(target: "socket", "RESUME from {}", &peer);
                                        unimplemented!()
                                    }

                                    OpCode::HEARTBEAT => {
                                        debug!(target: "socket", "HEARTBEAT from {}", &peer);
                                        debug!(target: "socket", "HEARTBEAT_ACK to {}", &peer);
                                        ws_sender.send(Message::Text(
                                            serde_json::to_string(
                                                &SocketMessage {
                                                    op: HEARTBEAT_ACK,
                                                    d: MessageData::HEARTBEAT_ACK {
                                                        health: 6.9 // trust
                                                    }
                                                }
                                            ).unwrap().to_owned()
                                        )).await?;
                                    }

                                    OpCode::INFO => {
                                        let info_data = get_infotype(msg.clone()).await;

                                        if info_data.is_ok() {
                                            let info = info_data.unwrap();

                                            debug!(target: "socket", "INFO from {} with type {:?}", &peer,  &info.0);

                                            match info.0 {
                                                InfoType::CHANNEL_REQ => {
                                                    if let InfoData::CHANNEL_REQ(dn) = info.1 {
                                                        let guild_id = dn.clone().guild_id.unwrap_or("dm".to_string());
                                                        debug!(target: "socket", "Creating voice channel for {} in {}", &dn.channel_id, &guild_id);

                                                        let token: String = rand::thread_rng()
                                                            .sample_iter(&Alphanumeric)
                                                            .take(64)
                                                            .map(char::from)
                                                            .collect();

                                                        let mut channel_set: HashSet<String> = HashSet::new();

                                                        if channel_set.insert(format!("token_{}", token)) {
                                                            let _: () = redis.sadd(format!("{}_{}_voice", guild_id, &dn.channel_id), channel_set)
                                                                .expect("Failed to insert into Redis!");

                                                            debug!(target: "socket", "CHANNEL_ASSIGN to {}", &peer);

                                                            ws_sender.send(Message::Text(
                                                                serde_json::to_string(
                                                                    &SocketMessage {
                                                                        op: OpCode::INFO,
                                                                        d: MessageData::INFO {
                                                                            _type: InfoType::CHANNEL_ASSIGN,
                                                                            data: InfoData::CHANNEL_ASSIGN {
                                                                                channel_id: dn.channel_id,
                                                                                guild_id: dn.guild_id,
                                                                                token
                                                                            }
                                                                        }
                                                                    }
                                                                ).unwrap().to_owned()
                                                            )).await?;
                                                        } else {
                                                            // cry about it
                                                            ws_sender.send(Message::Text((opcodes::ErrorCode::GENERAL as i32).to_string())).await?;
                                                        }
                                                    } else {
                                                        ws_sender.send(Message::Text((opcodes::ErrorCode::DECODE as i32).to_string())).await?;
                                                    }
                                                },
                                                InfoType::CHANNEL_DESTROY => todo!(),
                                                InfoType::VST_CREATE => {
                                                    if let InfoData::VST_CREATE(dn) = info.1 {
                                                        let guild_id = dn.clone().guild_id.unwrap_or("dm".to_string());
                                                        debug!(target: "socket", "Creating voice state for {} in {}", &dn.channel_id, &guild_id);

                                                        let session_id: String = rand::thread_rng()
                                                            .sample_iter(&Alphanumeric)
                                                            .take(32)
                                                            .map(char::from)
                                                            .collect();

                                                        let mut channel_set: HashSet<String> = HashSet::new();

                                                        if channel_set.insert(format!("{}", session_id)) {
                                                            let _: () = redis.sadd(format!("{}_{}_voice", guild_id, &dn.channel_id), channel_set)
                                                                .expect("Failed to insert into Redis!");

                                                            debug!(target: "socket", "VOICE_STATE_DONE to {}", &peer);

                                                            ws_sender.send(Message::Text(
                                                                serde_json::to_string(
                                                                    &SocketMessage {
                                                                        op: OpCode::INFO,
                                                                        d: MessageData::INFO {
                                                                            _type: InfoType::VST_DONE,
                                                                            data: InfoData::VST_DONE {
                                                                                user_id: dn.user_id,
                                                                                channel_id: dn.channel_id,
                                                                                guild_id: dn.guild_id,
                                                                                session_id
                                                                            }
                                                                        }
                                                                    }
                                                                ).unwrap().to_owned()
                                                            )).await?;
                                                        } else {
                                                            // cry about it
                                                            ws_sender.send(Message::Text((opcodes::ErrorCode::GENERAL as i32).to_string())).await?;
                                                        }
                                                    } else {
                                                        ws_sender.send(Message::Text((opcodes::ErrorCode::DECODE as i32).to_string())).await?;
                                                    }
                                                },
                                                InfoType::VST_UPDATE => todo!(),
                                                InfoType::VST_DESTROY => todo!(),
                                                _ => {
                                                    ws_sender.send(Message::Text((opcodes::ErrorCode::DECODE as i32).to_string())).await?;
                                                }
                                            }
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