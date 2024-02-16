use crate::{
    triggers::{
        value_pointer::{ValueKind, ValuePointerKind},
        Expression, ExpressionRef, TimeSpecifier, ValuePointer,
    },
    ui::ui_extension_methods::UIExtensionMethods,
};
use enum_iterator::Sequence;
use gwaihir_client_lib::UniqueUserId;

use super::{
    user_selectable_expression::UserSelectableExpression, ExpressionTreeAction,
    SimpleTriggerWidgetExtension,
};

#[derive(Clone, PartialEq, Sequence)]
pub enum ComparisonOperator {
    GreaterThan,
    LessThan,
    Equals,
    GreaterThanOrEquals,
    LessThanOrEquals,
    NotEquals,
}

impl ComparisonOperator {
    pub fn valid_for_value(&self, value: &ValuePointer) -> bool {
        let value_type = ValueKind::from(value);
        self.valid_for_value_type(value_type)
    }

    pub fn valid_for_value_type(&self, value_type: ValueKind) -> bool {
        match value_type {
            ValueKind::Bool => matches!(
                self,
                ComparisonOperator::Equals | ComparisonOperator::NotEquals
            ),
            ValueKind::UserId => matches!(
                self,
                ComparisonOperator::Equals | ComparisonOperator::NotEquals
            ),
            ValueKind::F64 => self.is_numeric_operator(),
            ValueKind::Usize => self.is_numeric_operator(),
            ValueKind::Duration => self.is_numeric_operator(),
        }
    }

    fn is_numeric_operator(&self) -> bool {
        matches!(
            self,
            ComparisonOperator::Equals
                | ComparisonOperator::NotEquals
                | ComparisonOperator::GreaterThan
                | ComparisonOperator::GreaterThanOrEquals
                | ComparisonOperator::LessThan
                | ComparisonOperator::LessThanOrEquals
        )
    }

    pub fn create_expression(&self, l: ValuePointer, r: ValuePointer) -> Expression {
        let new_expression = match self {
            ComparisonOperator::GreaterThan => Expression::GreaterThan,
            ComparisonOperator::LessThan => Expression::LessThan,
            ComparisonOperator::Equals => Expression::Equals,
            ComparisonOperator::GreaterThanOrEquals => Expression::GreaterThanOrEquals,
            ComparisonOperator::LessThanOrEquals => Expression::LessThanOrEquals,
            ComparisonOperator::NotEquals => Expression::NotEquals,
        };

        new_expression(l, r)
    }

    pub fn to_expression(&self, left: ValuePointer, right: ValuePointer) -> Expression {
        match self {
            ComparisonOperator::Equals => Expression::Equals(left, right),
            ComparisonOperator::NotEquals => Expression::NotEquals(left, right),
            ComparisonOperator::LessThan => Expression::LessThan(left, right),
            ComparisonOperator::LessThanOrEquals => Expression::LessThanOrEquals(left, right),
            ComparisonOperator::GreaterThan => Expression::GreaterThan(left, right),
            ComparisonOperator::GreaterThanOrEquals => Expression::GreaterThanOrEquals(left, right),
        }
    }

    pub fn ui(
        &self,
        id_base: String,
        left_value: &mut ValuePointer,
        right_value: &mut ValuePointer,
        ui: &mut egui::Ui,
    ) -> ExpressionTreeAction {
        ui.horizontal(|ui| {
            left_value.ui(ui);
            let mut current = self.clone();
            current.selector_ui(ui, left_value, right_value, id_base);
            right_value.ui(ui);

            ui.separator();
            if let Some(action) = self.actions_menu_ui(left_value, right_value, ui) {
                return action;
            }

            if *self == current {
                ExpressionTreeAction::None
            } else {
                ExpressionTreeAction::Update(
                    current.to_expression(left_value.clone(), right_value.clone()),
                )
            }
        })
        .inner
    }

    fn selector_ui(
        &mut self,
        ui: &mut egui::Ui,
        left_value: &mut ValuePointer,
        right_value: &mut ValuePointer,
        id_base: String,
    ) {
        let valid_operators: Vec<_> = enum_iterator::all::<ComparisonOperator>()
            .filter(|o| o.valid_for_value(left_value) && o.valid_for_value(right_value))
            .collect();
        if valid_operators.len() == 2 {
            if ui.button(self.to_string()).clicked() {
                *self = valid_operators
                .iter()
                .find(|o| *o != self)
                .expect("When there are only two operators in a vec of unique operators, one should be different from the current")
                .to_owned();
            }
        } else {
            egui::ComboBox::from_id_source(format!("{id_base}_operatorcombobox"))
                .selected_text(format!("{}", self))
                .width(10.0)
                .show_ui(ui, |ui| {
                    for operator in valid_operators {
                        ui.selectable_value_default_text(self, operator);
                    }
                });
        }
    }

    fn actions_menu_ui(
        &self,
        left_value: &mut ValuePointer,
        right_value: &mut ValuePointer,
        ui: &mut egui::Ui,
    ) -> Option<ExpressionTreeAction> {
        ui.menu_button("...", |ui| {
            let add_condition_response = self.add_condition_button_ui(left_value, right_value, ui);
            if add_condition_response.is_some() {
                return add_condition_response;
            }

            ui.menu_button("Swap Value", |ui| {
                ui.menu_button("Left", |ui| {
                    show_swap_value_ui(left_value, ui);
                });

                ui.menu_button("Right", |ui| {
                    show_swap_value_ui(right_value, ui);
                });
            });

            ui.separator();
            if ui.button("Delete").clicked() {
                ui.close_menu();
                return Some(ExpressionTreeAction::Remove);
            }

            None
        })
        .inner
        .flatten()
    }

