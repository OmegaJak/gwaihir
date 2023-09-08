use std::collections::{hash_map::Entry, HashMap};

use chrono_humanize::HumanTime;
use egui::{CollapsingHeader, RichText};
use eternity_rs::Eternity;
use gwaihir_client_lib::chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::DurationSeconds;

use crate::sensors::window_activity_interpreter::DEFAULT_TIME_TO_KEEP_WINDOW_ACTIVITY;
use crate::ui::ui_extension_methods::UIExtensionMethods;

use super::sensor_output::SensorOutput;
use super::{
    sensor_output::SensorWidget,
    window_activity::{ActiveWindow, WindowActivity},
};

const DEFAULT_MAX_NUM_APPS_IN_SUMMARY: usize = 5;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct SummarizedWindowActivity {
    pub current_window: ActiveWindow,
    pub recent_usage: Vec<AppUsage>,
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct AppUsage {
    pub app_name: String,

    #[serde_as(as = "DurationSeconds<i64>")]
    pub recent_usage: Duration,
}

impl From<SummarizedWindowActivity> for SensorOutput {
    fn from(value: SummarizedWindowActivity) -> Self {
        SensorOutput::SummarizedWindowActivity(value)
    }
}

impl SensorWidget for SummarizedWindowActivity {
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
                ui.label(format!(
                    "In the past {}:",
                    DEFAULT_TIME_TO_KEEP_WINDOW_ACTIVITY
                        .to_std()
                        .unwrap()
                        .humanize()
                ));

                for app_usage in self.recent_usage.iter() {
                    ui.horizontal_with_no_item_spacing(|ui| {
                        ui.label("\t");
                        ui.label(RichText::new(app_usage.app_name.clone()).strong());
                        ui.label(" for ");
                        ui.label(RichText::new(format!(
                            "{}",
                            app_usage.recent_usage.to_std().unwrap().humanize()
                        )));
                    });
                }
            });
    }
}

impl SummarizedWindowActivity {
    pub fn summarize(activity: &WindowActivity, now: DateTime<Utc>, cutoff: DateTime<Utc>) -> Self {
        let mut time_totals: HashMap<String, Duration> = HashMap::new();
        Self::add_time_total(
            time_totals.entry(activity.current_window.app_name.clone()),
            activity.current_window.started_using,
            None,
            now,
            cutoff,
        );

        for window in activity.previously_active_windows.iter() {
            if window.stopped_using < cutoff {
                continue;
            }

            Self::add_time_total(
                time_totals.entry(window.app_name.clone()),
                window.started_using,
                Some(window.stopped_using),
                now,
                cutoff,
            )
        }

        let recent_usage = Self::humanize_to_recent_usage(time_totals);
        SummarizedWindowActivity {
            current_window: activity.current_window.clone(),
            recent_usage,
        }
    }

    fn add_time_total(
        entry: Entry<'_, String, Duration>,
        started_using: DateTime<Utc>,
        stopped_using: Option<DateTime<Utc>>,
        now: DateTime<Utc>,
        cutoff: DateTime<Utc>,
    ) {
        let started_using = if started_using > cutoff {
            started_using
        } else {
            cutoff
        };

        let stopped_using = stopped_using.unwrap_or(now);

        let current_duration = stopped_using - started_using;
        entry
            .and_modify(|existing_duration| {
                *existing_duration = current_duration.checked_add(existing_duration).unwrap();
            })
            .or_insert(current_duration);
    }

    fn humanize_to_recent_usage(time_totals: HashMap<String, Duration>) -> Vec<AppUsage> {
        let mut recent_usage = time_totals
            .into_iter()
            .map(|(k, v)| AppUsage {
                app_name: k,
                recent_usage: Self::round_to_nearest_10_seconds(v),
            })
            .filter(|a| !a.recent_usage.is_zero())
            .collect::<Vec<_>>();
        recent_usage.sort_by(|a, b| {
            if a.recent_usage == b.recent_usage {
                a.app_name.cmp(&b.app_name)
            } else {
                b.recent_usage.cmp(&a.recent_usage)
            }
        });
        recent_usage.truncate(DEFAULT_MAX_NUM_APPS_IN_SUMMARY);
        recent_usage
    }

    fn round_to_nearest_10_seconds(duration: Duration) -> Duration {
        Duration::seconds(((duration.to_std().unwrap().as_secs_f64() / 10.0).round() * 10.0) as i64)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::sensors::outputs::window_activity::PreviouslyActiveWindow;
    use gwaihir_client_lib::chrono::NaiveDateTime;

    #[test]
    pub fn can_summarize() {
        // Tests: (yes this should be split into multiple tests)
        // across cutoff
        // ignored before cutoff
        // rounds to the nearest 10 seconds
        // sorts highest to lowest
        // includes current
        // doesn't include 0s (before or after rounding)
        // sorts consistently (UNTESTED)
        // only keeps the top DEFAULT_MAX_NUM_APPS_IN_SUMMARY apps (UNTESTED)
        let cutoff = DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp_millis(1694119546000).unwrap(),
            Utc,
        );
        let now = cutoff + Duration::seconds(100);
        let window_activity = WindowActivity {
            current_window: ActiveWindow {
                app_name: "Current".to_string(),
                started_using: cutoff + Duration::seconds(50),
            },
            previously_active_windows: vec![
                PreviouslyActiveWindow {
                    app_name: "Fully past cutoff".to_string(),
                    started_using: cutoff + Duration::seconds(30),
                    stopped_using: cutoff + Duration::seconds(40) + Duration::milliseconds(5600),
                },
                PreviouslyActiveWindow {
                    app_name: "Fully past cutoff".to_string(),
                    started_using: cutoff + Duration::seconds(10),
                    stopped_using: cutoff + Duration::seconds(20),
                },
                PreviouslyActiveWindow {
                    app_name: "Crossing cutoff".to_string(),
                    started_using: cutoff - Duration::seconds(10),
                    stopped_using: cutoff + Duration::seconds(10),
                },
                PreviouslyActiveWindow {
                    app_name: "Before cutoff".to_string(),
                    started_using: cutoff - Duration::seconds(20),
                    stopped_using: cutoff - Duration::seconds(10),
                },
                PreviouslyActiveWindow {
                    app_name: "Truly Zero".to_string(),
                    started_using: cutoff + Duration::seconds(10),
                    stopped_using: cutoff + Duration::seconds(10),
                },
                PreviouslyActiveWindow {
                    app_name: "Rounded to Zero".to_string(),
                    started_using: cutoff + Duration::seconds(10),
                    stopped_using: cutoff + Duration::seconds(14),
                },
            ],
        };
        let expected_usage = vec![
            AppUsage {
                app_name: "Current".to_string(),
                recent_usage: Duration::seconds(50),
            },
            AppUsage {
                app_name: "Fully past cutoff".to_string(),
                recent_usage: Duration::seconds(30),
            },
            AppUsage {
                app_name: "Crossing cutoff".to_string(),
                recent_usage: Duration::seconds(10),
            },
        ];

        let summary = SummarizedWindowActivity::summarize(&window_activity, now, cutoff);

        assert_eq!(summary.current_window, window_activity.current_window);
        assert_eq!(summary.recent_usage, expected_usage);
    }
}
