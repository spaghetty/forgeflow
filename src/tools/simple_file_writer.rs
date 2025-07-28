use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use tokio::fs;
use tracing::info;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("Failed to create directory: {0}")]
    DirectoryCreation(#[from] std::io::Error),
    #[error("File write error: {0}")]
    FileWrite(String),
}

#[derive(serde::Deserialize)]
pub struct SFWArgs {
    content: String,
}

/// A tool that can write content to a file in a designated directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleFileWriter {
    output_dir: PathBuf,
}

impl SimpleFileWriter {
    pub fn new(output_dir: PathBuf) -> Self {
        Self { output_dir }
    }
}

impl Tool for SimpleFileWriter {
    const NAME: &'static str = "simple.file.writer";

    type Args = SFWArgs;
    type Error = std::io::Error;
    type Output = ();

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description:
                "Writes a given content to a new file with a unique name in a secure directory."
                    .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "The content you want to write to the file."
                    }
                },
                "required": ["content"]
            }),
        }
    }

    async fn call(&self, params: Self::Args) -> Result<Self::Output, Self::Error> {
        fs::create_dir_all(&self.output_dir).await?;
        // Generate a unique filename and write the content
        let filename = format!("{}.txt", Uuid::new_v4());
        let file_path = self.output_dir.join(&filename);

        let result = fs::write(&file_path, params.content).await?;

        let success_message = format!("Successfully wrote content to '{}'", file_path.display());
        info!(message = %success_message);
        Ok(result)
    }
}
