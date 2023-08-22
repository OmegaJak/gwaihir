use egui::{CollapsingHeader, CollapsingResponse, Ui, WidgetText};

pub trait UIExtensionMethods {
    fn collapsing_default_open<R>(
        &mut self,
        heading: impl Into<WidgetText>,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> CollapsingResponse<R>;
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
}
