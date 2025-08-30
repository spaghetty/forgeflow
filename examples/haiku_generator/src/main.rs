// This example demonstrates a simple agent that generates haikus and saves them to a file.

use forgeflow::{agent::Agent, shutdown, tools::SimpleFileWriter, triggers::PollTrigger};
use rig::{
    client::CompletionClient,
    prelude::ProviderClient,
    providers::gemini::{
        Client, completion, completion::gemini_api_types::AdditionalParameters,
        completion::gemini_api_types::GenerationConfig,
    },
};
use serde_json;
use std::path::PathBuf;
use std::time::Duration;
use tracing::{Level, error, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    // Initialize the logger.
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    info!("Starting Forgeflow Example");

    // Create a tool to write the generated haikus to a file.
    let output_dir = PathBuf::from("./haikus");
    let file_writer_actuator = SimpleFileWriter::new(output_dir);

    // Create a new Gemini client.
    let gemini_client = Client::from_env();
    let gen_cfg = GenerationConfig {
        top_k: Some(1),
        top_p: Some(0.95),
        candidate_count: Some(1),
        ..Default::default()
    };
    let cfg = AdditionalParameters::default().with_config(gen_cfg);

    // Create a new Gemini agent.
    let gemini_agent = gemini_client
        .agent(completion::GEMINI_2_0_FLASH_LITE)
        .preamble(
            "You are an expert haiku writer, you will write all the haiku you generate to a file",
        )
        .temperature(0.9)
        .tool(file_writer_actuator)
        .additional_params(serde_json::to_value(cfg).unwrap())
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
                .with_shutdown_handler(shutdown::TimeBasedShutdown::new(Duration::from_secs(20)))
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
