//use forgeflow::tools::SimpleFileWriter;
use forgeflow::{agent::Agent, shutdown, triggers::GConf, triggers::GmailWatchTrigger};
//use rig::providers::gemini::Client;
//use rig::{prelude::ProviderClient, providers::gemini::completion::GEMINI_2_5_FLASH_PREVIEW_05_20};
use std::path::{Path, PathBuf};
//use std::time::Duration;
use tracing::{Level, error, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let conf = GConf {
        credentials_path: Path::new("./tmp/credential.json").to_path_buf(),
        token_path: Path::new("./tmp/token.json").to_path_buf(),
    };
    let mut trigger = GmailWatchTrigger::new("INBOX", conf).await.unwrap();

    let _ = trigger.auth().await;

    let main_agent = Agent::new().unwrap().with_trigger(Box::new(trigger));

    let _ = main_agent.run().await;

    //info!("Starting Forgeflow Example");
    //let output_dir = PathBuf::from("./haikus");
    //let file_writer_actuator = SimpleFileWriter::new(output_dir);

    // 2. Use the specific provider's builder to configure the model
    //let gemini_client = Client::from_env();

    //let gemini_agent = gemini_client
    //    .agent(GEMINI_2_5_FLASH_PREVIEW_05_20)
    //    .preamble("You are a very expert haiku writer, you will write all the haiku you generate to a file")
    //    .temperature(0.9)
    //    .tool(file_writer_actuator)
    //    .build();

    // 4. Configure the agent with the generic provider
    //let agent_result = Agent::new()
    //    .map(|agent| agent.with_model(Box::new(gemini_agent)))
    //    .and_then(|agent| {
    //        agent.with_prompt_template(
    //            "Write a haiku about the following topic: {{name}}".to_string(),
    //        )
    //    })
    //    .map(|agent| {
    //        agent
    //            .with_trigger(PollTrigger::new(
    //                "The Rust Programming Language",
    //                Duration::from_secs(12),
    //                true,
    //            ))
    //            .with_shutdown_handler(shutdown::TimeBasedShutdown::new(Duration::from_secs(10)))
    //    });

    //match agent_result {
    //    Ok(agent) => {
    //        if let Err(e) = agent.run().await {
    //            error!(error = %e, "Agent execution failed");
    //        }
    //    }
    //    Err(e) => {
    //        error!(error = %e, "Failed to build agent");
    //    }
    //}
    //info!("Forgeflow Example Finished");
}
