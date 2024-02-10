use egui::{Color32, TextBuffer};

use crate::{
    triggers::{Action, NotificationTemplate, NotificationTemplateError},
    ui::ui_extension_methods::UIExtensionMethods,
};

pub trait ActionWidgetExtensions {
    fn ui(&mut self, id_base: String, ui: &mut egui::Ui);
}

impl ActionWidgetExtensions for Action {
    fn ui(&mut self, id_base: String, ui: &mut egui::Ui) {
        match self {
            Action::ShowNotification(template) => {
                template.ui(id_base, ui);
            }
        }
    }
}

impl ActionWidgetExtensions for NotificationTemplate {
    fn ui(&mut self, id_base: String, ui: &mut egui::Ui) {
        ui.collapsing_default_open_with_id("Send Notification", format!("{id_base}_notif"), |ui| {
            let summary_id = format!("{id_base}_summary");
            let summary_egui_id = ui.make_persistent_id(summary_id);
            ui.horizontal(|ui| {
                ui.label("Summary: ");
                let mut summary = self.summary();
                let mut response = egui::TextEdit::singleline(&mut summary)
                    .desired_width(f32::INFINITY)
                    .show(ui)
                    .response;
                response = show_insert_notification_template_variable_menu(response, &mut summary);
                if response.changed() {
                    let result = self.recompile_with_summary(summary);
                    handle_notification_compilation_result(result, summary_egui_id, ui);
                }
            });

            show_err_ui(summary_egui_id, ui);

            let body_id = format!("{id_base}_body");
            let body_egui_id = ui.make_persistent_id(body_id);
            ui.horizontal(|ui| {
                ui.label("Body: ");
                let mut body = self.body();
                let mut response = ui.text_edit_multiline(&mut body);
                response = show_insert_notification_template_variable_menu(response, &mut body);
                if response.changed() {
                    let result = self.recompile_with_body(body);
                    handle_notification_compilation_result(result, body_egui_id, ui);
                }
            });

            show_err_ui(body_egui_id, ui);
        });
    }
}

fn show_err_ui(id: egui::Id, ui: &mut egui::Ui) {
    if let Some(err) = ui.memory(|mem| mem.data.get_temp::<String>(id)) {
        ui.colored_label(Color32::RED, err);
    }
}

fn handle_notification_compilation_result(
    result: Result<(), NotificationTemplateError>,
    id: egui::Id,
    ui: &mut egui::Ui,
) {
    match result {
        Ok(_) => {
            ui.memory_mut(|mem| mem.data.remove::<String>(id));
        }
        Err(e) => {
            ui.memory_mut(|mem| {
                mem.data.insert_temp(
                    id,
                    format!("Attempted to write invalid notification data: {}\n", e),
                );
            });
        }
    }
}

fn show_insert_notification_template_variable_menu(
    mut text_edit_response: egui::Response,
    text_edit_contents: &mut String,
) -> egui::Response {
    let mut inserted_variable = false;
    let text_edit_id = text_edit_response.id;
    text_edit_response = text_edit_response.context_menu(|ui| {
        ui.menu_button("Insert Variable", |ui| {
            for variable in NotificationTemplate::get_available_variables() {
                if ui.button(variable.clone()).clicked() {
                    if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), text_edit_id) {
                        if let Some(range) = state.ccursor_range() {
                            text_edit_contents.insert_text(&variable, range.primary.index);
                            state.set_ccursor_range(Some(egui::text_edit::CCursorRange::one(
                                egui::text::CCursor::new(range.primary.index + variable.len()),
                            )));
                        } else {
                            *text_edit_contents = format!("{text_edit_contents}{variable}");
                            state.set_ccursor_range(Some(egui::text_edit::CCursorRange::one(
                                egui::text::CCursor::new(text_edit_contents.chars().count()),
                            )));
                        }

                        inserted_variable = true;
                        state.store(ui.ctx(), text_edit_id);
                    } else {
                        log::warn!(
                            "Failed to insert variable because textedit state couldn't be loaded"
                        );
                    }
                    ui.ctx().memory_mut(|mem| mem.request_focus(text_edit_id));
                    ui.close_menu();
                }
            }
        });
    });

    if inserted_variable {
        text_edit_response.mark_changed();
    }

    text_edit_response
}
