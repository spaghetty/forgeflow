use chrono::Local;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

#[allow(dead_code)]
const LINE: &str = "============";

#[derive(Debug, thiserror::Error)]
pub enum DailySummaryWriterError {
    #[error("Failed to create directory: {0}")]
    DirectoryCreation(#[from] std::io::Error),
    #[error("File write error: {0}")]
    FileWrite(String),
}

/// A builder for [`DailySummaryWriter`].
pub struct DailySummaryWriterBuilder {
    output_dir: PathBuf,
}

impl DailySummaryWriterBuilder {
    /// Creates a new `DailySummaryWriterBuilder`.
    ///
    /// # Arguments
    ///
    /// * `output_dir` - The directory where the daily summary files will be stored.
    pub fn new(output_dir: PathBuf) -> Self {
        Self { output_dir }
    }

    /// Builds a `DailySummaryWriter`.
    pub fn build(&self) -> DailySummaryWriter {
        DailySummaryWriter::new(self.output_dir.clone())
    }
}

#[derive(Deserialize)]
pub struct DSWArgs {
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailySummaryWriter {
    output_dir: PathBuf,
}

impl DailySummaryWriter {
    pub fn new(output_dir: PathBuf) -> Self {
        Self { output_dir }
    }
}

impl Tool for DailySummaryWriter {
    const NAME: &'static str = "daily.summary.writer";

    type Args = DSWArgs;
    type Error = std::io::Error;
    type Output = ();

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Adds entries to a daily summary journal.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "The content to write to the summary journal."
                    }
                },
                "required": ["content"]
            }),
        }
    }

    async fn call(&self, params: Self::Args) -> Result<Self::Output, Self::Error> {
        let date = Local::now().format("%Y-%m-%d").to_string();
        let file_name = format!("{date}.txt");
        let file_path = self.output_dir.join(file_name);

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
            .await?;

        file.write_all(format!("\n{LINE}\n").as_bytes()).await?;
        file.write_all(params.content.as_bytes()).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs;

    #[tokio::test]
    async fn test_write_to_new_file() {
        let dir = tempdir().unwrap();
        let writer = DailySummaryWriterBuilder::new(dir.path().to_path_buf()).build();

        let args = DSWArgs {
            content: "This is the first summary.".to_string(),
        };

        writer.call(args).await.unwrap();

        let date = Local::now().format("%Y-%m-%d").to_string();
        let file_name = format!("{date}.txt");
        let file_path = dir.path().join(file_name);

        let content = fs::read_to_string(file_path).await.unwrap();
        assert!(content.contains(LINE));
        assert!(content.contains("This is the first summary."));
    }

    #[tokio::test]
    async fn test_append_to_existing_file() {
        let dir = tempdir().unwrap();
        let writer = DailySummaryWriterBuilder::new(dir.path().to_path_buf()).build();

        let args1 = DSWArgs {
            content: "This is the first summary.".to_string(),
        };
        writer.call(args1).await.unwrap();

        let args2 = DSWArgs {
            content: "This is the second summary.".to_string(),
        };
        writer.call(args2).await.unwrap();

        let date = Local::now().format("%Y-%m-%d").to_string();
        let file_name = format!("{date}.txt");
        let file_path = dir.path().join(file_name);

        let content = fs::read_to_string(file_path).await.unwrap();
        let occurrences = content.matches(LINE).count();
        assert_eq!(occurrences, 2);
        assert!(content.contains("This is the first summary."));
        assert!(content.contains("This is the second summary."));
    }
}
