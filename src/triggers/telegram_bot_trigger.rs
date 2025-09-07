// The `telegram_bot_trigger` module provides a trigger that listens for Telegram messages.

use crate::triggers::{event::TEvent, Trigger, TriggerError};
use async_trait::async_trait;
use serde_json::json;
use std::env;
use teloxide::{prelude::*, types::Update, Bot};
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

/// A builder for [`TelegramBotTrigger`].
pub struct TelegramBotTriggerBuilder {
    token: Option<String>,
}

impl TelegramBotTriggerBuilder {
    /// Creates a new `TelegramBotTriggerBuilder`.
    pub fn new() -> Self {
        Self { token: None }
    }

    /// Sets the Telegram bot token.
    ///
    /// If not set, the token will be read from the `TELEGRAM_BOT_TOKEN` environment variable.
    pub fn with_token(mut self, token: &str) -> Self {
        self.token = Some(token.to_string());
        self
    }

    /// Builds a `TelegramBotTrigger`.
    pub fn build(&self) -> Result<TelegramBotTrigger, TriggerError> {
        let token = match &self.token {
            Some(token) => token.clone(),
            None => env::var("TELEGRAM_BOT_TOKEN").map_err(|_| TriggerError::ActivationError)?,
        };

        let bot = Bot::new(token);

        Ok(TelegramBotTrigger { bot })
    }
}

impl Default for TelegramBotTriggerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A trigger that listens for incoming Telegram messages.
#[derive(Clone)]
pub struct TelegramBotTrigger {
    bot: Bot,
}

#[async_trait]
impl Trigger for TelegramBotTrigger {
    /// Launches the trigger's long-running task to listen for Telegram updates.
    async fn launch(
        &self,
        tx: mpsc::Sender<TEvent>,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<JoinHandle<()>, TriggerError> {
        let bot = self.bot.clone();

        let task_handle = tokio::spawn(async move {
            info!("TelegramBotTrigger started, listening for messages");

            let handler = |_bot: Bot, msg: Message, tx: mpsc::Sender<TEvent>| async move {
                if let Some(text) = msg.text() {
                    let event = TEvent {
                        name: "TelegramMessage".to_string(),
                        payload: Some(json!({
                            "message_id": msg.id.0,
                            "chat_id": msg.chat.id.0,
                            "username": msg.from.as_ref().and_then(|u| u.username.as_ref()),
                            "first_name": msg.from.as_ref().map(|u| &u.first_name),
                            "text": text,
                            "date": msg.date.timestamp(),
                        })),
                    };

                    if let Err(e) = tx.send(event).await {
                        warn!("Failed to send Telegram event: {}", e);
                    } else {
                        debug!("Sent Telegram event for message: {}", text);
                    }
                }

                respond(())
            };

            let mut dispatcher = Dispatcher::builder(
                bot,
                Update::filter_message().endpoint(move |bot, msg| {
                    let tx_clone = tx.clone();
                    handler(bot, msg, tx_clone)
                }),
            )
            .build();

            tokio::select! {
                _ = dispatcher.dispatch() => {
                    info!("Telegram dispatcher ended normally");
                }
                _ = shutdown_rx.recv() => {
                    info!("TelegramBotTrigger received shutdown signal, terminating");
                }
            }

            debug!("TelegramBotTrigger task completed");
        });

        Ok(task_handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;
    use std::sync::Mutex;

    lazy_static! {
        static ref ENV_LOCK: Mutex<()> = Mutex::new(());
    }

    #[test]
    fn test_builder_with_token() {
        let _lock = ENV_LOCK.lock().unwrap();
        let builder = TelegramBotTriggerBuilder::new().with_token("test_token");
        let trigger = builder.build();
        assert!(trigger.is_ok());
    }

    #[test]
    fn test_builder_with_env_var() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var("TELEGRAM_BOT_TOKEN", "test_token_from_env");
        }
        let builder = TelegramBotTriggerBuilder::new();
        let trigger = builder.build();
        assert!(trigger.is_ok());
        unsafe {
            std::env::remove_var("TELEGRAM_BOT_TOKEN");
        }
    }

    #[test]
    fn test_builder_fails_without_token() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::remove_var("TELEGRAM_BOT_TOKEN");
        }
        let builder = TelegramBotTriggerBuilder::new();
        let trigger_result = builder.build();
        assert!(trigger_result.is_err());
        match trigger_result {
            Err(TriggerError::ActivationError) => {
                // Correct error type
            }
            _ => panic!("Expected ActivationError"),
        }
    }
}
