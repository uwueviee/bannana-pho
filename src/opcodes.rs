//! LVSP is a protocol for Litecord to communicate with an external component
//! dedicated for voice data. The voice server is responsible for the
//! Voice Websocket Discord and Voice UDP connections.
//!
//! LVSP runs over a long-lived websocket with TLS. The encoding is JSON.
//!
//! The message data is defined by each opcode.
//!
//! **Note:** the snowflake type follows the same rules as the Discord Gateway's
//! snowflake type: A string encoding a Discord Snowflake.
//!
//! [Source](https://gitlab.com/litecord/litecord/-/blob/master/docs/lvsp.md)
use std::any::Any;
use num_traits::real::Real;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use serde_repr::{Serialize_repr, Deserialize_repr};
use tokio_tungstenite::tungstenite::Message;
use crate::infoops::{InfoData, InfoType};

/// Op codes sent/received by Litecord
#[derive(FromPrimitive, Serialize_repr, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub enum OpCode {
    /// Sent by the server when a connection is established.
    HELLO = 0,

    /// Sent by the client to identify itself.
    IDENTIFY = 1,

    RESUME = 2,

    READY = 3,

    /// Sent by the client as a keepalive / health monitoring method.
    ///
    /// The server MUST reply with a HEARTBEAT_ACK message back in a reasonable
    /// time period.
    HEARTBEAT = 4,

    /// Sent by the server in reply to a HEARTBEAT message coming from the client.
    HEARTBEAT_ACK = 5,

    /// Sent by either client or a server to send information between eachother.
    ///
    /// The INFO message is extensible in which many request / response scenarios
    /// are laid on.
    INFO = 6
}

/// Possible error codes
#[derive(FromPrimitive, Deserialize, Serialize)]
pub enum ErrorCode {
    /// General error, reconnect
    GENERAL = 4000,

    /// Authentication failure
    AUTH = 4001,

    /// Decode error, given message failed to decode as json
    DECODE = 4002
}

/// Sent by the client to identify itself.
#[derive(Deserialize, Serialize)]
pub struct IDENTIFY {
    /// HMAC SHA256 string of a shared secret and the HELLO nonce
    pub token: String
}

/// Sent by either client or a server to send information between each other.
///
/// The INFO message is extensible in which many request / response scenarios are laid on.
#[derive(Deserialize, Serialize)]
pub struct INFO {
    /// Info type
    #[serde(rename = "type")]
    pub _type: InfoType,

    /// Info data, varies depending on InfoType
    pub data: InfoData
}

/// Message data for the socket
#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum MessageData {
    /// Sent by the server when a connection is established.
    HELLO {
        /// Amount of milliseconds to heartbeat with
        heartbeat_interval: i32,

        /// Random 10-character string used in authentication
        nonce: String
    },

    /// Sent by the client to identify itself.
    IDENTIFY(IDENTIFY),

    READY {
        /// Health of the server (where 0 is worst and 1 is best)
        health: f32
    },

    /// Sent by the client as a keepalive / health monitoring method.
    ///
    /// The server MUST reply with a HEARTBEAT_ACK message back in a reasonable
    /// time period.
    HEARTBEAT {},

    /// Sent by the server in reply to a HEARTBEAT message coming from the client.
    ///
    /// The `health` field is a measure of the server's overall health. It is a
    /// float going from 0 to 1, where 0 is the worst health possible, and 1 is the
    /// best health possible.
    HEARTBEAT_ACK {
        /// Health of the server (where 0 is worst and 1 is best)
        health: f32
    },

    /// Sent by either client or a server to send information between eachother.
    ///
    /// The INFO message is extensible in which many request / response scenarios
    /// are laid on.
    INFO(INFO)
}

/// Message data is defined by each opcode.
///
/// **Note:** the snowflake type follows the same rules as the Discord Gateway's
/// snowflake type: A string encoding a Discord Snowflake.
#[derive(Deserialize, Serialize)]
pub struct SocketMessage {
    /// Operator code
    pub op: OpCode,

    /// Message data
    pub d: MessageData
}


pub fn get_opcode(msg: Message) -> Result<(OpCode, MessageData), ()> {
    let message_json: Result<SocketMessage, serde_json::Error> = serde_json::from_str(msg.to_text().expect("Failed to convert message to str!"));

    if message_json.is_ok() {
        let output = message_json.unwrap();

        Ok((output.op, output.d))
    } else {
        Err(())
    }
}