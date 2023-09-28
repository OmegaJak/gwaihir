use std::{thread::JoinHandle, time::Duration};

use egui::Context;

// TODO: Once https://github.com/emilk/egui/issues/3109 is resolved, use request_repaint_after in main loop instead
pub fn create_periodic_repaint_thread(
    egui_ctx: Context,
    duration_between_repaints: Duration,
) -> JoinHandle<()> {
    std::thread::spawn(move || loop {
        egui_ctx.request_repaint();
        std::thread::sleep(duration_between_repaints);
    })
}
