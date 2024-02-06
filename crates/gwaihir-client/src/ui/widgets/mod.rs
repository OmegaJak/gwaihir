pub mod auto_launch_checkbox;

/// Shows a window at the center of the screen, while avoiding mutability issues that would come
/// from passing a mutable reference to a field into the `open` method.
///
/// *Returns:* Whether the window should be shown on the next frame
#[must_use = "The resultant bool value indicating whether the window is shown must be stored to allow closing of the window"]
pub fn show_centered_window<R>(
    initial_shown_value: bool,
    window_title: impl Into<egui::WidgetText>,
    ctx: &egui::Context,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> bool {
    let mut shown = initial_shown_value;
    egui::Window::new(window_title)
        .pivot(egui::Align2::CENTER_CENTER)
        .default_pos(ctx.screen_rect().center())
        .open(&mut shown)
        .show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, add_contents);
        });
    shown
}
