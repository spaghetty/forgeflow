// This example demonstrates a Telegram bot agent that receives messages and logs them to files.
//
// Key features:
// - Telegram bot integration for receiving messages
// - LLM-powered processing of messages
// - File logging using SimpleFileWriter tool
// - Automatic retry logic for reliability

use forgeflow::{
    agent::AgentBuilder, shutdown, SimpleFileWriterBuilder, TelegramBotTriggerBuilder,
};
use rig::{
    client::CompletionClient, 
    prelude::ProviderClient, 
    providers::gemini::Client,
    providers::gemini::completion,
    providers::gemini::completion::gemini_api_types::{AdditionalParameters, GenerationConfig},
};
use std::path::PathBuf;
use tracing::{Level, error, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    
    // Initialize the logger
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    info!("Starting Telegram Agent");

    // Create the Telegram trigger using the builder
    let trigger = match TelegramBotTriggerBuilder::new().build() {
        Ok(trigger) => trigger,
        Err(e) => {
            error!("Failed to create Telegram trigger: {}", e);
            error!("Make sure TELEGRAM_BOT_TOKEN is set in your environment");
            return;
        }
    };
    info!("TelegramBotTrigger initialized");

    // Create the file writer tool for logging messages
    let output_dir = PathBuf::from("./telegram_logs");
    let file_writer = SimpleFileWriterBuilder::new(output_dir).build();

    // Create and configure the Gemini client
    let gemini_client = Client::from_env();
    let gen_cfg = GenerationConfig {
        top_k: Some(1),
        top_p: Some(0.95),
        candidate_count: Some(1),
        ..Default::default()
    };
    let cfg = AdditionalParameters::default().with_config(gen_cfg);

    // Create the Gemini agent with the file writer tool
    let gemini_agent = gemini_client
        .agent(completion::GEMINI_2_0_FLASH_LITE)
        .preamble("You are a helpful assistant that logs Telegram messages to files. When you receive a message, write it to a file using the file writer tool.")
        .temperature(0.7)
        .tool(file_writer)
        .additional_params(serde_json::to_value(cfg).unwrap())
        .build();

    info!("Created Gemini agent with file writing capability");

    // Create the ForgeFlow agent
    let agent_result = AgentBuilder::new()
        .add_trigger(Box::new(trigger))
        .with_shutdown_handler(shutdown::CtrlCShutdown::new())
        .with_model(Box::new(gemini_agent))  // Retry will be added automatically by default
        .with_prompt_template(
            "You received a Telegram message:\n\
            Message ID: {{payload.message_id}}\n\
            Chat ID: {{payload.chat_id}}\n\
            From: {{payload.first_name}} (@{{payload.username}})\n\
            Text: {{payload.text}}\n\
            Date: {{payload.date}}\n\n\
            Please log this message to a file using the file writer tool."
                .to_string(),
        )
        .build();

    // Run the agent
    match agent_result {
        Ok(agent) => {
            info!("Agent started successfully. Send messages to your Telegram bot!");
            if let Err(e) = agent.run().await {
                error!(error = %e, "Agent execution failed");
            }
        }
        Err(e) => {
            error!(error = %e, "Failed to build agent");
        }
    }
    
    info!("Telegram Agent finished");
}
