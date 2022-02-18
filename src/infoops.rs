use std::any::Any;
use num_traits::real::Real;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use serde_repr::{Serialize_repr, Deserialize_repr};
use tokio_tungstenite::tungstenite::Message;

/// Info message types
#[derive(Serialize_repr, Deserialize_repr)]
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
#[serde(untagged)]
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

#[derive(Deserialize, Serialize)]
pub struct InfoMessage {
    /// Info type
    #[serde(rename = "type")]
    _type: InfoType,

    /// Info data, varies depending on InfoType
    data: InfoData
}

pub fn get_infotype(msg: Message) -> Result<(InfoType, InfoData), ()> {
    let message_json: Result<Value, serde_json::Error> = serde_json::from_str(
        msg.to_text().expect("Failed to convert message to str!")
    );

    if message_json.is_ok() {
        // TODO: Maybe find a better way?
        let info_data: InfoMessage = serde_json::from_value(
            message_json.unwrap().get("d").unwrap().clone()
        ).expect("Failed to get inner data for InfoData!");

        Ok((info_data._type, info_data.data))
    } else {
        Err(())
    }
}