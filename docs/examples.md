# Examples

This document provides a guide to understanding and using the examples provided with the Forgeflow framework.

## Haiku Generator

This example demonstrates a simple agent that generates haikus and saves them to a file. It uses a `PollTrigger` to fire an event every 12 seconds, and a `SimpleFileWriter` tool to write the generated haikus to the `./haikus` directory.

### Running the Example

```bash
cargo run --example haiku_generator
```

## Gmail Hook

This example demonstrates a more complex agent that watches for new emails, uses an LLM to summarize them, and then marks them as read. It uses a `GmailWatchTrigger` to watch for new unread emails, a `GmailTool` to mark emails as read, and a `SimpleFileWriter` to save the summaries to the `./daily_summary` directory.

### Running the Example

```bash
cargo run --example gmail_hook
```
