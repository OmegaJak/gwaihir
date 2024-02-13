use crate::{
    triggers::{Expression, ExpressionRef, ValuePointer},
    ui::ui_extension_methods::UIExtensionMethods,
};
use enum_iterator::Sequence;

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

pub enum ValueType {
    Bool,
    UserId,
    F64,
    Usize,
}

impl ComparisonOperator {
    pub fn valid_for_value(&self, value: &ValuePointer) -> bool {
        let value_type = ValueType::from(value);
        self.valid_for_value_type(value_type)
    }

    pub fn valid_for_value_type(&self, value_type: ValueType) -> bool {
        match value_type {
            ValueType::Bool => matches!(
                self,
                ComparisonOperator::Equals | ComparisonOperator::NotEquals
            ),
            ValueType::UserId => matches!(
                self,
                ComparisonOperator::Equals | ComparisonOperator::NotEquals
            ),
            ValueType::F64 => self.is_numeric_operator(),
            ValueType::Usize => self.is_numeric_operator(),
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

impl From<&ValuePointer> for ValueType {
    fn from(value: &ValuePointer) -> Self {
        match value {
            ValuePointer::OnlineStatus(_)
            | ValuePointer::LockStatus(_)
            | ValuePointer::ConstBool(_) => ValueType::Bool,
            ValuePointer::TotalKeyboardMouseUsage(_) | ValuePointer::ConstF64(_) => ValueType::F64,
            ValuePointer::UserId | ValuePointer::ConstUserId(_) => ValueType::UserId,
            ValuePointer::NumAppsUsingMicrophone(_) | ValuePointer::ConstUsize(_) => {
                ValueType::Usize
            }
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
