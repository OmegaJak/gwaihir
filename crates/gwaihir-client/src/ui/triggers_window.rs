use super::widgets::show_centered_window;
use crate::triggers::{Action, NotificationTemplate, Trigger, TriggerManager};
use egui::Color32;

pub struct TriggersWindow {
    shown: bool,
    criteria_input: String,
    notification_summary_input: String,
    notification_body_input: String,
    drop_after_match: bool,
    err: Option<String>,
}

impl TriggersWindow {
    pub fn new() -> Self {
        Self {
            shown: false,
            criteria_input: Default::default(),
            notification_summary_input: Default::default(),
            notification_body_input: Default::default(),
            drop_after_match: false,
            err: None,
        }
    }

    pub fn set_shown(&mut self, shown: bool) {
        self.shown = shown;
    }

    pub fn show(&mut self, ctx: &egui::Context, change_matcher: &mut TriggerManager) {
        self.shown = show_centered_window(self.shown, "Triggers", ctx, |ui| {
            ui.heading("Current");
            egui::Grid::new("existing_notifications")
                .num_columns(1)
                .striped(true)
                .show(ui, |ui| {
                    for (id, serialized_matcher_criteria) in
                        change_matcher.get_serialized_triggers()
                    {
                        ui.horizontal(|ui| {
                            ui.label(serialized_matcher_criteria.clone());
                            if ui.button("Copy").clicked() {
                                ui.output_mut(|o| {
                                    o.copied_text = serialized_matcher_criteria.clone()
                                });
                            }

                            if ui.button("X").clicked() {
                                change_matcher.remove_trigger_by_id(&id);
                            }
                        });
                        ui.end_row();
                    }
                });

            ui.heading("Add new ");
            ui.horizontal(|ui| {
                ui.label("Criteria: ");
                egui::TextEdit::singleline(&mut self.criteria_input)
                    .desired_width(f32::INFINITY)
                    .show(ui);
            });
            ui.horizontal(|ui| {
                ui.label("Notif Summary: ");
                egui::TextEdit::singleline(&mut self.notification_summary_input)
                    .desired_width(f32::INFINITY)
                    .show(ui);
            });
            ui.horizontal(|ui| {
                ui.label("Notif Body: ");
                ui.text_edit_multiline(&mut self.notification_body_input);
            });
            ui.checkbox(&mut self.drop_after_match, "Drop After Match");
            if ui.button("Add").clicked() {
                match ron::from_str(&self.criteria_input) {
                    Ok(criteria) => {
                        let matcher = Trigger {
                            criteria,
                            drop_after_trigger: self.drop_after_match,
                            actions: vec![Action::ShowNotification(NotificationTemplate::new(
                                self.notification_summary_input.clone(),
                                self.notification_body_input.clone(),
                            ))],
                        };
                        change_matcher.add_trigger(matcher);
                        self.criteria_input.clear();
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
