// The `simple_file_writer` module provides a tool for writing content to a file.

use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use tokio::fs;
use tracing::info;
use uuid::Uuid;

/// The `FileWriterError` enum defines the possible errors that can occur within the `SimpleFileWriter`.
#[derive(Debug, thiserror::Error)]
pub enum FileWriterError {
    /// An error occurred while creating a directory.
    #[error("Failed to create directory: {0}")]
    DirectoryCreation(#[from] std::io::Error),
    /// An error occurred while writing to a file.
    #[error("File write error: {0}")]
    FileWrite(String),
}

/// A builder for [`SimpleFileWriter`].
pub struct SimpleFileWriterBuilder {
    output_dir: PathBuf,
}

impl SimpleFileWriterBuilder {
    /// Creates a new `SimpleFileWriterBuilder`.
    ///
    /// # Arguments
    ///
    /// * `output_dir` - The directory where the files will be written.
    pub fn new(output_dir: PathBuf) -> Self {
        Self { output_dir }
    }

    /// Builds a `SimpleFileWriter`.
    pub fn build(&self) -> SimpleFileWriter {
        SimpleFileWriter::new(self.output_dir.clone())
    }
}

/// The arguments for the `SimpleFileWriter` tool.
#[derive(serde::Deserialize)]
pub struct SFWArgs {
    /// The content to write to the file.
    content: String,
}

/// A tool that can write content to a file in a designated directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleFileWriter {
    /// The directory to write the file to.
    output_dir: PathBuf,
}

impl SimpleFileWriter {
    /// Creates a new `SimpleFileWriter`.
    pub fn new(output_dir: PathBuf) -> Self {
        Self { output_dir }
    }
}

impl Tool for SimpleFileWriter {
    const NAME: &'static str = "simple.file.writer";

    type Args = SFWArgs;
    type Error = std::io::Error;
    type Output = ();

    /// Returns the definition of the tool.
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

    /// Calls the tool to write the content to a file.
    async fn call(&self, params: Self::Args) -> Result<Self::Output, Self::Error> {
        fs::create_dir_all(&self.output_dir).await?;
        // Generate a unique filename and write the content
        let filename = format!("{}.txt", Uuid::new_v4());
        let file_path = self.output_dir.join(&filename);

        fs::write(&file_path, params.content).await?;

        let success_message = format!("Successfully wrote content to '{}'", file_path.display());
        info!(message = %success_message);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_simple_file_writer_tool() {
        // --- 1. Arrange ---
        let dir = tempdir().unwrap();
        let output_dir = dir.path().to_path_buf();
        let content = "This is a test.".to_string();

        // Use the builder to create the tool
        let writer = SimpleFileWriterBuilder::new(output_dir.clone()).build();
        let args = SFWArgs {
            content: content.clone(),
        };

        // --- 2. Act ---
        let result = writer.call(args).await;

        // --- 3. Assert ---
        assert!(result.is_ok());

        // Verify that the file was created with the correct content
        let mut entries = fs::read_dir(output_dir).await.unwrap();
        let entry = entries.next_entry().await.unwrap().unwrap();
        let file_content = fs::read_to_string(entry.path()).await.unwrap();
        assert_eq!(file_content, content);
    }
}
