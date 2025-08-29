// This example demonstrates a simple agent that generates haikus and saves them to a file.

use forgeflow::{agent::Agent, shutdown, tools::SimpleFileWriter, triggers::PollTrigger};
use rig::{
    prelude::ProviderClient, providers::gemini::Client,
    providers::gemini::completion::GEMINI_2_5_FLASH_PREVIEW_05_20,
};
use std::path::PathBuf;
use std::time::Duration;
use tracing::{Level, error, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    // Initialize the logger.
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    info!("Starting Forgeflow Example");

    // Create a tool to write the generated haikus to a file.
    let output_dir = PathBuf::from("./haikus");
    let file_writer_actuator = SimpleFileWriter::new(output_dir);

    // Create a new Gemini client.
    let gemini_client = Client::from_env();

    // Create a new Gemini agent.
    let gemini_agent = gemini_client
        .agent(GEMINI_2_5_FLASH_PREVIEW_05_20)
        .preamble("You are a very expert haiku writer, you will write all the haiku you generate to a file")
        .temperature(0.9)
        .tool(file_writer_actuator)
        .build();

    // Create the agent.
    let agent_result = Agent::new()
        .map(|agent| agent.with_model(Box::new(gemini_agent)))
        .and_then(|agent| {
            agent.with_prompt_template(
                "Write a haiku about the following topic: {{name}}".to_string(),
            )
        })
        .map(|agent| {
            agent
                .add_trigger(PollTrigger::new(
                    "The Rust Programming Language",
                    Duration::from_secs(12),
                    true,
                ))
                .with_shutdown_handler(shutdown::TimeBasedShutdown::new(Duration::from_secs(10)))
        });

    // Run the agent.
    match agent_result {
        Ok(agent) => {
            if let Err(e) = agent.run().await {
                error!(error = %e, "Agent execution failed");
            }
        }
        Err(e) => {
            error!(error = %e, "Failed to build agent");
        }
    }

    info!("Forgeflow Example Finished");
}
