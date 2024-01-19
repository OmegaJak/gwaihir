use super::widgets::show_centered_window;
use crate::{networking::network_manager::NetworkManager, persistence::Persistence};
use egui::ComboBox;
use gwaihir_client_lib::{NetworkInterface, NetworkType};

pub struct NetworkWindow {
    shown: bool,
    selected_network_type: NetworkType,
}

impl NetworkWindow {
    pub fn new(network_manager: &NetworkManager) -> Self {
        let initial_network_type = network_manager.get_network_type();
        Self {
            shown: false,
            selected_network_type: initial_network_type,
        }
    }

    pub fn set_shown(&mut self, shown: bool) {
        self.shown = shown;
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        network_manager: &mut NetworkManager,
        persistence: &mut Persistence,
        clear_user_statuses: impl FnOnce(),
    ) {
        self.shown = show_centered_window(self.shown, "Network", ctx, |ui| {
            ui.label(format!(
                "Current network: {}",
                network_manager.get_network_type()
            ));
            ui.horizontal(|ui| {
                ui.label("Network: ");
                ComboBox::from_id_source("network_type_selector")
                    .selected_text(self.selected_network_type.to_string())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.selected_network_type,
                            NetworkType::Offline,
                            NetworkType::Offline.to_string(),
                        );
                        ui.selectable_value(
                            &mut self.selected_network_type,
                            NetworkType::SpacetimeDB,
                            NetworkType::SpacetimeDB.to_string(),
                        );
                    });
            });

            self.show_network_specific_config(ui, persistence);

            if ui.button("Update Network").clicked() {
                clear_user_statuses();
                network_manager.reinit_network(
                    self.selected_network_type.clone(),
                    persistence.spacetimedb_db_name.clone(),
                );
            }
        });
    }

    fn show_network_specific_config(&self, ui: &mut egui::Ui, persistence: &mut Persistence) {
        match self.selected_network_type {
            NetworkType::Offline => (),
            NetworkType::SpacetimeDB => {
                ui.horizontal(|ui| {
                    ui.label("DB Name: ");
                    ui.text_edit_singleline(&mut persistence.spacetimedb_db_name);
                });
            }
        }
    }
}
