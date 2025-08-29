# Utilities

This document describes the utility modules provided by the Forgeflow framework.

## Google Authentication

The `google_auth` module provides a helper function for authenticating with the Gmail API. It uses the `yup-oauth2` crate to handle the OAuth 2.0 flow and token management.

### `gmail_auth`

This function takes a `GConf` struct and a slice of scopes and returns a `GmailHubType` that can be used to interact with the Gmail API.

### `GConf`

This struct holds the configuration for Google authentication, including the paths to the `credential.json` and `token.json` files.

## Template Engine

The `template` module provides a simple template engine based on the `handlebars` crate. It is used by the agent to render prompts for the language model.

### `TEngine`

This struct is a wrapper around the `Handlebars` struct. It provides a `render_template` method that takes a template string and a JSON value and returns the rendered string.
