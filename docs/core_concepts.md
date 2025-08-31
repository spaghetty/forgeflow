# Core Concepts

This document provides a more detailed explanation of the core concepts in the Forgeflow framework.

## Agent

The `Agent` is the heart of the framework. It is responsible for:

*   **Managing triggers:** The agent can be configured with multiple triggers, which it will launch and manage.
*   **Handling events:** The agent receives events from the triggers and processes them.
*   **Interacting with LLMs:** The agent can be configured with a language model to process events and generate responses.
*   **Using tools:** The agent can be equipped with tools to perform actions based on the LLM's responses.
*   **Graceful shutdown:** The agent can be shut down gracefully using a shutdown handler.

To create a new agent, you use the `AgentBuilder::new()` method. You can then chain methods to configure the agent with triggers, a model, a prompt template, and a shutdown handler, and finally call the `build()` method to create the agent.

## Triggers

Triggers are responsible for initiating agent actions. They are defined by the `Trigger` trait, which has a single method: `launch`. This method launches a long-running task that will send events to the agent.

To create a new trigger, you need to implement the `Trigger` trait for your struct. The `launch` method should contain the logic for listening for events and sending them to the agent through the provided channel.

## LLM

The `LLM` trait provides an abstraction for interacting with language models. This allows the agent to be used with any language model that implements the trait.

To add support for a new language model, you need to create a new struct that implements the `LLM` trait. The `prompt` method should contain the logic for sending a prompt to the language model and returning the response.

## Tools

Tools are used by the agent to perform actions. They are defined by the `Tool` trait, which has two methods: `definition` and `call`.

*   **`definition`:** This method returns a `ToolDefinition` struct that describes the tool, including its name, description, and parameters.
*   **`call`:** This method is called by the agent to execute the tool. It takes a set of arguments and returns a result.

To create a new tool, you need to implement the `Tool` trait for your struct. The `definition` method should provide a JSON schema for the tool's parameters, and the `call` method should contain the logic for executing the tool.

## Shutdown

The `Shutdown` trait provides a mechanism for gracefully shutting down the agent. It has a single method: `wait_for_signal`. This method returns a future that resolves when a shutdown signal is received.

To create a new shutdown handler, you need to implement the `Shutdown` trait for your struct. The `wait_for_signal` method should contain the logic for listening for a shutdown signal.