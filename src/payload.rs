use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct MessagePayload {
    pub method: String,
    partner_id: Option<String>,
    pub id: u32,
    pub params: serde_json::Value,
}

#[derive(Serialize, Debug)]
pub struct ResponsePayload {
    id: u32,
    result: serde_json::Value,
}

pub fn parse_json(payload: &String) -> MessagePayload {
    let payload: MessagePayload = serde_json::from_str(&payload).unwrap();
    payload
}

impl ResponsePayload {
    pub fn new(id: u32, result: serde_json::Value) -> ResponsePayload {
        ResponsePayload {
            id: id,
            result: result,
        }
    }
}
