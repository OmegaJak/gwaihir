use super::{trigger_widget_extensions::TriggerWidgetExtension, TriggerAction};
use crate::{
    triggers::{Trigger, TriggerManager},
    ui::{ui_extension_methods::UIExtensionMethods, widgets::show_centered_window},
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

                        ui.horizontal_right(|ui| {
                            if ui.button("Add trigger").clicked() {
                                trigger_manager.add_trigger(Default::default());
                            }
                        });
                    });
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut trigger_actions = Vec::new();
                    for (id, trigger) in trigger_manager.triggers_iter_mut() {
                        trigger_actions.push((id.to_owned(), trigger.ui(id, ui)));
                    }

                    for (id, action) in trigger_actions {
                        match action {
                            TriggerAction::None => {}
                            TriggerAction::MoveUp => {
                                trigger_manager.move_trigger(&id, -1);
                            }
                            TriggerAction::MoveDown => {
                                trigger_manager.move_trigger(&id, 1);
                            }
                            TriggerAction::Delete => {
                                if let Some(trigger) = trigger_manager.get_trigger_by_id(&id) {
                                    self.last_deleted_trigger = Some(trigger.to_owned());
                                    trigger_manager.remove_trigger_by_id(&id);
                                }
                            }
                        }
                    }
                });
            });
        });
    }
}
