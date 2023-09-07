use chrono_humanize::HumanTime;
use egui::{CollapsingHeader, RichText};
use gwaihir_client_lib::chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};

use crate::ui_extension_methods::{nicely_formatted_datetime, UIExtensionMethods};

use super::sensor_output::SensorWidget;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct WindowActivity {
    pub current_window: ActiveWindow,
    pub previously_active_windows: Vec<PreviouslyActiveWindow>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ActiveWindow {
    pub app_name: String,
    pub started_using: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct PreviouslyActiveWindow {
    pub app_name: String,
    pub started_using: DateTime<Utc>,
    pub stopped_using: DateTime<Utc>,
}

pub trait RepresentsWindow {
    fn app_name(&self) -> &str;
}

pub trait SameWindow {
    fn same_window_as(&self, other: &impl RepresentsWindow) -> bool;
}

impl SensorWidget for WindowActivity {
    fn show(&self, ui: &mut egui::Ui, id: &gwaihir_client_lib::UniqueUserId) {
        let time_using_current: HumanTime = HumanTime::from(self.current_window.started_using);

        let layout_job = ui.create_default_layout_job(vec![
            RichText::new("Using: ").color(ui.style().visuals.text_color()),
            RichText::new(format!("{} ", self.current_window.app_name)).strong(),
            RichText::new(format!("(started using {})", time_using_current))
                .color(ui.style().visuals.text_color()),
        ]);

        CollapsingHeader::new(layout_job)
            .default_open(false)
            .id_source(format!("{}_previous_windows", id.as_ref()))
            .show(ui, |ui| {
                for previous_window in self.previously_active_windows.iter() {
                    ui.horizontal_with_no_item_spacing(|ui| {
                        ui.label(RichText::new(format!("{} ", previous_window.app_name)).strong());
                        ui.label("from ");
                        ui.label(format!(
                            "{} to ",
                            nicely_formatted_datetime(
                                previous_window.started_using.with_timezone(&Local)
                            )
                        ))
                        .on_hover_text_at_pointer(format!(
                            "{}",
                            previous_window.started_using.with_timezone(&Local)
                        ));
                        ui.label(format!(
                            "{}",
                            nicely_formatted_datetime(
                                previous_window.stopped_using.with_timezone(&Local)
                            )
                        ))
                        .on_hover_text_at_pointer(format!(
                            "{}",
                            previous_window.stopped_using.with_timezone(&Local)
                        ));
                    });
                }
            });
    }
}

impl From<active_win_pos_rs::ActiveWindow> for ActiveWindow {
    fn from(window: active_win_pos_rs::ActiveWindow) -> Self {
        Self {
            app_name: window.app_name,
            started_using: Utc::now(),
        }
    }
}

impl RepresentsWindow for ActiveWindow {
    fn app_name(&self) -> &str {
        &self.app_name
    }
}

impl ActiveWindow {
    pub fn to_no_longer_active(self) -> PreviouslyActiveWindow {
        PreviouslyActiveWindow {
            app_name: self.app_name,
            started_using: self.started_using,
            stopped_using: Utc::now(),
        }
    }
}

impl RepresentsWindow for PreviouslyActiveWindow {
    fn app_name(&self) -> &str {
        &self.app_name
    }
}

impl RepresentsWindow for active_win_pos_rs::ActiveWindow {
    fn app_name(&self) -> &str {
        &self.app_name
    }
}

impl<T: RepresentsWindow> SameWindow for T {
    fn same_window_as(&self, other: &impl RepresentsWindow) -> bool {
        self.app_name() == other.app_name()
    }
}
