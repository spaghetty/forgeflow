# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## ForgeFlow: Agentic Workflow Engine

ForgeFlow is a Rust-based framework for building and running autonomous agents that can interact with various services and perform complex tasks using LLMs.

## Development Commands

### Building and Testing
```bash
# Build the library
cargo build

# Build with optimizations
cargo build --release

# Check code without building
cargo check

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture
```

### Running Examples
```bash
# Run the haiku generator example (uses PollTrigger)
cargo run --example haiku_generator

# Run the Gmail hook example (requires Gmail API setup)
cargo run --example gmail_hook
```

### Code Quality
```bash
# Format code
cargo fmt

# Run clippy for linting
cargo clippy

# Run clippy with all features
cargo clippy --all-features
```

## Environment Setup

### For Gmail Examples
Set up Gmail API credentials:
1. Place `credentials.json` in `./tmp/`
2. Create `token.json` in `./tmp/`
3. Set `GEMINI_API_KEY` environment variable:
   ```bash
   export GEMINI_API_KEY='your_api_key'
   ```

## Core Architecture

### Agent-Centric Design
ForgeFlow follows an event-driven architecture centered around **Agents** that coordinate:
- **Triggers**: Event sources that initiate workflows
- **LLMs**: Language models for processing and decision-making  
- **Tools**: Capabilities for external interactions
- **Events**: Structured data flow between components

### Key Components

#### Agent (`src/agent.rs`)
The central coordinator that:
- Manages multiple triggers simultaneously
- Processes events through an event loop
- Renders prompts using Handlebars templates
- Handles graceful shutdown with inflight request tracking
- Built using `AgentBuilder` pattern

#### Triggers (`src/triggers/`)
Event sources implementing the `Trigger` trait:
- `PollTrigger`: Time-based periodic events
- `GmailWatchTrigger`: Gmail API webhook monitoring
- All triggers run as async tasks with shutdown coordination

#### Tools (`src/tools/`)
Action capabilities for agents:
- `SimpleFileWriter`: File system operations
- `GmailTool`: Gmail API interactions
- `DailySummaryWriter`: Specialized file writing

#### LLM Integration (`src/llm/`)
Language model abstraction with:
- Trait-based design for multiple LLM providers
- Built-in retry logic for reliability
- Configured for Google Gemini via `rig` crate

### Event Flow
1. Triggers detect events and emit `TEvent` structures
2. Agent receives events via mpsc channels
3. Events are serialized to JSON for template rendering
4. Handlebars templates generate LLM prompts
5. LLM responses drive tool execution decisions

### Templating System
Uses Handlebars with custom helpers:
- `{{name}}` - Access event fields
- `{{payload.field}}` - Nested JSON access
- `{{verbatim object}}` - JSON serialization
- Templates defined at agent build time

### Workspace Structure
- Root crate: Core ForgeFlow library
- `examples/`: Separate workspace members for demonstrations
- `docs/`: Comprehensive documentation
- `src/`: Library implementation with modular design

### Concurrency Model
- Async/await throughout with Tokio runtime
- Broadcast channels for shutdown coordination
- MPSC channels for event distribution
- Atomic counters for inflight request tracking
- Graceful shutdown with configurable handlers

### Testing Strategy
Run individual module tests:
```bash
# Test specific module
cargo test agent::tests

# Test with specific trigger
cargo test triggers::poll_trigger::tests
```
