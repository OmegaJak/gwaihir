use super::TriggerContext;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use upon::Engine;

#[derive(Error, Debug)]
pub enum TextTemplateError {
    #[error("Template compilation failed: {0}")]
    CompilationFailure(upon::Error),
    #[error("Template rendering failed: {0}")]
    RenderFailure(upon::Error),
}

#[derive(Serialize, Deserialize)]
#[serde(into = "TextTemplateShadow", try_from = "TextTemplateShadow")]
pub struct TextTemplate {
    raw_text: String,
    template: Engine<'static>,
}

#[derive(Serialize, Deserialize)]
struct TextTemplateShadow {
    text: String,
}

#[derive(Serialize)]
struct RenderContext {
    user: String,
}

impl TryFrom<TextTemplateShadow> for TextTemplate {
    type Error = TextTemplateError;

    fn try_from(value: TextTemplateShadow) -> Result<Self, Self::Error> {
        Self::new(value.text)
    }
}

impl From<TextTemplate> for TextTemplateShadow {
    fn from(value: TextTemplate) -> Self {
        Self {
            text: value.raw_text,
        }
    }
}

impl Clone for TextTemplate {
    fn clone(&self) -> Self {
        Self::new(self.raw_text.clone())
            .expect("A template that was valid should still be valid with the same text")
    }
}

impl PartialEq for TextTemplate {
    fn eq(&self, other: &Self) -> bool {
        self.raw_text == other.raw_text
    }
}

impl TextTemplate {
    const TEMPLATE_NAME: &'static str = "text";

    pub fn new(text: String) -> Result<Self, TextTemplateError> {
        Ok(Self {
            raw_text: text.clone(),
            template: Self::compile_template(text)?,
        })
    }

    pub fn raw_text(&self) -> String {
        self.raw_text.clone()
    }

    pub fn recompile(&mut self, new_text: String) -> Result<(), TextTemplateError> {
        self.template = Self::compile_template(new_text.clone())?;
        self.raw_text = new_text;

        Ok(())
    }

    pub fn render<T>(
        &self,
        context: &TriggerContext<'_, '_, T>,
    ) -> Result<String, TextTemplateError> {
        let render_context = RenderContext {
            user: context.user.clone(),
        };
        self.template
            .template(Self::TEMPLATE_NAME)
            .render(&render_context)
            .to_string()
            .map_err(TextTemplateError::RenderFailure)
    }

    pub fn get_available_variables() -> Vec<String> {
        vec!["{{user}}".to_owned()]
    }

    fn compile_template<'a>(text: String) -> Result<Engine<'a>, TextTemplateError> {
        let mut template = Engine::new();
        template
            .add_template(Self::TEMPLATE_NAME, text)
            .map_err(TextTemplateError::CompilationFailure)?;
        Ok(template)
    }
}
