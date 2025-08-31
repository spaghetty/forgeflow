# Tools

Tools are used by the agent to perform actions. They are defined by the `Tool` trait, which has two methods: `definition` and `call`.

*   **`definition`:** This method returns a `ToolDefinition` struct that describes the tool, including its name, description, and parameters.
*   **`call`:** This method is called by the agent to execute the tool. It takes a set of arguments and returns a result.

## `SimpleFileWriter`

This tool writes content to a file in a designated directory.

### Example

```rust
use forgeflow::tools::SimpleFileWriter;
use std::path::PathBuf;

let output_dir = PathBuf::from("./haikus");
let file_writer_actuator = SimpleFileWriter::new(output_dir);
```

## `GmailTool`

This tool marks an email as read in Gmail.

### Example

```rust
use forgeflow::tools::gmail_actions::GmailTool;
use forgeflow::utils::google_auth::GConf;
use std::path::Path;

let conf = GConf::new(
    Path::new("./tmp/credential.json").to_path_buf(),
    Path::new("./tmp/token.json").to_path_buf(),
);
let gmail_actions = GmailTool::new(conf.clone());
```

## `DailySummaryWriter`

This tool writes content to a file in a designated directory. The file will be named with the current date (e.g., YYYY-MM-DD.txt). If the file for the current date doesn't exist, it will be created. If the file already exists, the content will be appended to it. Each entry will be separated by a line of `============`.

### Example

```rust
use forgeflow::tools::DailySummaryWriter;
use std::path::PathBuf;

let output_dir = PathBuf::from("./daily_summary");
let summary_writer = DailySummaryWriter::new(output_dir);
```