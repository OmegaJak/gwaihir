use super::widgets::show_centered_window;
use crate::change_matcher::{ChangeMatcher, Matcher};
use egui::Color32;

pub struct NotificationsWindow {
    shown: bool,
    new_matcher_input: String,
    new_matcher_drop_after_match: bool,
    err: Option<String>,
}

impl NotificationsWindow {
    pub fn new() -> Self {
        Self {
            shown: false,
            new_matcher_input: String::new(),
            new_matcher_drop_after_match: false,
            err: None,
        }
    }

    pub fn set_shown(&mut self, shown: bool) {
        self.shown = shown;
    }

    pub fn show(&mut self, ctx: &egui::Context, change_matcher: &mut ChangeMatcher) {
        self.shown = show_centered_window(self.shown, "Notifications", ctx, |ui| {
            ui.heading("Current");
            egui::Grid::new("existing_notifications")
                .num_columns(1)
                .striped(true)
                .show(ui, |ui| {
                    for (id, serialized_matcher_criteria) in
                        change_matcher.get_serialized_matchers()
                    {
                        ui.horizontal(|ui| {
                            ui.label(serialized_matcher_criteria.clone());
                            if ui.button("Copy").clicked() {
                                ui.output_mut(|o| {
                                    o.copied_text = serialized_matcher_criteria.clone()
                                });
                            }

                            if ui.button("X").clicked() {
                                change_matcher.remove_matcher_by_id(&id);
                            }
                        });
                        ui.end_row();
                    }
                });

            ui.heading("Add new ");
            ui.horizontal(|ui| {
                ui.label("Criteria: ");
                egui::TextEdit::singleline(&mut self.new_matcher_input)
                    .desired_width(f32::INFINITY)
                    .show(ui);
            });
            ui.checkbox(&mut self.new_matcher_drop_after_match, "Drop After Match");
            if ui.button("Add").clicked() {
                match ron::from_str(&self.new_matcher_input) {
                    Ok(criteria) => {
                        let matcher = Matcher {
                            criteria,
                            drop_after_match: self.new_matcher_drop_after_match,
                        };
                        change_matcher.add_matcher(matcher);
                        self.new_matcher_input.clear();
                    }
                    Err(err) => self.err = Some(err.to_string()),
                }
            }

            if let Some(err) = &self.err {
                ui.separator();
                ui.colored_label(Color32::RED, err);
            }
        });
    }
}
