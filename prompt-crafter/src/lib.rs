#![allow(dead_code)]

/// A component of a prompt.
pub trait PromptComponent {
    fn to_string(&self) -> String;
}

/// A struct representing a prompt.
pub struct Prompt {
    components: Vec<Box<dyn PromptComponent>>,
}

impl Prompt {
    pub fn builder() -> PromptBuilder {
        PromptBuilder::new()
    }

    pub fn to_string(&self) -> String {
        self.components
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
            .join("\n\n")
    }
}

/// A builder for creating prompts.
pub struct PromptBuilder {
    components: Vec<Box<dyn PromptComponent>>,
}

impl PromptBuilder {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    pub fn add(mut self, component: impl PromptComponent + 'static) -> Self {
        self.components.push(Box::new(component));
        self
    }

    pub fn build(self) -> Prompt {
        Prompt {
            components: self.components,
        }
    }
}

/// A component for adding a block of instructions.
pub struct Instruction {
    text: String,
}

impl Instruction {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
        }
    }
}

impl PromptComponent for Instruction {
    fn to_string(&self) -> String {
        format!("### Instruction ###\n{}", self.text)
    }
}

/// A component for adding a block of context.
pub struct Context {
    text: String,
}

impl Context {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
        }
    }
}

impl PromptComponent for Context {
    fn to_string(&self) -> String {
        format!("### Context ###\n{}", self.text)
    }
}

/// A component for defining the persona of the model.
pub struct Persona {
    text: String,
}

impl Persona {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
        }
    }
}

impl PromptComponent for Persona {
    fn to_string(&self) -> String {
        format!("### Persona ###\n{}", self.text)
    }
}

/// A component for providing a single example.
pub struct Example {
    input: String,
    output: String,
}

impl Example {
    pub fn new(input: &str, output: &str) -> Self {
        Self {
            input: input.to_string(),
            output: output.to_string(),
        }
    }
}

impl PromptComponent for Example {
    fn to_string(&self) -> String {
        format!("Input: {}\nOutput: {}", self.input, self.output)
    }
}

/// A component for providing a list of examples.
pub struct FewShot {
    examples: Vec<Example>,
}

impl FewShot {
    pub fn new(examples: Vec<Example>) -> Self {
        Self { examples }
    }
}

impl PromptComponent for FewShot {
    fn to_string(&self) -> String {
        self.examples
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<String>>()
            .join("\n\n")
    }
}

/// A component for adding a separator line.
pub struct Delimiter;

impl PromptComponent for Delimiter {
    fn to_string(&self) -> String {
        "---".to_string()
    }
}

/// A component for defining the desired output format.
pub struct OutputFormat {
    format_description: String,
}

impl OutputFormat {
    pub fn new(format_description: &str) -> Self {
        Self {
            format_description: format_description.to_string(),
        }
    }
}

impl PromptComponent for OutputFormat {
    fn to_string(&self) -> String {
        format!(
            "### Output Format ###\nYour response must be in the following format:\n{}",
            self.format_description
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_to_string() {
        let instruction = Instruction::new("This is an instruction.");
        assert_eq!(
            instruction.to_string(),
            "### Instruction ###\nThis is an instruction."
        );
    }

    #[test]
    fn test_context_to_string() {
        let context = Context::new("This is some context.");
        assert_eq!(
            context.to_string(),
            "### Context ###\nThis is some context."
        );
    }

    #[test]
    fn test_persona_to_string() {
        let persona = Persona::new("You are a helpful assistant.");
        assert_eq!(
            persona.to_string(),
            "### Persona ###\nYou are a helpful assistant."
        );
    }

    #[test]
    fn test_example_to_string() {
        let example = Example::new("Input 1", "Output 1");
        assert_eq!(example.to_string(), "Input: Input 1\nOutput: Output 1");
    }

    #[test]
    fn test_few_shot_to_string() {
        let few_shot = FewShot::new(vec![
            Example::new("Input 1", "Output 1"),
            Example::new("Input 2", "Output 2"),
        ]);
        let expected = "Input: Input 1\nOutput: Output 1\n\nInput: Input 2\nOutput: Output 2";
        assert_eq!(few_shot.to_string(), expected);
    }

    #[test]
    fn test_delimiter_to_string() {
        let delimiter = Delimiter;
        assert_eq!(delimiter.to_string(), "---");
    }

    #[test]
    fn test_output_format_to_string() {
        let output_format = OutputFormat::new("JSON");
        assert_eq!(
            output_format.to_string(),
            "### Output Format ###\nYour response must be in the following format:\nJSON"
        );
    }

    #[test]
    fn test_prompt_builder() {
        let prompt = Prompt::builder()
            .add(Persona::new("You are a helpful assistant."))
            .add(Instruction::new("Instruction 1"))
            .add(Delimiter)
            .add(Context::new("This is some context."))
            .add(FewShot::new(vec![
                Example::new("Input 1", "Output 1"),
                Example::new("Input 2", "Output 2"),
            ]))
            .add(OutputFormat::new("JSON"))
            .build();

        let expected = "### Persona ###\nYou are a helpful assistant.\n\n### Instruction ###\nInstruction 1\n\n---\n\n### Context ###\nThis is some context.\n\nInput: Input 1\nOutput: Output 1\n\nInput: Input 2\nOutput: Output 2\n\n### Output Format ###\nYour response must be in the following format:\nJSON";
        assert_eq!(prompt.to_string(), expected);
    }
}
