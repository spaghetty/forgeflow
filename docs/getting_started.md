# Getting Started

This guide will walk you through the process of setting up and running the project.

## Prerequisites

*   Rust: Make sure you have a recent version of Rust installed. You can find installation instructions on the [official Rust website](https://www.rust-lang.org/tools/install).
*   Google Cloud Project: To use the Gmail-related features, you'll need a Google Cloud project with the Gmail API enabled. You'll also need to create OAuth 2.0 credentials and download them as a `credential.json` file.

## Setup

1.  **Clone the repository:**

    ```bash
    git clone https://github.com/your-username/forgeflow.git
    cd forgeflow
    ```

2.  **Set up Google authentication:**

    *   Place your `credential.json` file in the `tmp` directory.
    *   Run the `gmail_hook` example once to generate a `token.json` file. This will require you to go through the OAuth 2.0 flow in your browser.

3.  **Set up environment variables:**

    The `rig` crate, which is used for interacting with the Gemini API, requires an API key. You can set this key as an environment variable:

    ```bash
    export GEMINI_API_KEY="your-api-key"
    ```

## Running the Examples

Once you have everything set up, you can run the examples using the following commands:

*   **Haiku Generator:**

    ```bash
    cargo run --example haiku_generator
    ```

*   **Gmail Hook:**

    ```bash
    cargo run --example gmail_hook
    ```
