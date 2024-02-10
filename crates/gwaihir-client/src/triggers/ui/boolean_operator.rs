use crate::triggers::{Expression, ExpressionRef};

use super::{trigger_widget_extensions::RecursiveTriggerWidgetExtension, ExpressionTreeAction};

#[derive(Clone, Copy, PartialEq)]
pub enum BooleanOperator {
    And,
    Or,
}

impl BooleanOperator {
    pub fn ui(
        self,
        l: &mut ExpressionRef,
        r: &mut ExpressionRef,
        id_base: String,
        last_expression_type: Option<BooleanOperator>,
        ui: &mut egui::Ui,
    ) -> ExpressionTreeAction {
        let button_text = match self {
            BooleanOperator::And => "AND",
            BooleanOperator::Or => "OR",
        };
        let mut f = |ui: &mut egui::Ui| {
            let left_action = l.ui(format!("{id_base}_l"), Some(self), ui);
            let button_clicked = ui.button(button_text).clicked();
            let right_action = r.ui(format!("{id_base}_r"), Some(self), ui);

            if button_clicked {
                ExpressionTreeAction::Update(match self {
                    BooleanOperator::And => Expression::Or(l.to_owned(), r.to_owned()),
                    BooleanOperator::Or => Expression::And(l.to_owned(), r.to_owned()),
                })
            } else if let ExpressionTreeAction::Remove = left_action {
                ExpressionTreeAction::Update(r.as_ref().to_owned())
            } else if let ExpressionTreeAction::Remove = right_action {
                ExpressionTreeAction::Update(l.as_ref().to_owned())
            } else {
                ExpressionTreeAction::None
            }
        };

        if last_expression_type.is_some_and(|e| e != self) {
            ui.indent(format!("{id_base}_indent"), f).inner
        } else {
            f(ui)
        }
    }
}
