use egui::{
    text::LayoutJob, CollapsingHeader, CollapsingResponse, InnerResponse, RichText, Ui, WidgetText,
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
        let response = self.horizontal(add_contents);

        response
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
}
