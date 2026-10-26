use liveplot::channel_plot;
use std::sync::mpsc::channel;

mod app;
mod reader;
mod stats;

//run with
//% cd csv_streamer && cargo build && cd ..
//cd in3260_project && ../csv_streamer/target/debug/csv_streamer | cargo run

fn main() -> eframe::Result<()> {
    let (sink, rx) = channel_plot();
    let (tx, rx_stats) = channel();
    reader::start(sink.clone(), tx);
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_maximized(true),
        ..Default::default()
    };
    
    eframe::run_native(
        "live CDF",
        options,
        Box::new(|_cc| Ok(Box::new(app::App::new(rx, rx_stats, sink)))),
    )
}
