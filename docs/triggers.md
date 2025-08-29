# Triggers

Triggers are responsible for initiating agent actions. They are defined by the `Trigger` trait, which has a single method: `launch`. This method launches a long-running task that will send events to the agent.

## `PollTrigger`

This trigger fires an event at a regular interval. It can be configured with a payload, a frequency, and a `hot_start` option to fire an event immediately upon launch.

### Event Structure

The `PollTrigger` generates a `TEvent` with the following structure:

```json
{
  "name": "<event_name>",
  "payload": null
}
```

*   `name`: The name of the event, as specified in the `PollTrigger`'s configuration.
*   `payload`: This is always `null` for the `PollTrigger`.

### Example

```rust
use forgeflow::triggers::PollTrigger;
use std::time::Duration;

let trigger = PollTrigger::new(
    "The Rust Programming Language",
    Duration::from_secs(12),
    true,
);
```

## `GmailWatchTrigger`

This trigger watches for new unread emails in a Gmail account. It uses the Gmail API to check for new emails and sends an event to the agent when a new email is found.

### Event Structure

The `GmailWatchTrigger` generates a `TEvent` with the name `NewEmail` and a payload that contains the full email object from the Gmail API. For more information on the structure of the `NewEmail` event, please refer to the [Gmail `NewEmail` Event documentation](./events/gmail_event.md).

### Example

```rust
use forgeflow::triggers::GmailWatchTrigger;
use forgeflow::utils::google_auth::GConf;
use std::path::Path;

let conf = GConf::new(
    Path::new("./tmp/credential.json").to_path_buf(),
    Path::new("./tmp/token.json").to_path_buf(),
);
let trigger = GmailWatchTrigger::new(conf.clone()).await.unwrap();
```