    fn add_condition_button_ui(
        &self,
        left_value: &mut ValuePointer,
        right_value: &mut ValuePointer,
        ui: &mut egui::Ui,
    ) -> Option<ExpressionTreeAction> {
        ui.menu_button("Add Condition", |ui| {
            for addable in enum_iterator::all::<UserSelectableExpression>() {
                if ui.button(addable.clone().to_string()).clicked() {
                    let recreated_existing_expression =
                        self.create_expression(left_value.to_owned(), right_value.to_owned());
                    let selected_expression = addable.get_default();
                    let new_expression = Expression::And(
                        ExpressionRef::new(recreated_existing_expression),
                        ExpressionRef::new(selected_expression),
                    );
                    ui.close_menu();
                    return Some(ExpressionTreeAction::Update(new_expression));
                }
            }

            None
        })
        .inner
        .flatten()
    }
}

fn show_swap_value_ui(value: &mut ValuePointer, ui: &mut egui::Ui) {
    let current_kind = ValueKind::from(&value.kind());
    let value_kind = value.kind();
    for valid_replacement in ValuePointerKind::all()
        .iter()
        .filter(|k| **k != value_kind && ValueKind::from(*k) == current_kind)
    {
        if ui.button(valid_replacement.get_display_text()).clicked() {
            *value = valid_replacement.get_default_value_pointer();
            ui.close_menu();
        }
    }
}

impl From<&ValuePointer> for ValueKind {
    fn from(value: &ValuePointer) -> Self {
        ValueKind::from(&value.kind())
    }
}

impl From<&ValuePointerKind> for ValueKind {
    fn from(value: &ValuePointerKind) -> Self {
        match value {
            ValuePointerKind::OnlineStatus
            | ValuePointerKind::LockStatus
            | ValuePointerKind::ConstBool => ValueKind::Bool,
            ValuePointerKind::TotalKeyboardMouseUsage | ValuePointerKind::ConstF64 => {
                ValueKind::F64
            }
            ValuePointerKind::UserId | ValuePointerKind::ConstUserId => ValueKind::UserId,
            ValuePointerKind::NumAppsUsingMicrophone | ValuePointerKind::ConstUsize => {
                ValueKind::Usize
            }
            ValuePointerKind::TimeSinceMostRecentUpdate
            | ValuePointerKind::ConstDuration
            | ValuePointerKind::ActiveWindowDuration => ValueKind::Duration,
        }
    }
}

impl ValuePointerKind {
    pub fn get_default_value_pointer(&self) -> ValuePointer {
        let time = TimeSpecifier::Current;
        match self {
            ValuePointerKind::OnlineStatus => ValuePointer::OnlineStatus(time),
            ValuePointerKind::LockStatus => ValuePointer::LockStatus(time),
            ValuePointerKind::TotalKeyboardMouseUsage => {
                ValuePointer::TotalKeyboardMouseUsage(time)
            }
            ValuePointerKind::NumAppsUsingMicrophone => ValuePointer::NumAppsUsingMicrophone(time),
            ValuePointerKind::UserId => ValuePointer::UserId,
            ValuePointerKind::ConstBool => ValuePointer::ConstBool(true),
            ValuePointerKind::ConstUserId => ValuePointer::ConstUserId(UniqueUserId::new("")),
            ValuePointerKind::ConstF64 => ValuePointer::ConstF64(0.0),
            ValuePointerKind::ConstUsize => ValuePointer::ConstUsize(0),
            ValuePointerKind::TimeSinceMostRecentUpdate => ValuePointer::TimeSinceMostRecentUpdate,
            ValuePointerKind::ConstDuration => {
                ValuePointer::ConstDuration(std::time::Duration::from_secs(30))
            }
            ValuePointerKind::ActiveWindowDuration => ValuePointer::ActiveWindowDuration(time),
        }
    }

    fn get_display_text(&self) -> String {
        match self {
            ValuePointerKind::OnlineStatus => UserSelectableExpression::OnlineStatus.to_string(),
            ValuePointerKind::LockStatus => UserSelectableExpression::LockStatus.to_string(),
            ValuePointerKind::TotalKeyboardMouseUsage => {
                UserSelectableExpression::TotalKeyboardMouseUsage.to_string()
            }
            ValuePointerKind::NumAppsUsingMicrophone => {
                UserSelectableExpression::NumAppsUsingMicrophone.to_string()
            }
            ValuePointerKind::UserId => UserSelectableExpression::UserId.to_string(),
            ValuePointerKind::TimeSinceMostRecentUpdate => {
                UserSelectableExpression::TimeSinceMostRecentUpdate.to_string()
            }
            ValuePointerKind::ActiveWindowDuration => {
                UserSelectableExpression::ActiveWindowDuration.to_string()
            }
            ValuePointerKind::ConstBool
            | ValuePointerKind::ConstUserId
            | ValuePointerKind::ConstF64
            | ValuePointerKind::ConstUsize
            | ValuePointerKind::ConstDuration => "Fixed Value".to_owned(),
        }
    }
}

impl std::fmt::Display for ComparisonOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComparisonOperator::Equals => write!(f, "="),
            ComparisonOperator::NotEquals => write!(f, "≠"),
            ComparisonOperator::LessThan => write!(f, "<"),
            ComparisonOperator::LessThanOrEquals => write!(f, "≤"),
            ComparisonOperator::GreaterThan => write!(f, ">"),
            ComparisonOperator::GreaterThanOrEquals => write!(f, "≥"),
        }
    }
}
