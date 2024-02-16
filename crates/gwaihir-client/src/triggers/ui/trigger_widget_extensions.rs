use std::time::Duration;

use gwaihir_client_lib::UniqueUserId;
use uuid::Uuid;

use crate::{
    triggers::{
        ui::comparison_operator::ComparisonOperator, Expression, TimeSpecifier, Trigger,
        ValuePointer,
    },
    ui::ui_extension_methods::UIExtensionMethods,
};

use super::{
    action_widget_extensions::ActionWidgetExtensions, boolean_operator::BooleanOperator,
    user_selectable_expression::UserSelectableExpression, ExpressionTreeAction, TriggerAction,
};

pub trait SimpleTriggerWidgetExtension {
    fn ui(&mut self, ui: &mut egui::Ui);
}

pub trait TriggerWidgetExtension {
    fn ui(&mut self, trigger_id: &Uuid, ui: &mut egui::Ui) -> TriggerAction;
}

pub trait RecursiveTriggerWidgetExtension {
    fn ui(
        &mut self,
        id_base: String,
        last_expression_type: Option<BooleanOperator>,
        ui: &mut egui::Ui,
    ) -> ExpressionTreeAction;
}

impl TriggerWidgetExtension for Trigger {
    fn ui(&mut self, trigger_id: &Uuid, ui: &mut egui::Ui) -> TriggerAction {
        let mut action = TriggerAction::None;
        ui.horizontal(|ui| {
            ui.heading(self.name.clone())
                .context_menu(|ui| {
                    ui.name_input(
                        "Set name",
                        format!("set_trigger_name_{trigger_id}"),
                        |name| {
                            self.name = name;
                        },
                    );

                    ui.separator();
                    if ui.button("Delete").clicked() {
                        action = TriggerAction::Delete;
                        ui.close_menu();
                    }
                })
                .on_hover_text_at_pointer("Right click for options");
            ui.checkbox(&mut self.enabled, "Enabled");
            ui.checkbox(&mut self.requestable, "Requestable")
                .on_hover_text(
                    "If true, this trigger will only run \
                            for users that it is requested to run for. \
                            If false, it will run for all users.",
                );
            ui.horizontal_right(|ui| {
                if ui.button("⬆").clicked() {
                    action = TriggerAction::MoveUp;
                }

                if ui.button("⬇").clicked() {
                    action = TriggerAction::MoveDown;
                }
            });
        });
        ui.collapsing_default_open_with_id("Criteria", format!("{trigger_id}_criteria"), |ui| {
            self.criteria.ui(trigger_id.to_string(), None, ui);
        });
        ui.collapsing_default_open_with_id("Action(s)", format!("{trigger_id}_actions"), |ui| {
            for (i, action) in self.actions.iter_mut().enumerate() {
                action.ui(format!("{trigger_id}_action{i}"), ui);
            }
        });
        ui.separator();

        action
    }
}

impl SimpleTriggerWidgetExtension for ValuePointer {
    fn ui(&mut self, ui: &mut egui::Ui) {
        match self {
            ValuePointer::OnlineStatus(time) => {
                time.ui(ui);
                ui.label(UserSelectableExpression::OnlineStatus.to_string());
            }
            ValuePointer::LockStatus(time) => {
                time.ui(ui);
                ui.label(UserSelectableExpression::LockStatus.to_string());
            }
            ValuePointer::TotalKeyboardMouseUsage(time) => {
                time.ui(ui);
                ui.label(UserSelectableExpression::TotalKeyboardMouseUsage.to_string());
            }
            ValuePointer::NumAppsUsingMicrophone(time) => {
                time.ui(ui);
                ui.label(UserSelectableExpression::NumAppsUsingMicrophone.to_string());
            }
            ValuePointer::UserId => {
                ui.label("UserId");
            }
            ValuePointer::ConstBool(b) => {
                let text = if *b { "true" } else { "false" };
                if ui.small_button(text).clicked() {
                    *b = !*b;
                }
            }
            ValuePointer::ConstUserId(id) => {
                let mut id_str = id.to_string();
                if egui::TextEdit::singleline(&mut id_str)
                    .desired_width(100.0)
                    .show(ui)
                    .response
                    .changed()
                {
                    *id = UniqueUserId::new(id_str);
                }
            }
            ValuePointer::ConstF64(v) => {
                ui.add(egui::DragValue::new(v).speed(1).clamp_range(0.0..=100.0));
            }
            ValuePointer::ConstUsize(v) => {
                ui.add(egui::DragValue::new(v).speed(1).clamp_range(0..=100));
            }
            ValuePointer::TimeSinceMostRecentUpdate => {
                ui.label("Time Since MRU");
            }
            ValuePointer::ConstDuration(d) => {
                show_duration_ui(d, ui);
            }
            ValuePointer::ActiveWindowDuration(t) => {
                t.ui(ui);
                ui.label(UserSelectableExpression::ActiveWindowDuration.to_string());
            }
        }
    }
}

