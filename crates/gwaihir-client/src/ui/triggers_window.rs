use super::{ui_extension_methods::UIExtensionMethods, widgets::show_centered_window};
use crate::triggers::{
    Action, Expression, NotificationTemplate, TimeSpecifier, Trigger, TriggerManager,
    TriggerSource, ValuePointer,
};
use egui::{Color32, ComboBox};
use enum_iterator::Sequence;

pub struct TriggersWindow {
    shown: bool,
    name_input: String,
    criteria_input: String,
    notification_summary_input: String,
    notification_body_input: String,
    enabled_input: bool,
    requestable_input: bool,
    err: Option<String>,
}

impl TriggersWindow {
    pub fn new() -> Self {
        Self {
            shown: false,
            name_input: Default::default(),
            criteria_input: Default::default(),
            notification_summary_input: Default::default(),
            notification_body_input: Default::default(),
            enabled_input: true,
            requestable_input: false,
            err: None,
        }
    }

    pub fn set_shown(&mut self, shown: bool) {
        self.shown = shown;
    }

    pub fn show(&mut self, ctx: &egui::Context, change_matcher: &mut TriggerManager) {
        self.shown = show_centered_window(self.shown, "Triggers", ctx, |ui| {
            ui.heading("First");
            for (id, trigger) in change_matcher.triggers_iter_mut() {
                ui.horizontal(|ui| {
                    ui.label(trigger.name.clone());
                    ui.checkbox(&mut trigger.requestable, "Requestable");
                });
                criteria_ui_rec(&mut trigger.criteria, ui, id.to_string());
                ui.separator();
            }

            ui.heading("Current");
            egui::Grid::new("existing_notifications")
                .num_columns(1)
                .striped(true)
                .show(ui, |ui| {
                    for (id, serialized_matcher_criteria) in
                        change_matcher.get_serialized_triggers()
                    {
                        ui.horizontal(|ui| {
                            let mut tmp = serialized_matcher_criteria.clone();
                            ui.text_edit_multiline(&mut tmp);

                            if ui.button("X").clicked() {
                                change_matcher.remove_trigger_by_id(&id);
                            }
                        });
                        ui.end_row();
                    }
                });

            ui.heading("Add new ");
            ui.horizontal(|ui| {
                ui.label("Name: ");
                egui::TextEdit::singleline(&mut self.name_input)
                    .desired_width(f32::INFINITY)
                    .show(ui);
            });
            ui.horizontal(|ui| {
                ui.label("Criteria: ");
                egui::TextEdit::singleline(&mut self.criteria_input)
                    .desired_width(f32::INFINITY)
                    .show(ui);
            });
            ui.horizontal(|ui| {
                ui.label("Notif Summary: ");
                egui::TextEdit::singleline(&mut self.notification_summary_input)
                    .desired_width(f32::INFINITY)
                    .show(ui);
            });
            ui.horizontal(|ui| {
                ui.label("Notif Body: ");
                ui.text_edit_multiline(&mut self.notification_body_input);
            });
            ui.checkbox(&mut self.enabled_input, "Enabled");
            ui.checkbox(&mut self.requestable_input, "Requestable");
            if ui.button("Add").clicked() {
                match ron::from_str::<Expression>(&self.criteria_input) {
                    Ok(criteria) => {
                        let matcher = Trigger {
                            name: self.name_input.clone(),
                            enabled: self.enabled_input,
                            requestable: self.requestable_input,
                            criteria,
                            source: TriggerSource::User,
                            actions: vec![Action::ShowNotification(NotificationTemplate::new(
                                self.notification_summary_input.clone(),
                                self.notification_body_input.clone(),
                            ))],
                            requested_users: Default::default(),
                        };
                        change_matcher.add_trigger(matcher);
                        self.criteria_input.clear();
                    }
                    Err(err) => self.err = Some(err.to_string()),
                }
            }

            ui.separator();
            if ui.button("Reset default triggers").clicked() {
                change_matcher.reset_default_triggers();
            }

            if let Some(err) = &self.err {
                ui.separator();
                ui.colored_label(Color32::RED, err);
            }
        });
    }
}

