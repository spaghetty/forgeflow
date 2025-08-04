use forgeflow::tools::SimpleFileWriter;
use forgeflow::{agent::Agent, shutdown, triggers::GConf, triggers::GmailWatchTrigger};
use rig::providers::gemini::Client;
use rig::{prelude::ProviderClient, providers::gemini::completion::GEMINI_2_5_FLASH_PREVIEW_05_20};
use std::path::{Path, PathBuf};
//use std::time::Duration;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    //Create the trigger using GmailWatchTrigger::new
    let conf = GConf {
        credentials_path: Path::new("./tmp/credential.json").to_path_buf(),
        token_path: Path::new("./tmp/token.json").to_path_buf(),
    };
    let trigger = GmailWatchTrigger::new(conf).await.unwrap();
    info!("GmailWatchTrigger initialized");
    //let _ = trigger.auth().await;
    info!("GmailWatchTrigger authenticated");

    //Instatiante the right tool for the job
    let output_dir = PathBuf::from("./daily_summary");
    let file_writer_actuator = SimpleFileWriter::new(output_dir);
    //Instantiate the rigth model
    let gemini_client = Client::from_env();

    let gemini_agent = gemini_client
        .agent(GEMINI_2_5_FLASH_PREVIEW_05_20)
        .preamble("You are senior assistant and an expert in the technical email format,
            you know that body of an email is formed by multiple parts, each part can be text, attachment or different format of the same body. textual part of the email is encoded in base64 and you need to check all the body parts in a raw email.
            Your main task is to review emails, for me a help me saving time; you need to classify them into useless, important or neutral, consider that:
            importants: needs to be read quickly and acted upon
            neutral: needs to be read, relevant information in the mail, but no hurry. no urgent actions involved.
            useless: are mail that can be easyly ignored beacuse the relevant message is already in the summary you are saving.
            Than you can write the final output in a file.
            MANDATORY STRUCTURE OF THE SUMMARY: Subject, Date, Summary, Sender (who is), Email ID, Calassification, Reason for the classification")
        .temperature(0.9)
        .tool(file_writer_actuator)
        .build();

    let main_agent = Agent::new()
        .unwrap()
        .with_trigger(Box::new(trigger))
        .with_shutdown_handler(shutdown::CtrlCShutdown::new())
        .with_model(Box::new(gemini_agent))
        .with_prompt_template(
            "This is a {{name}}:
 receiveing data {{verbatim payload.payload.headers}}
 content in parts {{verbatimpayload.payload.parts}}"
                .to_string(),
        )
        .unwrap();

    let _ = main_agent.run().await;
}
