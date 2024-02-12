use std::fmt::Display;

use egui::{
    text::LayoutJob, Align, CollapsingHeader, CollapsingResponse, Id, InnerResponse, Layout,
    Response, RichText, TextEdit, Ui, Widget, WidgetText,
};

pub trait UIExtensionMethods {
    fn collapsing_default_open<R>(
        &mut self,
        heading: impl Into<WidgetText>,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> CollapsingResponse<R>;

    fn collapsing_default_open_with_id<R>(
        &mut self,
        heading: impl Into<WidgetText>,
        id_source: impl std::hash::Hash,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> CollapsingResponse<R>;

    fn horizontal_with_no_item_spacing<R>(
        &mut self,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R>;

    /// Horizontal with right-aligned contents
    fn horizontal_right<R>(&mut self, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R>;

    fn create_default_layout_job(&self, rich_texts: Vec<RichText>) -> LayoutJob;

    fn stateless_checkbox(&mut self, checked: bool, text: impl Into<WidgetText>) -> Option<bool>;

    fn selectable_value_default_text<Value: PartialEq + Clone + Display>(
        &mut self,
        current_value: &mut Value,
        selected_value: Value,
    ) -> Response;

    fn name_input(
        &mut self,
        button_text: impl Into<egui::WidgetText>,
        id_source: impl std::hash::Hash,
        set_name: impl FnMut(String),
    );
}

impl UIExtensionMethods for Ui {
    fn collapsing_default_open<R>(
        &mut self,
        heading: impl Into<WidgetText>,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> CollapsingResponse<R> {
        CollapsingHeader::new(heading)
            .default_open(true)
            .show(self, add_contents)
    }

    fn collapsing_default_open_with_id<R>(
        &mut self,
        heading: impl Into<WidgetText>,
        id_source: impl std::hash::Hash,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> CollapsingResponse<R> {
        CollapsingHeader::new(heading)
            .default_open(true)
            .id_source(id_source)
            .show(self, add_contents)
    }

    fn horizontal_with_no_item_spacing<R>(
        &mut self,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.spacing_mut().item_spacing.x = 0.0;

        self.horizontal(add_contents)
    }

    fn horizontal_right<R>(&mut self, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let initial_size = self.available_size_before_wrap();
        let layout = Layout::right_to_left(Align::Center);
        self.allocate_ui_with_layout(initial_size, layout, add_contents)
    }

    fn create_default_layout_job(&self, rich_texts: Vec<RichText>) -> LayoutJob {
        let mut layout_job = LayoutJob::default();
        for rich_text in rich_texts.into_iter() {
            rich_text.append_to(
                &mut layout_job,
                self.style(),
                egui::FontSelection::Default,
                egui::Align::Center,
            )
        }

        layout_job
    }

    /// This looks like a checkbox, but the state is assumed to be stored outside the checkbox itself.
    /// Thus, changes must be made externally by watching the `Response`.
    ///
    /// Returns `Some(true)` if the value was changed to a checked state, `Some(false)` if it was changed to an unchecked state,
    /// and `None` if the value was not changed
    fn stateless_checkbox(&mut self, checked: bool, text: impl Into<WidgetText>) -> Option<bool> {
        let mut value = checked;
        if egui::Checkbox::new(&mut value, text).ui(self).changed() {
            return Some(value);
        }

        None
    }

    fn selectable_value_default_text<Value: PartialEq + Clone + Display>(
        &mut self,
        current_value: &mut Value,
        selected_value: Value,
    ) -> Response {
        self.selectable_value(
            current_value,
            selected_value.clone(),
            selected_value.to_string(),
        )
    }

    fn name_input(
        &mut self,
        button_text: impl Into<egui::WidgetText>,
        id_source: impl std::hash::Hash,
        mut set_name: impl FnMut(String),
    ) {
        let id_source = Id::new(id_source);
        let input_id = self.make_persistent_id(id_source);
        let mut name = self.memory_mut(|mem| {
            mem.data
                .get_temp_mut_or_default::<String>(input_id)
                .to_string()
        });

        self.horizontal(|ui| {
            let text_edit_response = TextEdit::singleline(&mut name).desired_width(100.0).ui(ui);
            if text_edit_response.changed() {
                ui.memory_mut(|mem| mem.data.insert_temp(input_id, name.clone()));
            }

            if ui.button(button_text).clicked()
                || (text_edit_response.lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter)))
            {
                set_name(name.clone());
                ui.memory_mut(|mem| mem.data.remove::<String>(input_id));
                ui.close_menu();
            }
        });
    }
}
