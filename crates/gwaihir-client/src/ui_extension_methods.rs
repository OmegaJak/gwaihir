use egui::{CollapsingHeader, CollapsingResponse, InnerResponse, Ui, WidgetText};

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
}
