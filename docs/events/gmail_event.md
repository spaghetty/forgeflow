# Gmail `NewEmail` Event

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

## Templating

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
