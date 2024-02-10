use super::TriggerContext;
use crate::notification::NotificationDispatch;
use log_err::LogErrResult;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use upon::Engine;

#[derive(Serialize, Deserialize)]
#[serde(
    into = "NotificationTemplateShadow",
    from = "NotificationTemplateShadow"
)]
pub struct NotificationTemplate {
    summary: String,
    body: String,

    template: Engine<'static>,
}

#[derive(Serialize, Deserialize)]
struct NotificationTemplateShadow {
    summary: String,
    body: String,
}

#[derive(Serialize)]
struct RenderContext {
    user: String,
}

#[derive(Error, Debug)]
pub enum NotificationTemplateError {
    #[error("Template compilation failed: {0}")]
    CompilationFailure(#[from] upon::Error),
}

impl From<NotificationTemplateShadow> for NotificationTemplate {
    fn from(shadow: NotificationTemplateShadow) -> Self {
        NotificationTemplate::new(shadow.summary, shadow.body)
    }
}

impl From<NotificationTemplate> for NotificationTemplateShadow {
    fn from(value: NotificationTemplate) -> Self {
        Self {
            summary: value.summary,
            body: value.body,
        }
    }
}

impl Clone for NotificationTemplate {
    fn clone(&self) -> Self {
        Self::new(self.summary.clone(), self.body.clone())
    }
}

impl PartialEq for NotificationTemplate {
    fn eq(&self, other: &Self) -> bool {
        self.summary == other.summary && self.body == other.body
    }
}

impl NotificationTemplate {
    const SUMMARY_TEMPLATE_NAME: &'static str = "summary";
    const BODY_TEMPLATE_NAME: &'static str = "body";

    pub fn new(summary: String, body: String) -> Self {
        Self {
            template: Self::compile_template(summary.clone(), body.clone()).unwrap(),
            summary,
            body,
        }
    }

    pub fn summary(&self) -> String {
        self.summary.clone()
    }

    pub fn body(&self) -> String {
        self.body.clone()
    }

    pub fn recompile_with_summary(
        &mut self,
        summary: String,
    ) -> Result<(), NotificationTemplateError> {
        self.template = Self::compile_template(summary.clone(), self.body())?;
        self.summary = summary;

        Ok(())
    }

    pub fn recompile_with_body(&mut self, body: String) -> Result<(), NotificationTemplateError> {
        self.template = Self::compile_template(self.summary(), body.clone())?;
        self.body = body;

        Ok(())
    }

    pub fn get_available_variables() -> Vec<String> {
        vec!["{{user}}".to_owned()]
    }

    pub(super) fn show_notification<T: NotificationDispatch>(
        &self,
        context: &TriggerContext<'_, T>,
    ) {
        let render_context = RenderContext {
            user: context.user.clone(),
        };
        let summary = self
            .template
            .template(Self::SUMMARY_TEMPLATE_NAME)
            .render(&render_context)
            .to_string()
            .log_unwrap();
        let body = self
            .template
            .template(Self::BODY_TEMPLATE_NAME)
            .render(&render_context)
            .to_string()
            .log_unwrap();
        context
            .notification_dispatch
            .show_notification(&summary, &body);
    }

    fn compile_template<'a>(
        summary: String,
        body: String,
    ) -> Result<Engine<'a>, NotificationTemplateError> {
        let mut template = Engine::new();
        template.add_template(Self::SUMMARY_TEMPLATE_NAME, summary)?;
        template.add_template(Self::BODY_TEMPLATE_NAME, body)?;
        Ok(template)
    }
}
