# ForgeFlow: Agentic Workflow Engine

ForgeFlow is a Rust-based framework for building and running autonomous agents that can interact with various services and perform complex tasks. It provides a flexible and extensible architecture for creating sophisticated agentic workflows.

## Key Concepts

- **Agents:** The core of ForgeFlow, agents are responsible for executing tasks based on a given prompt and context. They leverage large language models (LLMs) to understand and respond to triggers.

- **Triggers:** Events that initiate an agent's workflow. ForgeFlow supports various trigger types, including:
    - `PollTrigger`: Fires at a regular interval.
    - `GmailWatchTrigger`: Monitors a Gmail inbox for new messages.

- **Tools:** Capabilities that agents can use to interact with the outside world. Examples include:
    - `SimpleFileWriter`: Writes content to a file.
    - `GmailTool`: Interacts with the Gmail API to perform actions like marking messages as read.

## Project Philosophy

The philosophy behind ForgeFlow is inspired by workflow automation tools like n8n, but with a focus on a code-first, agentic approach within the Rust ecosystem. The goal is to provide a simple yet powerful framework where developers can declaratively define complex workflows.

The core workflow is straightforward:
1.  **Set a Trigger:** Choose an event source to kick off the agent (e.g., a new email, a timed interval).
2.  **Equip an Agent:** Provide the agent with a set of tools (capabilities) and a configured LLM.
3.  **Run:** Let the agent execute its logic based on the trigger's input.

### Future Roadmap

The project is currently in an experimental phase, with a focus on expanding its capabilities. The next steps include:

-   **More Triggers:** Adding a wider variety of built-in triggers to connect to different event sources (e.g., webhooks, message queues).
-   **More Tools:** Growing the collection of pre-built tools to give agents more out-of-the-box capabilities.
-   **LLM Memory:** Implementing a memory component to allow agents to retain context and learn from past interactions, enabling more stateful and intelligent behavior.

## Event Structure and Templating

ForgeFlow uses a templating system to allow agents to dynamically construct prompts based on the data received from triggers. This is particularly useful for handling complex event structures, such as the `NewEmail` event from the `GmailWatchTrigger`.

### Gmail `NewEmail` Event Structure

When a new email is received, the `GmailWatchTrigger` fires a `NewEmail` event with a payload that contains the full email object from the Gmail API. Here is a simplified and anonymized example of the JSON structure:

```json
{
  "name": "NewEmail",
  "payload": {
    "id": "19866324d5dd1cad",
    "payload": {
      "headers": [
        { "name": "Subject", "value": "Your Subject Here" },
        { "name": "From", "value": "sender@example.com" },
        { "name": "Date", "value": "Fri, 01 Aug 2025 15:13:48 +0000" }
      ],
      "parts": [
        {
          "mimeType": "text/plain",
          "body": {
            "data": "VGhpcyBpcyB0aGUgYm9keSBvZiB0aGUgZW1haWwu"
          }
        },
        {
          "mimeType": "application/pdf",
          "filename": "document.pdf",
          "body": {
            "attachmentId": "..."
          }
        }
      ]
    }
  }
}
```

### Templating

ForgeFlow uses a simple templating syntax with `{{` and `}}` to denote placeholders. You can access nested fields in the JSON payload using dot notation. The `verbatim` helper is used to serialize a JSON object or array into a string.

Here is an example of a prompt template that extracts the email's subject, headers, and body parts:

```rust
.with_prompt_template(
    "This is a {{name}}:\nthis message id is {{payload.id}}, yous it for acting on the specific email.\n receiveing data {{verbatim payload.payload.headers}}\n content in parts {{verbatim payload.payload.parts}}".to_string(),
)
```

This template will be rendered as follows:

```
This is a NewEmail:
this message id is 19866324d5dd1cad, yous it for acting on the specific email.
 receiveing data [{"name":"Subject","value":"Your Subject Here"},...]
 content in parts [{"mimeType":"text/plain","body":{"data":"..."}},...]
```

This allows you to create highly customized prompts that provide the LLM with the precise information it needs to perform its task.


## LLM Configuration


The examples are configured to use Google's Gemini LLM through the `rig` library. You can easily adapt the code to use any other LLM supported by `rig` by changing the client initialization in the example's `main.rs` file.

To run the examples with Gemini, you must set the `GEMINI_API_KEY` environment variable:

```bash
export GEMINI_API_KEY='your_api_key'
```

## Examples

### Haiku Generator

This example demonstrates a simple agent that generates haikus on a given topic and saves them to a file. It uses a `PollTrigger` to run periodically.

**To run the Haiku Generator example:**

```bash
cargo run --example haiku_generator
```

### Gmail Hook

This example showcases a more complex agent that monitors a Gmail inbox, classifies emails, and takes action based on the classification. It uses the `GmailWatchTrigger` to receive notifications for new emails and the `GmailTool` to mark messages as read.

**To run the Gmail Hook example:**

1.  **Enable the Gmail API:** Follow the instructions in the [Google Cloud documentation](https://developers.google.com/gmail/api/quickstart/go) to enable the Gmail API and download your `credentials.json` file.

2.  **Set up your environment:**
    - Place your `credentials.json` file in the `./tmp` directory.
    - Create a `token.json` file in the `./tmp` directory. This file will be used to store your OAuth2 token.

3.  **Run the example:**

```bash
cargo run --example gmail_hook
```

## Getting Started

To get started with ForgeFlow, you'll need to have Rust and Cargo installed. You can then create a new project and add `forgeflow` as a dependency in your `Cargo.toml` file.

```toml
[dependencies]
forgeflow = { git = "https://github.com/spaghetty/forgeflow.git" }
```

## Project Structure

- `src/`: Contains the core ForgeFlow library code.
    - `agent.rs`: Defines the `Agent` struct and its associated methods.
    - `triggers/`: Contains the various trigger implementations.
    - `tools/`: Contains the tool implementations.
    - `utils/`: Provides utility functions, such as Google authentication.
- `examples/`: Contains example projects that demonstrate how to use ForgeFlow.
