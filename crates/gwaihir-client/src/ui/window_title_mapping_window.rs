use std::sync::{Arc, RwLock};

use crate::sensors::window_sensor::{
    active_window_provider::WindowIdentifiers,
    window_title_mapper::{self, WindowTitleMapper, WindowTitleMappings},
};

use super::widgets::show_centered_window;

pub struct WindowTitleMappingWindow {
    shown: bool,
    window_title_mapper: WindowTitleMapper,
}

impl WindowTitleMappingWindow {
    pub fn new(window_title_mapper: WindowTitleMapper) -> Self {
        Self {
            shown: false,
            window_title_mapper,
        }
    }

    pub fn record_observed_title(&mut self, window_identifiers: WindowIdentifiers) {
        self.window_title_mapper
            .record_observed_title(window_identifiers);
    }

    pub fn set_shown(&mut self, shown: bool) {
        self.shown = shown;
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        self.shown = show_centered_window(self.shown, "Window Titles", ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for window_identifiers in self.window_title_mapper.iter_pending() {
                    ui.label(format!(
                        "{}: {}",
                        window_identifiers.app_name, window_identifiers.window_title
                    ));
                }
            });
        });
    }
}
