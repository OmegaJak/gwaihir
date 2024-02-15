use super::text_template_extensions::{TextEditStyle, TextTemplateExtensions};
use crate::{
    triggers::{summary_template::SummaryTemplate, Action, NotificationTemplate},
    ui::ui_extension_methods::UIExtensionMethods,
};

pub trait ActionWidgetExtensions {
    fn ui(&mut self, id_base: String, ui: &mut egui::Ui) -> egui::Response;
}

impl ActionWidgetExtensions for Action {
    fn ui(&mut self, id_base: String, ui: &mut egui::Ui) -> egui::Response {
        let res = match self {
            Action::ShowNotification(template) => template.ui(format!("{id_base}_notif"), ui),
            Action::SetSummary(summary) => summary.ui(format!("{id_base}_setsummary"), ui),
        };

        res.on_hover_text_at_pointer("Right click for more options")
            .context_menu(|ui| {
                ui.menu_button("Swap", |ui| {
                    if !matches!(self, Action::SetSummary(_)) && ui.button("Set Summary").clicked()
                    {
                        *self = Action::SetSummary(SummaryTemplate::default())
                    }

                    if !matches!(self, Action::ShowNotification(_))
                        && ui.button("Show Notification").clicked()
                    {
                        *self = Action::ShowNotification(NotificationTemplate::default())
                    }
                });
            })
    }
}

impl ActionWidgetExtensions for NotificationTemplate {
    fn ui(&mut self, id_base: String, ui: &mut egui::Ui) -> egui::Response {
        ui.collapsing_default_open_with_id("Send Notification", id_base.clone(), |ui| {
            self.summary.ui(
                format!("{id_base}_summary"),
                "Summary: ",
                TextEditStyle::Singleline,
                ui,
            );

            self.body.ui(
                format!("{id_base}_body"),
                "Body: ",
                TextEditStyle::Multiline,
                ui,
            );
        })
        .header_response
    }
}

impl ActionWidgetExtensions for SummaryTemplate {
    fn ui(&mut self, id_base: String, ui: &mut egui::Ui) -> egui::Response {
        ui.collapsing_default_open_with_id("Set Summary", id_base.clone(), |ui| {
            self.summary.ui(
                format!("{id_base}_setsummary"),
                "Summary: ",
                TextEditStyle::Singleline,
                ui,
            );
        })
        .header_response
    }
}