impl SimpleTriggerWidgetExtension for TimeSpecifier {
    fn ui(&mut self, ui: &mut egui::Ui) {
        match self.clone() {
            TimeSpecifier::Last => {
                if ui.small_button("Last").clicked() {
                    *self = TimeSpecifier::Current;
                }
            }
            TimeSpecifier::Current => {
                if ui.small_button("Current").clicked() {
                    *self = TimeSpecifier::Last;
                }
            }
        };
    }
}

impl RecursiveTriggerWidgetExtension for Expression {
    fn ui(
        &mut self,
        mut id_base: String,
        last_expression_type: Option<BooleanOperator>,
        ui: &mut egui::Ui,
    ) -> ExpressionTreeAction {
        id_base = format!("{id_base}_{}", get_expression_id_extension(self));
        type E = Expression;
        let action = match self {
            E::And(l, r) => BooleanOperator::And.ui(l, r, id_base, last_expression_type, ui),
            E::Or(l, r) => BooleanOperator::Or.ui(l, r, id_base, last_expression_type, ui),
            E::Equals(l, r) => ComparisonOperator::Equals.ui(id_base, l, r, ui),
            E::NotEquals(l, r) => ComparisonOperator::NotEquals.ui(id_base, l, r, ui),
            E::GreaterThan(l, r) => ComparisonOperator::GreaterThan.ui(id_base, l, r, ui),
            E::GreaterThanOrEquals(l, r) => {
                ComparisonOperator::GreaterThanOrEquals.ui(id_base, l, r, ui)
            }
            E::LessThan(l, r) => ComparisonOperator::LessThan.ui(id_base, l, r, ui),
            E::LessThanOrEquals(l, r) => ComparisonOperator::LessThanOrEquals.ui(id_base, l, r, ui),
            E::True => {
                ui.label("True");
                ExpressionTreeAction::None
            }
        };

        if let ExpressionTreeAction::Update(new_expression) = action {
            *self = new_expression;
            ExpressionTreeAction::None
        } else {
            action
        }
    }
}

fn get_expression_id_extension(criteria: &mut Expression) -> &str {
    match criteria {
        Expression::And(_, _) => "and",
        Expression::Or(_, _) => "or",
        Expression::Equals(_, _) => "eq",
        Expression::NotEquals(_, _) => "neq",
        Expression::GreaterThan(_, _) => "gt",
        Expression::LessThan(_, _) => "lt",
        Expression::GreaterThanOrEquals(_, _) => "ge",
        Expression::LessThanOrEquals(_, _) => "le",
        Expression::True => "true",
    }
}

fn show_duration_ui(d: &mut Duration, ui: &mut egui::Ui) {
    const SECS_PER_HR: u64 = 3600;
    const SECS_PER_MIN: u64 = 60;
    let mut hours = d.as_secs() / SECS_PER_HR;
    let mut minutes = (d.as_secs() / SECS_PER_MIN) % SECS_PER_MIN;
    let mut secs = d.as_secs() % SECS_PER_MIN;

    ui.horizontal_with_no_item_spacing(|ui| {
        let h_res = ui.add(
            egui::DragValue::new(&mut hours)
                .speed(0.25)
                .clamp_range(0..=1000),
        );
        ui.label("h ");
        let m_res = ui.add(
            egui::DragValue::new(&mut minutes)
                .speed(0.25)
                .clamp_range(0..=59),
        );
        ui.label("m ");
        let s_res = ui.add(
            egui::DragValue::new(&mut secs)
                .speed(0.25)
                .clamp_range(0..=59),
        );
        ui.label("s ");

        if h_res.changed() || m_res.changed() || s_res.changed() {
            *d = Duration::from_secs(hours * SECS_PER_HR + minutes * SECS_PER_MIN + secs);
        }
    });
}