fn criteria_ui_rec(criteria: &mut Expression, ui: &mut egui::Ui, id_base: String) {
    let new_criteria = match criteria {
        Expression::And(l, r) => {
            criteria_ui_rec(l, ui, format!("{id_base}_andl"));
            let mut button_clicked = false;
            if ui.button("AND").clicked() {
                button_clicked = true;
            }
            criteria_ui_rec(r, ui, format!("{id_base}_andr"));

            if button_clicked {
                Some(Expression::Or(l.clone(), r.clone()))
            } else {
                None
            }
        }
        Expression::Or(l, r) => {
            criteria_ui_rec(l, ui, format!("{id_base}_orl"));
            let button_clicked = ui.button("OR").clicked();
            criteria_ui_rec(r, ui, format!("{id_base}_orr"));

            if button_clicked {
                Some(Expression::And(l.clone(), r.clone()))
            } else {
                None
            }
        }
        Expression::Equals(l, r) => show_operator_ui(
            ComparisonOperator::Equals,
            format!("{id_base}_eq"),
            l,
            r,
            ui,
        ),
        Expression::NotEquals(l, r) => show_operator_ui(
            ComparisonOperator::NotEquals,
            format!("{id_base}_neq"),
            l,
            r,
            ui,
        ),
        Expression::GreaterThan(l, r) => show_operator_ui(
            ComparisonOperator::GreaterThan,
            format!("{id_base}_gt"),
            l,
            r,
            ui,
        ),
        Expression::GreaterThanOrEquals(l, r) => show_operator_ui(
            ComparisonOperator::GreaterThanOrEquals,
            format!("{id_base}_ge"),
            l,
            r,
            ui,
        ),
        Expression::LessThan(l, r) => show_operator_ui(
            ComparisonOperator::LessThan,
            format!("{id_base}_lt"),
            l,
            r,
            ui,
        ),
        Expression::LessThanOrEquals(l, r) => show_operator_ui(
            ComparisonOperator::LessThanOrEquals,
            format!("{id_base}_le"),
            l,
            r,
            ui,
        ),
        Expression::True => {
            ui.label("True");
            None
        }
    };
    if let Some(new_criteria) = new_criteria {
        *criteria = new_criteria;
    }
}

fn show_operator_ui(
    operator: ComparisonOperator,
    id_base: String,
    left_value: &mut ValuePointer,
    right_value: &mut ValuePointer,
    ui: &mut egui::Ui,
) -> Option<Expression> {
    ui.horizontal(|ui| {
        value_pointer_ui(left_value, ui);
        let mut current = operator.clone();
        show_operator_selector_ui(&mut current, ui, left_value, right_value, id_base);
        value_pointer_ui(right_value, ui);

        get_updated_expression(operator, current, left_value, right_value)
    })
    .inner
}

fn show_operator_selector_ui(
    current_operator: &mut ComparisonOperator,
    ui: &mut egui::Ui,
    left_value: &mut ValuePointer,
    right_value: &mut ValuePointer,
    id_base: String,
) {
    ComboBox::from_id_source(format!("{id_base}_operatorcombobox"))
        .selected_text(format!("{}", current_operator))
        .show_ui(ui, |ui| {
            for operator in enum_iterator::all::<ComparisonOperator>()
                .filter(|o| o.valid_for_value(left_value) && o.valid_for_value(right_value))
            {
                ui.selectable_value_default_text(current_operator, operator);
            }
        });
}

