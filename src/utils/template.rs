use handlebars::{Handlebars, handlebars_helper, no_escape};
use serde_json::{self, Value};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TEngineError {
    #[error("I/O error")]
    IoError(#[from] std::io::Error),
    #[error("Template error")]
    TemplateError(#[from] handlebars::TemplateError),
    #[error("JSON error")]
    JsonError(#[from] serde_json::Error),
    #[error("Render error")]
    RenderError(#[from] handlebars::RenderError),
    #[error("Template not found")]
    TemplateNotFoundError(String),
}

pub struct TEngine {
    handlebars: Handlebars<'static>,
}

impl Default for TEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TEngine {
    pub fn new() -> Self {
        let mut te = TEngine {
            handlebars: Handlebars::new(),
        };
        handlebars_helper!(obj: |v: Value| {
            //let output = output.replace("\"", "'/");
            serde_json::to_string(&v).unwrap()
        });
        te.handlebars.register_escape_fn(no_escape);
        te.handlebars.register_helper("verbatim", Box::new(obj));
        te
    }

    pub fn register_template_string(
        &mut self,
        name: &str,
        template: &str,
    ) -> Result<(), TEngineError> {
        self.handlebars.register_template_string(name, template)?;
        Ok(())
    }

    pub fn render_template(
        &self,
        template: &str,
        data: &serde_json::Value,
    ) -> Result<String, TEngineError> {
        let result = self.handlebars.render_template(template, data)?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let engine = TEngine::new();
        let template = "Hello, {{name}}!";
        let data = serde_json::json!({"name": "World"});
        let rendered = engine.render_template(template, &data).unwrap();
        assert_eq!(rendered, "Hello, World!");
    }

    #[test]
    fn it_works_with_complex_data() {
        let engine = TEngine::new();
        let template = "my story, {{verbatim story}}!";
        let data = serde_json::json!({"story": [{"year": 1920, "work": "novel"}, {"year": 1930, "work": "poem"}, {"year": 1940, "work": "short story"}]});
        let rendered = engine.render_template(template, &data).unwrap();
        assert_eq!(
            rendered,
            "my story, [{\"work\":\"novel\",\"year\":1920},{\"work\":\"poem\",\"year\":1930},{\"work\":\"short story\",\"year\":1940}]!"
        );
    }
}
