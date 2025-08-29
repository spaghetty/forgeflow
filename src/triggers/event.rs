// The `event` module defines the `TEvent` struct, which represents an event that can be processed by the agent.

use serde::Serialize;
use serde_json::Value;

/// The `TEvent` struct represents an event that can be processed by the agent.
#[derive(Serialize, Debug)]
pub struct TEvent {
    /// The name of the event.
    pub name: String,
    /// The payload of the event, which can be any JSON value.
    pub payload: Option<Value>,
}