fn get_updated_expression(
    current_operator: ComparisonOperator,
    selected_operator: ComparisonOperator,
    left_value: &mut ValuePointer,
    right_value: &mut ValuePointer,
) -> Option<Expression> {
    if selected_operator == current_operator {
        return None;
    }

    let left = left_value.clone();
    let right = right_value.clone();
    Some(match selected_operator {
        ComparisonOperator::Equals => Expression::Equals(left, right),
        ComparisonOperator::NotEquals => Expression::NotEquals(left, right),
        ComparisonOperator::LessThan => Expression::LessThan(left, right),
        ComparisonOperator::LessThanOrEquals => Expression::LessThanOrEquals(left, right),
        ComparisonOperator::GreaterThan => Expression::GreaterThan(left, right),
        ComparisonOperator::GreaterThanOrEquals => Expression::GreaterThanOrEquals(left, right),
    })
}

fn value_pointer_ui(value: &mut ValuePointer, ui: &mut egui::Ui) {
    match value {
        ValuePointer::OnlineStatus(time) => {
            time_specifier_ui(time, ui);
            ui.label("Online Status");
        }
        ValuePointer::LockStatus(time) => {
            time_specifier_ui(time, ui);
            ui.label("Lock Status");
        }
        ValuePointer::TotalKeyboardMouseUsage(time) => {
            time_specifier_ui(time, ui);
            ui.label("Total KB/M Usage");
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
            ui.label(id.to_string());
        }
        ValuePointer::ConstF64(v) => {
            ui.add(egui::DragValue::new(v).speed(1).clamp_range(0.0..=100.0));
        }
    };
}

fn time_specifier_ui(time: &mut TimeSpecifier, ui: &mut egui::Ui) {
    match time.clone() {
        TimeSpecifier::Last => {
            if ui.small_button("Last").clicked() {
                *time = TimeSpecifier::Current;
            }
        }
        TimeSpecifier::Current => {
            if ui.small_button("Current").clicked() {
                *time = TimeSpecifier::Last;
            }
        }
    };
}

#[derive(Clone, PartialEq, Sequence)]
enum ComparisonOperator {
    GreaterThan,
    LessThan,
    Equals,
    GreaterThanOrEquals,
    LessThanOrEquals,
    NotEquals,
}

enum ValueType {
    Bool,
    UserId,
    F64,
}

impl ComparisonOperator {
    fn valid_for_value(&self, value: &ValuePointer) -> bool {
        let value_type = ValueType::from_pointer(value);
        self.valid_for_value_type(value_type)
    }

    fn valid_for_value_type(&self, value_type: ValueType) -> bool {
        match value_type {
            ValueType::Bool => matches!(
                self,
                ComparisonOperator::Equals | ComparisonOperator::NotEquals
            ),
            ValueType::UserId => matches!(
                self,
                ComparisonOperator::Equals | ComparisonOperator::NotEquals
            ),
            ValueType::F64 => matches!(
                self,
                ComparisonOperator::Equals
                    | ComparisonOperator::NotEquals
                    | ComparisonOperator::GreaterThan
                    | ComparisonOperator::GreaterThanOrEquals
                    | ComparisonOperator::LessThan
                    | ComparisonOperator::LessThanOrEquals
            ),
        }
    }
}

impl ValueType {
    fn from_pointer(value_pointer: &ValuePointer) -> Self {
        match value_pointer {
            ValuePointer::OnlineStatus(_)
            | ValuePointer::LockStatus(_)
            | ValuePointer::ConstBool(_) => ValueType::Bool,
            ValuePointer::TotalKeyboardMouseUsage(_) | ValuePointer::ConstF64(_) => ValueType::F64,
            ValuePointer::UserId | ValuePointer::ConstUserId(_) => ValueType::UserId,
        }
    }
}

impl std::fmt::Display for ComparisonOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComparisonOperator::Equals => write!(f, "="),
            ComparisonOperator::NotEquals => write!(f, "≠"),
            ComparisonOperator::LessThan => write!(f, "<"),
            ComparisonOperator::LessThanOrEquals => write!(f, "<="),
            ComparisonOperator::GreaterThan => write!(f, ">"),
            ComparisonOperator::GreaterThanOrEquals => write!(f, ">="),
        }
    }
}
