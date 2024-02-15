use super::{text_template::TextTemplate, TextTemplateError};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct SummaryTemplate {
    pub summary: TextTemplate,
}

impl Default for SummaryTemplate {
    fn default() -> Self {
        Self {
            summary: TextTemplate::new("Summary".to_owned()).unwrap(),
        }
    }
}

impl SummaryTemplate {
    pub fn new(summary: String) -> Result<Self, TextTemplateError> {
        Ok(Self {
            summary: TextTemplate::new(summary)?,
        })
    }
}
