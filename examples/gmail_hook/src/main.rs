// This example demonstrates a more complex agent that watches for new emails, uses an LLM to summarize them, and then marks them as read.

use forgeflow::{
    agent::Agent,
    shutdown,
    tools::{SimpleFileWriter, gmail_actions::GmailTool},
    triggers::GmailWatchTrigger,
    utils::google_auth::GConf,
};
use rig::{
    client::CompletionClient, prelude::ProviderClient, providers::gemini::Client,
    providers::gemini::completion,
    providers::gemini::completion::gemini_api_types::AdditionalParameters,
    providers::gemini::completion::gemini_api_types::GenerationConfig,
};
use std::path::{Path, PathBuf};

use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    // Initialize the logger.
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Create the trigger using GmailWatchTrigger::new
    let conf = GConf::new(
        Path::new("./tmp/credential.json").to_path_buf(),
        Path::new("./tmp/token.json").to_path_buf(),
    );
    let trigger = GmailWatchTrigger::new(conf.clone()).await.unwrap();
    info!("GmailWatchTrigger initialized");

    //Instatiante the right tool for the job
    let output_dir = PathBuf::from("./daily_summary");
    let file_writer_actuator = SimpleFileWriter::new(output_dir);

    let gmail_actions = GmailTool::new(conf.clone());
    //Instantiate the rigth model
    let gemini_client = Client::from_env();

    let gen_cfg = GenerationConfig {
        top_k: Some(1),
        top_p: Some(0.95),
        candidate_count: Some(1),
        ..Default::default()
    };
    let cfg = AdditionalParameters::default().with_config(gen_cfg);

    let gemini_agent = gemini_client
        .agent(completion::GEMINI_2_0_FLASH_LITE)
        .preamble("You are senior assistant and an expert in the technical email format,
            you know that body of an email is formed by multiple parts, each part can be text, attachment or different format of the same body. textual part of the email is encoded in base64 and you need to check all the body parts in a raw email.
            Your main task is to review emails, for me a help me saving time; you need to classify them into useless, important or neutral, consider that:
            importants: needs to be read quickly and acted upon
            neutral: needs to be read, relevant information in the mail, but no hurry. no urgent actions involved.
            useless: are mail that can be easyly ignored beacuse the relevant message is already in the summary you are saving.
            if the message is classifide as useless mark it as read in my inbox and than you can write the final output in a file.
            MANDATORY STRUCTURE OF THE SUMMARY: Subject, Date, Summary, Sender (who is), Email ID, Calassification, Reason for the classification")
        .temperature(0.9)
        .tool(file_writer_actuator)
        .tool(gmail_actions)
        .additional_params(serde_json::to_value(cfg).unwrap())
        .build();

    // Create the agent.
    let main_agent = Agent::new()
        .unwrap()
        .add_trigger(Box::new(trigger))
        .with_shutdown_handler(shutdown::CtrlCShutdown::new())
        .with_model(Box::new(gemini_agent))
        .with_prompt_template(
            "This is a {{name}}:
this message id is {{payload.id}}, yous it for acting on the specific email.
 receiveing data {{verbatim payload.payload.headers}}
 content in parts {{verbatim payload.payload.parts}}"
                .to_string(),
        )
        .unwrap();

    // Run the agent.
    let _ = main_agent.run().await;
    tracing::info!("Agent run completed");
}
