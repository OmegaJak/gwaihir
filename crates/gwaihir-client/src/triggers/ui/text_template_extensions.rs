use crate::triggers::{text_template::TextTemplate, TextTemplateError};
use egui::{text_selection::CCursorRange, Color32, TextBuffer};

pub enum TextEditStyle {
    Singleline,
    Multiline,
}

pub trait TextTemplateExtensions {
    fn ui(
        &mut self,
        id_base: String,
        label: impl Into<egui::WidgetText>,
        text_edit_style: TextEditStyle,
        ui: &mut egui::Ui,
    );
}

impl TextTemplateExtensions for TextTemplate {
    fn ui(
        &mut self,
        id_base: String,
        label: impl Into<egui::WidgetText>,
        text_edit_style: TextEditStyle,
        ui: &mut egui::Ui,
    ) {
        let summary_id = format!("{id_base}_summary");
        let egui_id = ui.make_persistent_id(summary_id);
        ui.horizontal(|ui| {
            ui.label(label);
            let mut text = self.raw_text();
            let mut response = match text_edit_style {
                TextEditStyle::Singleline => {
                    egui::TextEdit::singleline(&mut text)
                        .desired_width(f32::INFINITY)
                        .show(ui)
                        .response
                }
                TextEditStyle::Multiline => ui.text_edit_multiline(&mut text),
            };
            response = show_insert_notification_template_variable_menu(response, &mut text);
            if response.changed() {
                let result = self.recompile(text);
                handle_notification_compilation_result(result, egui_id, ui);
            }
        });

        show_err_ui(egui_id, ui);
    }
}

fn show_err_ui(id: egui::Id, ui: &mut egui::Ui) {
    if let Some(err) = ui.memory(|mem| mem.data.get_temp::<String>(id)) {
        ui.colored_label(Color32::RED, err);
    }
}

fn handle_notification_compilation_result(
    result: Result<(), TextTemplateError>,
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
                    format!("Attempted to write an invalid text template: {}\n", e),
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
    text_edit_response.context_menu(|ui| {
        ui.menu_button("Insert Variable", |ui| {
            for variable in TextTemplate::get_available_variables() {
                if ui.button(variable.clone()).clicked() {
                    if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), text_edit_id) {
                        if let Some(range) = state.cursor.char_range() {
                            text_edit_contents.insert_text(&variable, range.primary.index);
                            state.cursor.set_char_range(Some(CCursorRange::one(
                                egui::text::CCursor::new(range.primary.index + variable.len()),
                            )));
                        } else {
                            *text_edit_contents = format!("{text_edit_contents}{variable}");
                            state.cursor.set_char_range(Some(CCursorRange::one(
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
