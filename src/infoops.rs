use std::any::Any;
use num_traits::real::Real;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use serde_repr::{Serialize_repr, Deserialize_repr};
use tokio_tungstenite::tungstenite::Message;
use crate::opcodes::INFO;

/// Info message types
#[derive(FromPrimitive, Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
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

    /// Sent by the server to indicate the success of a VOICE_STATE_CREATE.
    ///
    /// Has the same fields as VOICE_STATE_CREATE, but with extras.
    VST_DONE = 4,

    /// Sent by the client when a user is leaving a channel OR moving between channels
    /// in a guild. More on state transitions later on.
    VST_DESTROY= 5,

    /// Sent to update an existing voice state. Potentially unused.
    VST_UPDATE = 6,

}

/// Request a channel to be created inside the voice server.
///
/// The Server MUST reply back with a CHANNEL_ASSIGN when resources are
/// allocated for the channel.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct CHANNEL_REQ {
    /// Channel ID
    pub channel_id: String,

    /// Guild ID, not provided if dm / group dm
    pub guild_id: Option<String>
}

/// Sent by the Server to signal the successful creation of a voice channel.
#[derive(Deserialize, Serialize, Debug)]
pub struct CHANNEL_ASSIGN {
    /// Channel ID
    pub channel_id: String,

    /// Guild ID, not provided if dm / group dm
    pub guild_id: Option<String>,

    /// Authentication token
    pub token: String
}

/// Sent by the client to create a voice state.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct VST_CREATE {
    /// User ID
    pub user_id: String,

    /// Channel ID
    pub channel_id: String,

    /// Guild ID, not provided if dm / group dm
    pub guild_id: Option<String>
}

/// Info message data
#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum InfoData {
    /// Request a channel to be created inside the voice server.
    ///
    /// The Server MUST reply back with a CHANNEL_ASSIGN when resources are
    /// allocated for the channel.
    CHANNEL_REQ(CHANNEL_REQ),

    /// Sent by the Server to signal the successful creation of a voice channel.
    CHANNEL_ASSIGN {
        /// Channel ID
        channel_id: String,

        /// Guild ID, not provided if dm / group dm
        guild_id: Option<String>,

        /// Authentication token
        token: String
    },

    /// Sent by the client to signal the destruction of a voice channel. Be it
    /// a channel being deleted, or all members in it leaving.
    CHANNEL_DESTROY {
        /// Channel ID
        channel_id: String,

        /// Guild ID, not provided if dm / group dm
        guild_id: Option<String>
    },

    /// Sent by the client to create a voice state.
    VST_CREATE(VST_CREATE),

    /// Sent by the server to indicate the success of a VST_CREATE.
    VST_DONE {
        /// User ID
        user_id: String,

        /// Channel ID
        channel_id: String,

        /// Guild ID, not provided if dm / group dm
        guild_id: Option<String>,

        /// Session ID for the voice state
        session_id: String
    },

    /// Sent by the client when a user is leaving a channel OR moving between channels
    /// in a guild. More on state transitions later on.
    VST_DESTROY {
        /// Session ID for the voice state
        session_id: String
    },

    /// Sent to update an existing voice state. Potentially unused.
    VST_UPDATE {
        session_id: String
    }
}

pub async fn get_infotype(msg: Message) -> Result<(InfoType, InfoData), ()> {
    let message_json: Result<Value, serde_json::Error> = serde_json::from_str(
        msg.to_text().expect("Failed to convert message to str!")
    );

    if message_json.is_ok() {
        // TODO: Maybe find a better way?
        let info_data: INFO = serde_json::from_value(
            message_json.unwrap().get("d").unwrap().clone()
        ).expect("Failed to get inner data for InfoData!");

        Ok((info_data._type, info_data.data))
    } else {
        Err(())
    }
}