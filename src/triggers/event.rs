use serde::Serialize;
use serde_json::Value;

#[derive(Serialize, Debug)]
pub struct TEvent {
    pub name: String,
    pub payload: Option<Value>,
}
