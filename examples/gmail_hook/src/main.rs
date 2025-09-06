// This example demonstrates a more complex agent that watches for new emails, uses an LLM to summarize them, and then marks them as read.
//
// Key features:
// - Gmail API integration for watching new emails
// - LLM-powered email summarization and classification
// - Automatic retry logic for handling API rate limits (429 errors)
// - Daily summary generation and email management
//
// The RetryableLLM wrapper automatically handles rate limiting by:
// - Detecting 429 (rate limit) errors from the Gemini API
// - Implementing exponential backoff with jitter
// - Respecting retry delay hints from Google API responses
// - Only retrying on transient errors, not permanent failures

use dotenv;
use forgeflow::{
    agent::AgentBuilder,
    llm::decorators::RetryableLLM,
    shutdown,
    tools::{DailySummaryWriter, gmail_tool::GmailTool},
    triggers::GmailWatchTrigger,
    utils::google_auth::{GConf, GoogleAuthFlow, InnerConf},
};
use prompt_crafter::{Context, Instruction, OutputFormat, Persona, Prompt};
use rig::{
    client::CompletionClient, prelude::ProviderClient, providers::gemini::Client,
    providers::gemini::completion,
    providers::gemini::completion::gemini_api_types::AdditionalParameters,
    providers::gemini::completion::gemini_api_types::GenerationConfig,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tracing::{Level, error, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Initialize the logger.
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Create the trigger using GmailWatchTrigger::new
    let conf = GConf::from(Arc::new(InnerConf {
        credentials_path: Path::new("./tmp/credential.json").to_path_buf(),
        token_path: Path::new("./tmp/token.json").to_path_buf(),
        flow: GoogleAuthFlow::Redirect {
            port: None,
            open_browser: true,
        },
    }));
    let trigger = GmailWatchTrigger::new(conf.clone()).await.unwrap();
    info!("GmailWatchTrigger initialized");

    //Instatiante the right tool for the job
    let output_dir = PathBuf::from("./daily_summary");
    let summary_writer = DailySummaryWriter::new(output_dir);

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

    let system_prompt = Prompt::builder()
        .add(Context::new("I receive a lot of emails most of which are useless, and I spend a lot of time reading them. Your job is to help me saving time providing a valid summary of each email and a classification that helps me to go straight to the point. Email will be presented to you as low level technical API response so, you need to be:
            * able to read the body of an email even if it is presented as multipart/mixed
            * able to decode text that is encoded in base64
            * able to clearly identify if there are any relevant attachments
            * able to extract the subject, date, sender, and email ID from the email"))
        .add(Persona::new("You are a senior personal assistant, very detail oriented and attentive."))
        .add(Instruction::new("Your main task is to create a daily summary journal for all the emails, you mast structure it as specified in the output format section. Create a compelling summary of it and classify them according to the following categories:
            Critical: similar to Important but with huge impact on your life or work.
            Important: needs to be read quickly and acted upon
            Neutral: needs to be read, relevant information are in the mail, but no hurry. no urgent actions involved.
            Useless: are mail that can be easily ignored beacuse the relevant message is already in the summary you are saving.

            moreover, you must mark all the email classified as Useless as read"))
        .add(OutputFormat::new("must contain this field: Subject, Date, Summary, Sender (who is the real sender of the mail), Email ID, Calassification, Reason for the classification, use emoji appropriatly to highlight relevant information and expecially the classification, here an example:\n
            Subject 💁: Must-reads: Mortgage backfire, loyalty is dead, & more
            Date 📅: Thu, 28 Aug 2025 18:04:15 +0000
            Summary👌: This email is a newsletter from Business Insider, summarizing top reads of August, including articles on mortgage, workplace loyalty, and Microsoft pay.
            Sender🙋‍♀️: Business Insider <subscriptions@email.businessinsider.com>
            Email ID: 198f1da162ec9fd9
            Calassification😒: 😅Useless😅
            Reason for the classification: The email is a newsletter providing summaries of articles. The summary is enough to understand the main topics and the relevant information is already saved"))
        .build();

    let gemini_agent = gemini_client
        .agent(completion::GEMINI_2_0_FLASH_LITE)
        .preamble(&system_prompt.to_string())
        .temperature(0.9)
        .tool(summary_writer)
        .tool(gmail_actions)
        .additional_params(serde_json::to_value(cfg).unwrap())
        .build();

    // Wrap the Gemini agent with retry logic to handle rate limiting (429 errors)
    // This is crucial for production email processing where rate limits are common
    // The wrapper will:
    // - Automatically retry on 429 rate limit errors up to 3 times
    // - Use exponential backoff with jitter to avoid thundering herd issues
    // - Respect any retry delay hints provided by the Gemini API
    // - Immediately fail on permanent errors (4xx/5xx except 429)
    let retryable_gemini_agent = RetryableLLM::new(gemini_agent, 3);
    info!("Created RetryableLLM wrapper with 3 retry attempts for rate limiting");

    // Create the agent.
    let agent_result = AgentBuilder::new()
        .add_trigger(Box::new(trigger))
        .with_shutdown_handler(shutdown::CtrlCShutdown::new())
        .with_model(Box::new(retryable_gemini_agent))
        .with_prompt_template(
            "This is a {{name}}:\nthis message id is {{payload.id}}, use it for acting on the specific email.\n receiveing data {{verbatim payload.payload.headers}}\n content in parts {{verbatim payload.payload.parts}}"
                .to_string(),
        )
        .build();

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
    tracing::info!("Agent run completed");
}
