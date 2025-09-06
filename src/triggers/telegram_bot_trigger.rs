// The `telegram_bot_trigger` module provides a trigger that listens for Telegram messages.

use crate::triggers::{Trigger, TriggerError, event::TEvent};
use async_trait::async_trait;
use serde_json::json;
use std::env;
use teloxide::{Bot, prelude::*, types::Update};
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

/// A trigger that listens for incoming Telegram messages.
#[derive(Clone)]
pub struct TelegramBotTrigger {
    bot: Bot,
}

impl TelegramBotTrigger {
    /// Creates a new `TelegramBotTrigger`.
    ///
    /// Reads the bot token from the `TELEGRAM_BOT_TOKEN` environment variable.
    pub fn new() -> Result<Self, TriggerError> {
        let token = env::var("TELEGRAM_BOT_TOKEN").map_err(|_| TriggerError::ActivationError)?;

        let bot = Bot::new(token);

        Ok(Self { bot })
    }
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

            // Create a simple handler function for messages
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

            // Use teloxide's standard dispatcher pattern with shutdown handling
            let mut dispatcher = Dispatcher::builder(
                bot,
                Update::filter_message().endpoint(move |bot, msg| {
                    let tx_clone = tx.clone();
                    handler(bot, msg, tx_clone)
                }),
            )
            .build();

            // Run dispatcher with shutdown signal
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
