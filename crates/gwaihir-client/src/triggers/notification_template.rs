use super::{
    text_template::{TextTemplate, TextTemplateError},
    TriggerContext,
};
use crate::notification::NotificationDispatch;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(
    into = "NotificationTemplateShadow",
    try_from = "NotificationTemplateShadow"
)]
pub struct NotificationTemplate {
    pub summary: TextTemplate,
    pub body: TextTemplate,
}

#[derive(Serialize, Deserialize)]
struct NotificationTemplateShadow {
    summary: String,
    body: String,
}

impl TryFrom<NotificationTemplateShadow> for NotificationTemplate {
    type Error = TextTemplateError;

    fn try_from(value: NotificationTemplateShadow) -> Result<Self, Self::Error> {
        NotificationTemplate::new(value.summary, value.body)
    }
}

impl From<NotificationTemplate> for NotificationTemplateShadow {
    fn from(value: NotificationTemplate) -> Self {
        Self {
            summary: value.summary.raw_text(),
            body: value.body.raw_text(),
        }
    }
}

impl Clone for NotificationTemplate {
    fn clone(&self) -> Self {
        Self::new(self.summary.raw_text(), self.body.raw_text())
            .expect("A template that was valid should still be valid with the same text")
    }
}

impl Default for NotificationTemplate {
    fn default() -> Self {
        Self::new("".to_owned(), "".to_owned()).expect("empty strings are valid text templates")
    }
}

impl NotificationTemplate {
    pub fn new(summary: String, body: String) -> Result<Self, TextTemplateError> {
        Ok(Self {
            summary: TextTemplate::new(summary)?,
            body: TextTemplate::new(body)?,
        })
    }

    pub fn summary_text(&self) -> String {
        self.summary.raw_text()
    }

    pub fn body_text(&self) -> String {
        self.body.raw_text()
    }

    pub fn recompile_with_summary(&mut self, summary: String) -> Result<(), TextTemplateError> {
        self.summary.recompile(summary)?;
        Ok(())
    }

    pub fn recompile_with_body(&mut self, body: String) -> Result<(), TextTemplateError> {
        self.body.recompile(body)?;
        Ok(())
    }

    pub(super) fn show_notification<T: NotificationDispatch>(
        &self,
        context: &TriggerContext<'_, '_, T>,
    ) -> Result<(), TextTemplateError> {
        let summary = self.summary.render(context)?;
        let body = self.body.render(context)?;
        context
            .notification_dispatch
            .show_notification(&summary, &body);
        Ok(())
    }
}
