use egui::{CollapsingHeader, Color32, RichText, Stroke, Vec2, WidgetText};
use egui_plot::{
    log_grid_spacer, uniform_grid_spacer, AxisBools, Bar, BarChart, CoordinatesFormatter, Legend,
    Line, Plot, PlotPoints,
};
use log::warn;
use serde::{Deserialize, Serialize};

use crate::{sensors::keyboard_mouse_sensor, ui::ui_extension_methods::UIExtensionMethods};

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
    min: f64,
    value: f64,
    max: f64,
}

enum UsageLevel {
    High,
    Medium,
    Low,
    None,
}

impl SensorWidget for KeyboardMouseActivity {
    fn show(&self, ui: &mut egui::Ui, id: &gwaihir_client_lib::UniqueUserId) {
        let keyboard_summary = summarize(&self.keyboard_usage);
        let mouse_button_summary = summarize(&self.mouse_button_usage);
        let mouse_movement_summary = summarize(&self.mouse_movement);

        let text = ui.create_default_layout_job(vec![
            RichText::new("Keyboard: ").color(ui.visuals().text_color()),
            get_summary_text(&keyboard_summary),
            RichText::new(", Mouse Buttons: ").color(ui.visuals().text_color()),
            get_summary_text(&mouse_button_summary),
            RichText::new(", Mouse Movement: ").color(ui.visuals().text_color()),
            get_summary_text(&mouse_movement_summary),
        ]);

        CollapsingHeader::new(text)
            .default_open(false)
            .id_source(format!("{}_kb_mouse_overview", id.as_ref()))
            .show(ui, |ui| {
                show_activity_graph_section(
                    "Keyboard",
                    &self.keyboard_usage.data,
                    keyboard_summary,
                    ui,
                    id,
                );
                show_activity_graph_section(
                    "Mouse Buttons",
                    &self.mouse_button_usage.data,
                    mouse_button_summary,
                    ui,
                    id,
                );
                show_activity_graph_section(
                    "Mouse Movement",
                    &self.mouse_movement.data,
                    mouse_movement_summary,
                    ui,
                    id,
                );
            });
    }
}

fn show_activity_graph_section(
    collapse_header_text: &str,
    activity_data: &[f64],
    data_summary: Option<UsageSummary>,
    ui: &mut egui::Ui,
    id: &gwaihir_client_lib::UniqueUserId,
) {
    CollapsingHeader::new(collapse_header_text)
        .id_source(format!("{}_{}_graph", id.as_ref(), collapse_header_text))
        .default_open(true)
        .show(ui, |ui| {
            if !activity_data.is_empty() {
                show_activity_graph(activity_data, data_summary, ui);
            } else {
                ui.label("No data yet");
            }
        });
}

fn show_activity_graph(
    activity_data: &[f64],
    data_summary: Option<UsageSummary>,
    ui: &mut egui::Ui,
) {
    let bucket_duration_s = keyboard_mouse_sensor::BUCKET_DURATION_SECONDS as f64;
    let bars: Vec<_> = activity_data
        .iter()
        .rev()
        .enumerate()
        .map(|(i, v)| {
            let color = if let Some(summary) = data_summary.as_ref() {
                let usage_level = get_usage_level(*v, summary.min, summary.max);
                usage_level.map_or(Color32::BLACK, |l| get_usage_level_color(&l))
            } else {
                Color32::BLACK
            };
            Bar::new(
                -(bucket_duration_s / 2.0) - ((i as f64) * bucket_duration_s),
                *v,
            )
            .width(bucket_duration_s)
            .stroke(Stroke::new(1.0, color))
            .fill(color.linear_multiply(0.2))
        })
        .collect();
    let mut bar_chart = BarChart::new(bars)
        .color(Color32::BLUE)
        .name("Normal Distribution");

    Plot::new("Normal Distribution Demo")
        .clamp_grid(false)
        .set_margin_fraction(Vec2::new(0.0, 0.1))
        .include_y(1.0) // so we have something when there's no data
        .include_y(data_summary.map_or(1.0, |s| s.max) * 1.5) // so we can see label
        .auto_bounds_x()
        .x_axis_label("seconds")
        .auto_bounds_y()
        .y_axis_width(3)
        .height(100.0)
        .allow_zoom(false)
        .allow_drag(false)
        .allow_boxed_zoom(false)
        .x_grid_spacer(uniform_grid_spacer(|_i| [600.0, 60.0, 10.0]))
        .show(ui, |plot_ui| plot_ui.bar_chart(bar_chart));
}

fn get_summary_text(summary: &Option<UsageSummary>) -> RichText {
    if let Some(summary) = summary {
        let color = get_usage_level_color(&summary.level);
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

fn get_usage_level_color(level: &UsageLevel) -> Color32 {
    match level {
        UsageLevel::None => Color32::from_rgb(0, Color32::DARK_GREEN.g() / 2, 0),
        UsageLevel::Low => Color32::DARK_GREEN,
        UsageLevel::Medium => Color32::GOLD,
        UsageLevel::High => Color32::RED,
    }
}

fn summarize(activity: &KeyboardMouseActivityData) -> Option<UsageSummary> {
    if activity.data.is_empty() {
        return Some(UsageSummary {
            level: UsageLevel::None,
            min: 0.0,
            value: 0.0,
            max: 0.0,
        });
    }

    let min = 0.0;
    let max = activity
        .data
        .iter()
        .max_by(|a, b| a.total_cmp(b))
        .map(|v| *v)?;
    let most_recent_value = activity.data.last()?;

    Some(UsageSummary {
        level: get_usage_level(*most_recent_value, min, max)?,
        min,
        value: *most_recent_value,
        max,
    })
}

fn get_usage_level(value: f64, min: f64, max: f64) -> Option<UsageLevel> {
    let fractional_usage = if max != min {
        (value - min) / (max - min)
    } else {
        0.0
    };

    match fractional_usage {
        x if x == 0.0 => Some(UsageLevel::None),
        x if x <= ONE_THIRD => Some(UsageLevel::Low),
        x if x <= TWO_THIRDS => Some(UsageLevel::Medium),
        x if x <= 1.0 => Some(UsageLevel::High),
        _ => None,
    }
}