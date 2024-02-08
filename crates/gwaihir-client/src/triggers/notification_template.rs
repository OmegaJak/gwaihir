use super::TriggerContext;
use crate::notification::NotificationDispatch;
use log_err::LogErrResult;
use serde::{Deserialize, Serialize};
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
    const SUMMARY_TEMPLATE_NAME: &str = "summary";
    const BODY_TEMPLATE_NAME: &str = "body";

    pub fn new(summary: String, body: String) -> Self {
        Self {
            template: Self::compile_template(summary.clone(), body.clone()),
            summary,
            body,
        }
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

    fn compile_template<'a>(summary: String, body: String) -> Engine<'a> {
        let mut template = Engine::new();
        template
            .add_template(Self::SUMMARY_TEMPLATE_NAME, summary)
            .unwrap();
        template
            .add_template(Self::BODY_TEMPLATE_NAME, body)
            .unwrap();
        template
    }
}
