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
                    ui.stateless_checkbox(trigger.requestable(), "Requestable");
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
                    Ok(mut criteria) => {
                        if self.requestable_input {
                            criteria = Expression::And(
                                Expression::RequestedForUser.into(),
                                criteria.into(),
                            );
                        }

                        let matcher = Trigger {
                            name: self.name_input.clone(),
                            enabled: self.enabled_input,
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
            ui.label("AND");
            criteria_ui_rec(r, ui, format!("{id_base}_andr"));
            None
        }
        Expression::Or(_, _) => None,
        Expression::Equals(l, r) => {
            let id_base = format!("{id_base}_eq");
            ui.horizontal(|ui| {
                value_pointer_ui(l, ui);
                let mut current = ComparisonOperators::Equals;
                ComboBox::from_id_source(format!("{id_base}_operatorcombobox"))
                    .selected_text(format!("{}", current))
                    .show_ui(ui, |ui| {
                        for operator in enum_iterator::all::<ComparisonOperators>()
                            .filter(|o| o.valid_for_value(l) && o.valid_for_value(r))
                        {
                            ui.selectable_value_default_text(&mut current, operator);
                        }
                    });
                value_pointer_ui(r, ui);

                match current {
                    ComparisonOperators::NotEquals => {
                        Some(Expression::NotEquals(l.clone(), r.clone()))
                    }
                    ComparisonOperators::LessThan => None,
                    ComparisonOperators::LessThanOrEquals => None,
                    ComparisonOperators::GreaterThan => None,
                    ComparisonOperators::GreaterThanOrEquals => None,
                    ComparisonOperators::Equals => None,
                }
            })
            .inner
        }
        Expression::NotEquals(_, _) => None,
        Expression::GreaterThan(_, _) => None,
        Expression::LessThan(_, _) => None,
        Expression::GreaterThanOrEquals(_, _) => None,
        Expression::LessThanOrEquals(_, _) => None,
        Expression::RequestedForUser => None,
    };
    if let Some(new_criteria) = new_criteria {
        *criteria = new_criteria;
    }
}

fn value_pointer_ui(value: &ValuePointer, ui: &mut egui::Ui) {
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
        ValuePointer::UserId => {}
        ValuePointer::ConstBool(b) => {
            if *b {
                ui.label("true");
            } else {
                ui.label("false");
            }
        }
        ValuePointer::ConstUserId(_) => {}
        ValuePointer::ConstF64(v) => {
            ui.label(v.to_string());
        }
    };
}

fn time_specifier_ui(time: &TimeSpecifier, ui: &mut egui::Ui) {
    match time {
        TimeSpecifier::Last => ui.small_button("Last"),
        TimeSpecifier::Current => ui.small_button("Current"),
    };
}

#[derive(Clone, PartialEq, Sequence)]
enum ComparisonOperators {
    Equals,
    NotEquals,
    LessThan,
    LessThanOrEquals,
    GreaterThan,
    GreaterThanOrEquals,
}

enum ValueType {
    Bool,
    UserId,
    F64,
}

impl ComparisonOperators {
    fn valid_for_value(&self, value: &ValuePointer) -> bool {
        let value_type = ValueType::from_pointer(value);
        self.valid_for_value_type(value_type)
    }

    fn valid_for_value_type(&self, value_type: ValueType) -> bool {
        match value_type {
            ValueType::Bool => matches!(
                self,
                ComparisonOperators::Equals | ComparisonOperators::NotEquals
            ),
            ValueType::UserId => matches!(
                self,
                ComparisonOperators::Equals | ComparisonOperators::NotEquals
            ),
            ValueType::F64 => matches!(
                self,
                ComparisonOperators::Equals
                    | ComparisonOperators::NotEquals
                    | ComparisonOperators::GreaterThan
                    | ComparisonOperators::GreaterThanOrEquals
                    | ComparisonOperators::LessThan
                    | ComparisonOperators::LessThanOrEquals
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

impl std::fmt::Display for ComparisonOperators {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComparisonOperators::Equals => write!(f, "="),
            ComparisonOperators::NotEquals => write!(f, "≠"),
            ComparisonOperators::LessThan => write!(f, "<"),
            ComparisonOperators::LessThanOrEquals => write!(f, "<="),
            ComparisonOperators::GreaterThan => write!(f, ">"),
            ComparisonOperators::GreaterThanOrEquals => write!(f, ">="),
        }
    }
}
