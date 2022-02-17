use serde::{Serialize, Deserialize};

#[derive(FromPrimitive, Deserialize, Serialize)]
pub enum OpCode {
    HELLO = 0,
    IDENTIFY = 1,
    RESUME = 2,
    READY = 3,
    HEARTBEAT = 4,
    HEARTBEAT_ACK = 5,
    INFO = 6
}

#[derive(FromPrimitive, Deserialize, Serialize)]
pub enum ErrorCode {
    GENERAL = 4000,
    AUTH = 4001,
    DECODE = 4002
}

#[derive(Deserialize, Serialize)]
pub enum MessageData {
    HELLO {
        heartbeat_interval: i32,
        nonce: String
    },
    IDENTIFY {
        token: String
    },
    READY {
        health: f32
    },
    HEARTBEAT,
    HEARTBEAT_ACK {
        health: f32
    },
    INFO {
        #[serde(rename = "type")]
        _type: InfoType,
        data: InfoData
    }
}

#[derive(Deserialize, Serialize)]
pub enum InfoType {
    CHANNEL_REQ = 0,
    CHANNEL_ASSIGN = 1,
    CHANNEL_DESTROY = 2,
    VST_CREATE = 3,
    VST_DONE = 4,
    VST_UPDATE = 5,
    VST_LEAVE = 6
}

#[derive(Deserialize, Serialize)]
pub enum InfoData {
    CHANNEL_REQ {
        channel_id: u64,
        guild_id: Option<u64>
    },
    CHANNEL_ASSIGN {
        channel_id: u64,
        guild_id: Option<u64>,
        token: String
    },
    CHANNEL_DESTROY {
        channel_id: u64,
        guild_id: Option<u64>
    },
    VST_CREATE {
        user_id: u64,
        channel_id: u64,
        guild_id: Option<u64>
    },
    VST_DONE {
        user_id: u64,
        channel_id: u64,
        guild_id: Option<u64>,
        session_id: String
    },
    VST_DESTROY {
        session_id: String
    }
}

#[derive(Deserialize, Serialize)]
struct SocketMessage {
    op: OpCode,
    d: MessageData
}