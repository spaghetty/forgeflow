# Project Overview

ForgeFlow is a Rust-based framework for building and running autonomous agents that can interact with various services and perform complex tasks. It provides a flexible and extensible architecture for creating sophisticated agentic workflows.

## Project Philosophy

The philosophy behind ForgeFlow is inspired by workflow automation tools like n8n, but with a focus on a code-first, agentic approach within the Rust ecosystem. The goal is to provide a simple yet powerful framework where developers can declaratively define complex workflows.

The core workflow is straightforward:
1.  **Set a Trigger:** Choose an event source to kick off the agent (e.g., a new email, a timed interval).
2.  **Equip an Agent:** Provide the agent with a set of tools (capabilities) and a configured LLM.
3.  **Run:** Let the agent execute its logic based on the trigger's input.

## Core Concepts

The framework is built around a few core concepts:

*   **Agent:** The central component of the framework. An agent is responsible for coordinating the other components and executing the main logic.
*   **Triggers:** Triggers are responsible for initiating agent actions. They can be based on a schedule (e.g., `PollTrigger`) or external events (e.g., `GmailWatchTrigger`).
*   **LLM:** The `LLM` trait provides an abstraction for interacting with language models. The current implementation uses the `rig` crate to interact with Google's Gemini models.
*   **Tools:** Tools are used by the agent to perform actions. The framework provides a `SimpleFileWriter` tool for writing files and a `GmailTool` for interacting with the Gmail API.
*   **Shutdown:** The framework provides a graceful shutdown mechanism that can be triggered by a `Ctrl-C` signal or a time-based shutdown.

## Event Structure and Templating

ForgeFlow uses a templating system to allow agents to dynamically construct prompts based on the data received from triggers. This is particularly useful for handling complex event structures, such as the `NewEmail` event from the `GmailWatchTrigger`.

### Gmail `NewEmail` Event Structure

For more information on the structure of the `NewEmail` event, please refer to the [Gmail `NewEmail` Event documentation](./events/gmail_event.md).

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
 receiveing data [{"name":"Subject","value":"Your Subject Here"},...]\n content in parts [{"mimeType":"text/plain","body":{"data":"..."}},...]
```

This allows you to create highly customized prompts that provide the LLM with the precise information it needs to perform its task.
