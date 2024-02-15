use super::Expression;

mod action_widget_extensions;
mod boolean_operator;
mod comparison_operator;
mod text_template_extensions;
mod trigger_widget_extensions;
mod triggers_window;
mod user_selectable_expression;

use trigger_widget_extensions::SimpleTriggerWidgetExtension;
pub use triggers_window::TriggersWindow;

#[derive(Debug)]
enum ExpressionTreeAction {
    None,
    Remove,
    Update(Expression),
}

enum TriggerAction {
    None,
    MoveUp,
    MoveDown,
    Delete,
}
