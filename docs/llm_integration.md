# LLM Integration

The `LLM` trait provides an abstraction for interacting with language models. This allows the agent to be used with any language model that implements the trait.

The current implementation uses the `rig` crate to interact with Google's Gemini models. The `RigAgent` struct is an adapter that implements the `LLM` trait and uses the `rig` crate to send prompts to the Gemini API.

## Example

```rust
use rig::providers::gemini::Client;
use rig::{prelude::ProviderClient, providers::gemini::completion::GEMINI_2_5_FLASH_PREVIEW_05_20};

let gemini_client = Client::from_env();

let gemini_agent = gemini_client
    .agent(GEMINI_2_5_FLASH_PREVIEW_05_20)
    .preamble("You are a very expert haiku writer, you will write all the haiku you generate to a file")
    .temperature(0.9)
    .tool(file_writer_actuator)
    .build();
```
