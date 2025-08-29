# Events

Events are the primary way that triggers communicate with the agent. They are represented by the `TEvent` struct, which has two fields:

*   `name`: A string that identifies the type of event.
*   `payload`: An optional JSON value that contains the data associated with the event.

The payload of an event can have any valid JSON structure. This allows for a great deal of flexibility in the types of events that can be created and processed.

## Templating

ForgeFlow uses a templating system to allow agents to dynamically construct prompts based on the data received from triggers. This is particularly useful for handling complex event structures.

The templating engine uses the `handlebars` crate, which provides a simple and powerful syntax for creating templates. You can access nested fields in the JSON payload using dot notation. The `verbatim` helper is used to serialize a JSON object or array into a string.

For more information on how to use templates, please refer to the [Handlebars documentation](https://handlebarsjs.com/).
