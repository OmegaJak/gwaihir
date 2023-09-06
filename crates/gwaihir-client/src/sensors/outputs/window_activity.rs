use chrono_humanize::HumanTime;
use egui::{CollapsingHeader, RichText};
use gwaihir_client_lib::chrono::{DateTime, Local, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ui_extension_methods::UIExtensionMethods;

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
        let time_using_current: HumanTime = HumanTime::from(
            self.current_window
                .started_using
                .signed_duration_since(Local::now()),
        );
        ui.horizontal_with_no_item_spacing(|ui| {
            ui.label("Current window: ");
            ui.label(RichText::new(format!("{}", self.current_window.app_name)).strong());
            ui.label(" (started using ");
            ui.label(format!("{}", time_using_current))
                .on_hover_text_at_pointer(format!(
                    "{}",
                    self.current_window.started_using.with_timezone(&Local)
                ));
            ui.label(")");
        });

        if !self.previously_active_windows.is_empty() {
            CollapsingHeader::new("Previous Windows")
                .default_open(false)
                .id_source(format!("{}_previous_windows", id.as_ref()))
                .show(ui, |ui| {
                    for previous_window in self.previously_active_windows.iter() {
                        ui.horizontal_with_no_item_spacing(|ui| {
                            ui.label(
                                RichText::new(format!("{} ", previous_window.app_name)).strong(),
                            );
                            ui.label("from ");
                            ui.label(format!(
                                "{} to ",
                                self.format_datetime(
                                    previous_window.started_using.with_timezone(&Local)
                                )
                            ))
                            .on_hover_text_at_pointer(format!(
                                "{}",
                                previous_window.started_using.with_timezone(&Local)
                            ));
                            ui.label(format!(
                                "{}",
                                self.format_datetime(
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
}

impl WindowActivity {
    fn format_datetime(&self, datetime: DateTime<Local>) -> String {
        let time_format = "%l:%M%P";
        if datetime.date_naive() == Local::now().date_naive() {
            return datetime.format(time_format).to_string();
        } else {
            return datetime.format(&format!("%D {}", time_format)).to_string();
        }
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
