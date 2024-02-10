use super::{trigger_widget_extensions::TriggerWidgetExtension, TriggerAction};
use crate::{
    triggers::{Trigger, TriggerManager},
    ui::widgets::show_centered_window,
};

pub struct TriggersWindow {
    shown: bool,
    last_deleted_trigger: Option<Trigger>,
}

impl Default for TriggersWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl TriggersWindow {
    pub fn new() -> Self {
        Self {
            shown: false,
            last_deleted_trigger: None,
        }
    }

    pub fn set_shown(&mut self, shown: bool) {
        self.shown = shown;
    }

    pub fn show(&mut self, ctx: &egui::Context, trigger_manager: &mut TriggerManager) {
        self.shown = show_centered_window(self.shown, "Triggers", ctx, |ui| {
            egui::TopBottomPanel::bottom("triggers_bottom_panel")
                .resizable(false)
                .min_height(0.0)
                .show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Reset default triggers").clicked() {
                            trigger_manager.reset_default_triggers();
                        }
                        if let Some(trigger) = self.last_deleted_trigger.as_ref() {
                            if ui.button("Recover last deleted trigger").clicked() {
                                trigger_manager.add_trigger(trigger.clone());
                                self.last_deleted_trigger = None;
                            }
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            if ui.button("Add trigger").clicked() {
                                trigger_manager.add_trigger(Default::default());
                            }
                        });
                    });
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut triggers_to_remove = Vec::new();
                    for (id, trigger) in trigger_manager.triggers_iter_mut() {
                        if let TriggerAction::Delete = trigger.ui(id, ui) {
                            self.last_deleted_trigger = Some(trigger.clone());
                            triggers_to_remove.push(id.to_owned());
                        }
                    }

                    for id in triggers_to_remove {
                        trigger_manager.remove_trigger_by_id(&id);
                    }
                });
            });
        });
    }
}
