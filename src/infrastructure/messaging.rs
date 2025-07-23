/// Messaging infrastructure
/// Handles internal messaging and event publishing

use tokio::sync::mpsc;
use serde_json::Value;
use crate::shared::error::AppResult;

pub struct MessageBroker {
    sender: mpsc::UnboundedSender<Message>,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub topic: String,
    pub payload: Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl MessageBroker {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<Message>) {
        let (sender, receiver) = mpsc::unbounded_channel();
        (Self { sender }, receiver)
    }

    pub fn publish(&self, topic: String, payload: Value) -> AppResult<()> {
        let message = Message {
            topic,
            payload,
            timestamp: chrono::Utc::now(),
        };

        self.sender.send(message)
            .map_err(|_| crate::shared::error::AppError::InternalServer("Failed to send message".to_string()))?;

        Ok(())
    }
}
