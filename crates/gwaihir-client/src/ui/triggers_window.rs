use super::{ui_extension_methods::UIExtensionMethods, widgets::show_centered_window};
use crate::triggers::{
    Action, Expression, ExpressionRef, NotificationTemplate, TimeSpecifier, Trigger,
    TriggerManager, TriggerSource, ValuePointer,
};
use egui::{Color32, ComboBox};
use enum_iterator::Sequence;
use gwaihir_client_lib::UniqueUserId;

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
                show_add_condition_ui(&mut trigger.criteria, ui);
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

enum ExpressionTreeAction {
    None,
    RemoveNode,
    UpdateNode(Expression),
}

fn criteria_ui_rec(
    criteria: &mut Expression,
    ui: &mut egui::Ui,
    id_base: String,
) -> ExpressionTreeAction {
    let action = match criteria {
        Expression::And(l, r) => {
            show_binary_boolean_ui(l, r, format!("{id_base}_and"), "AND", Expression::Or, ui)
        }
        Expression::Or(l, r) => {
            show_binary_boolean_ui(l, r, format!("{id_base}_or"), "OR", Expression::And, ui)
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
            ExpressionTreeAction::None
        }
    };

    if let ExpressionTreeAction::UpdateNode(new_expression) = action {
        *criteria = new_expression;
        ExpressionTreeAction::None
    } else {
        action
    }
}

fn show_binary_boolean_ui(
    l: &mut ExpressionRef,
    r: &mut ExpressionRef,
    id_base: String,
    button_text: impl Into<egui::WidgetText>,
    convert_to_other: impl FnOnce(ExpressionRef, ExpressionRef) -> Expression,
    ui: &mut egui::Ui,
) -> ExpressionTreeAction {
    let left_action = criteria_ui_rec(l, ui, format!("{id_base}_l"));
    let button_clicked = ui.button(button_text).clicked();
    let right_action = criteria_ui_rec(r, ui, format!("{id_base}_r"));

    if button_clicked {
        ExpressionTreeAction::UpdateNode(convert_to_other(l.to_owned(), r.to_owned()))
    } else if let ExpressionTreeAction::RemoveNode = left_action {
        ExpressionTreeAction::UpdateNode(r.as_ref().to_owned())
    } else if let ExpressionTreeAction::RemoveNode = right_action {
        ExpressionTreeAction::UpdateNode(l.as_ref().to_owned())
    } else {
        ExpressionTreeAction::None
    }
}

fn show_operator_ui(
    operator: ComparisonOperator,
    id_base: String,
    left_value: &mut ValuePointer,
    right_value: &mut ValuePointer,
    ui: &mut egui::Ui,
) -> ExpressionTreeAction {
    ui.horizontal(|ui| {
        value_pointer_ui(left_value, ui);
        let mut current = operator.clone();
        show_operator_selector_ui(&mut current, ui, left_value, right_value, id_base);
        value_pointer_ui(right_value, ui);

        if ui.button("x").clicked() {
            return ExpressionTreeAction::RemoveNode;
        }

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
        .width(10.0)
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
) -> ExpressionTreeAction {
    if selected_operator == current_operator {
        return ExpressionTreeAction::None;
    }

    let left = left_value.clone();
    let right = right_value.clone();
    ExpressionTreeAction::UpdateNode(match selected_operator {
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
            let mut id_str = id.to_string();
            if ui.text_edit_singleline(&mut id_str).changed() {
                *id = UniqueUserId::new(id_str);
            }
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

fn show_add_condition_ui(criteria: &mut Expression, ui: &mut egui::Ui) {
    ui.menu_button("Add condition", |ui| {
        for addable in enum_iterator::all::<AddableExpression>() {
            if ui.button(addable.clone().to_string()).clicked() {
                let starting_expression = addable.get_default();
                let last_expression = criteria.get_left_to_right_dfs_last_expr_mut();
                *last_expression = Expression::And(
                    ExpressionRef::new(last_expression.to_owned()),
                    ExpressionRef::new(starting_expression),
                );
            }
        }
    });
}

#[derive(Clone, PartialEq, Sequence)]
enum AddableExpression {
    OnlineStatus,
    LockStatus,
    TotalKeyboardMouseUsage,
    UserId,
}

impl AddableExpression {
    fn get_default(&self) -> Expression {
        match self {
            AddableExpression::OnlineStatus => Expression::Equals(
                ValuePointer::OnlineStatus(TimeSpecifier::Current),
                ValuePointer::ConstBool(true),
            ),
            AddableExpression::LockStatus => Expression::Equals(
                ValuePointer::LockStatus(TimeSpecifier::Current),
                ValuePointer::ConstBool(true),
            ),
            AddableExpression::TotalKeyboardMouseUsage => Expression::Equals(
                ValuePointer::TotalKeyboardMouseUsage(TimeSpecifier::Current),
                ValuePointer::ConstF64(0.0),
            ),
            AddableExpression::UserId => Expression::Equals(
                ValuePointer::UserId,
                ValuePointer::ConstUserId(UniqueUserId::new("")),
            ),
        }
    }
}

impl std::fmt::Display for AddableExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddableExpression::OnlineStatus => write!(f, "Online Status"),
            AddableExpression::LockStatus => write!(f, "Lock Status"),
            AddableExpression::TotalKeyboardMouseUsage => write!(f, "Total KB/M Usage"),
            AddableExpression::UserId => write!(f, "User Id"),
        }
    }
}
