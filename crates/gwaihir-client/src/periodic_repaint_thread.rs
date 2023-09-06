use std::{
    sync::mpsc::{channel, Receiver, Sender},
    thread::JoinHandle,
    time::{Duration, Instant},
};

use egui::Context;

// TODO: Once https://github.com/emilk/egui/issues/3109 is resolved, use request_repaint_after in main loop instead
pub fn create_periodic_repaint_thread(
    egui_ctx: Context,
    duration_between_repaints: Duration,
) -> (JoinHandle<()>, Sender<()>) {
    let (shutdown_tx, shutdown_rx) = channel();
    let handle = std::thread::spawn(move || loop {
        match shutdown_rx.try_recv() {
            Err(std::sync::mpsc::TryRecvError::Empty) => (),
            _ => return,
        }

        egui_ctx.request_repaint();
        std::thread::sleep(duration_between_repaints);
    });

    (handle, shutdown_tx)
}
