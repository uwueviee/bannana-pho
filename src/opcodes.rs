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
use serde::{Serialize, Deserialize};

/// Op codes sent/received by Litecord
#[derive(FromPrimitive, Deserialize, Serialize)]
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

/// Message data for the socket
#[derive(Deserialize, Serialize)]
pub enum MessageData {
    /// Sent by the server when a connection is established.
    HELLO {
        /// Amount of milliseconds to heartbeat with
        heartbeat_interval: i32,

        /// Random 10-character string used in authentication
        nonce: String
    },

    /// Sent by the client to identify itself.
    IDENTIFY {
        /// HMAC SHA256 string of a shared secret and the HELLO nonce
        token: String
    },

    READY {
        /// Health of the server (where 0 is worst and 1 is best)
        health: f32
    },

    /// Sent by the client as a keepalive / health monitoring method.
    ///
    /// The server MUST reply with a HEARTBEAT_ACK message back in a reasonable
    /// time period.
    HEARTBEAT,

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
    INFO {
        /// Info type
        #[serde(rename = "type")]
        _type: InfoType,

        /// Info data, varies depending on InfoType
        data: InfoData
    }
}

/// Info message types
#[derive(Deserialize, Serialize)]
pub enum InfoType {
    /// Request a channel to be created inside the voice server.
    CHANNEL_REQ = 0,

    /// Sent by the Server to signal the successful creation of a voice channel.
    CHANNEL_ASSIGN = 1,

    /// Sent by the client to signal the destruction of a voice channel. Be it
    /// a channel being deleted, or all members in it leaving.
    CHANNEL_DESTROY = 2,

    /// Sent by the client to create a voice state.
    VST_CREATE = 3,

    /// Sent by the server to indicate the success of a VST_CREATE.
    VST_DONE = 4,

    /// Sent by the client when a user is leaving a channel OR moving between channels
    /// in a guild. More on state transitions later on.
    VST_UPDATE = 5,

    /// Voice state leave.
    VST_LEAVE = 6
}

/// Info message data
#[derive(Deserialize, Serialize)]
pub enum InfoData {
    /// Request a channel to be created inside the voice server.
    ///
    /// The Server MUST reply back with a CHANNEL_ASSIGN when resources are
    /// allocated for the channel.
    CHANNEL_REQ {
        /// Channel ID
        channel_id: u64,

        /// Guild ID, not provided if dm / group dm
        guild_id: Option<u64>
    },

    /// Sent by the Server to signal the successful creation of a voice channel.
    CHANNEL_ASSIGN {
        /// Channel ID
        channel_id: u64,

        /// Guild ID, not provided if dm / group dm
        guild_id: Option<u64>,

        /// Authentication token
        token: String
    },

    /// Sent by the client to signal the destruction of a voice channel. Be it
    /// a channel being deleted, or all members in it leaving.
    CHANNEL_DESTROY {
        /// Channel ID
        channel_id: u64,

        /// Guild ID, not provided if dm / group dm
        guild_id: Option<u64>
    },

    /// Sent by the client to create a voice state.
    VST_CREATE {
        /// User ID
        user_id: u64,

        /// Channel ID
        channel_id: u64,

        /// Guild ID, not provided if dm / group dm
        guild_id: Option<u64>
    },

    /// Sent by the server to indicate the success of a VST_CREATE.
    VST_DONE {
        /// User ID
        user_id: u64,

        /// Channel ID
        channel_id: u64,

        /// Guild ID, not provided if dm / group dm
        guild_id: Option<u64>,

        /// Session ID for the voice state
        session_id: String
    },

    /// Sent by the client when a user is leaving a channel OR moving between channels
    /// in a guild. More on state transitions later on.
    VST_DESTROY {
        /// Session ID for the voice state
        session_id: String
    }
}

/// Message data is defined by each opcode.
///
/// **Note:** the snowflake type follows the same rules as the Discord Gateway's
/// snowflake type: A string encoding a Discord Snowflake.
#[derive(Deserialize, Serialize)]
struct SocketMessage {
    /// Operator code
    op: OpCode,

    /// Message data
    d: MessageData
}