use egui::{Color32, RichText};
use serde::{Deserialize, Serialize};

use crate::ui::ui_extension_methods::UIExtensionMethods;

use super::sensor_output::SensorWidget;

const ONE_THIRD: f64 = 1.0 / 3.0;
const TWO_THIRDS: f64 = 2.0 / 3.0;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct KeyboardMouseActivity {
    pub keyboard_usage: KeyboardMouseActivityData,
    pub mouse_movement: KeyboardMouseActivityData,
    pub mouse_button_usage: KeyboardMouseActivityData,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct KeyboardMouseActivityData {
    pub data: Vec<f64>,
}

struct UsageSummary {
    level: UsageLevel,
    value: f64,
}

enum UsageLevel {
    High,
    Medium,
    Low,
    None,
}

impl SensorWidget for KeyboardMouseActivity {
    fn show(&self, ui: &mut egui::Ui, _id: &gwaihir_client_lib::UniqueUserId) {
        let keyboard_summary = summarize(&self.keyboard_usage);
        let mouse_button_summary = summarize(&self.mouse_button_usage);
        let mouse_movement_summary = summarize(&self.mouse_movement);

        if mouse_movement_summary.is_none() {
            ui.label("None");
        }

        let text = ui.create_default_layout_job(vec![
            RichText::new("Keyboard: ").color(ui.visuals().text_color()),
            self.get_summary_text(keyboard_summary),
            RichText::new(", Mouse Buttons: ").color(ui.visuals().text_color()),
            self.get_summary_text(mouse_button_summary),
            RichText::new(", Mouse Movement: ").color(ui.visuals().text_color()),
            self.get_summary_text(mouse_movement_summary),
        ]);

        ui.label(text);
    }
}

impl KeyboardMouseActivity {
    fn get_summary_text(&self, summary: Option<UsageSummary>) -> RichText {
        if let Some(summary) = summary {
            let color = match summary.level {
                UsageLevel::None => Color32::from_rgb(0, Color32::DARK_GREEN.g() / 2, 0),
                UsageLevel::Low => Color32::DARK_GREEN,
                UsageLevel::Medium => Color32::GOLD,
                UsageLevel::High => Color32::RED,
            };
            let level_text = match summary.level {
                UsageLevel::None => "None",
                UsageLevel::Low => "Low",
                UsageLevel::Medium => "Medium",
                UsageLevel::High => "High",
            };

            RichText::new(format!("{} ({:.1})", level_text, summary.value)).color(color)
        } else {
            RichText::new("ERROR").color(Color32::BLUE)
        }
    }
}

fn summarize(activity: &KeyboardMouseActivityData) -> Option<UsageSummary> {
    if activity.data.is_empty() {
        return Some(UsageSummary {
            level: UsageLevel::None,
            value: 0.0,
        });
    }

    let min = 0.0;
    let max = activity
        .data
        .iter()
        .max_by(|a, b| a.total_cmp(b))
        .map(|v| *v)?;
    let most_recent_value = activity.data.last()?;
    let fractional_usage = if max != min {
        (most_recent_value - min) / (max - min)
    } else {
        0.0
    };

    match fractional_usage {
        x if x == 0.0 => Some(UsageSummary {
            level: UsageLevel::None,
            value: *most_recent_value,
        }),
        x if x <= ONE_THIRD => Some(UsageSummary {
            level: UsageLevel::Low,
            value: *most_recent_value,
        }),
        x if x <= TWO_THIRDS => Some(UsageSummary {
            level: UsageLevel::Medium,
            value: *most_recent_value,
        }),
        x if x <= 1.0 => Some(UsageSummary {
            level: UsageLevel::High,
            value: *most_recent_value,
        }),
        _ => None,
    }
}
