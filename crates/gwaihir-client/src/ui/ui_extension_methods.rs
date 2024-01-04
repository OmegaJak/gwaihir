use egui::{
    text::LayoutJob, CollapsingHeader, CollapsingResponse, InnerResponse, RichText, Ui, Widget,
    WidgetText,
};

pub trait UIExtensionMethods {
    fn collapsing_default_open<R>(
        &mut self,
        heading: impl Into<WidgetText>,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> CollapsingResponse<R>;

    fn horizontal_with_no_item_spacing<R>(
        &mut self,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R>;

    fn create_default_layout_job(&self, rich_texts: Vec<RichText>) -> LayoutJob;

    fn stateless_checkbox(&mut self, checked: bool, text: impl Into<WidgetText>) -> Option<bool>;
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

    fn horizontal_with_no_item_spacing<R>(
        &mut self,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.spacing_mut().item_spacing.x = 0.0;

        self.horizontal(add_contents)
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
}
