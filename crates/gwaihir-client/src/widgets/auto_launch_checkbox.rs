use auto_launch::AutoLaunch;
use egui::{Response, Ui, Widget};

#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
pub struct AutoLaunchCheckbox {
    enabled: bool,
    auto_launch: AutoLaunch,
}

impl AutoLaunchCheckbox {
    pub fn new(app_name: String, app_path: Option<String>) -> Self {
        let app_path = app_path.unwrap_or_else(|| {
            std::env::current_exe()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
        });
        let auto_launch = AutoLaunch::new(&app_name, &app_path, &[] as &[&str]);
        let enabled = auto_launch.is_enabled().unwrap();
        AutoLaunchCheckbox {
            enabled,
            auto_launch,
        }
    }
}

impl Widget for AutoLaunchCheckbox {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        let inner_response = ui.checkbox(&mut self.enabled, "Run on startup");
        if inner_response.changed() {
            if self.enabled {
                self.auto_launch.enable().unwrap();
            } else {
                self.auto_launch.disable().unwrap();
            }
        }

        inner_response
    }
}

pub trait AutoLaunchCheckboxUiExtension {
    fn auto_launch_checkbox(&mut self, app_name: String, app_path: Option<String>) -> Response;
}

impl AutoLaunchCheckboxUiExtension for Ui {
    fn auto_launch_checkbox(&mut self, app_name: String, app_path: Option<String>) -> Response {
        let checkbox = AutoLaunchCheckbox::new(app_name, app_path);
        self.add(checkbox)
    }
}
