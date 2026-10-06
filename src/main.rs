use std::sync::mpsc::channel;
use liveplot::{channel_plot};

mod app;
mod reader;
mod stats;

//run with
//% cd csv_streamer && cargo build && cd ..
//cd live_cdf && ../csv_streamer/target/debug/csv_streamer | cargo run

fn main() -> eframe::Result<()> {
    let (sink, rx) = channel_plot();
    let (tx,rx_stats)= channel();
    reader::start(sink.clone(),tx);
    eframe::run_native("live CDF",eframe::NativeOptions::default(),Box::new(|_cc| Ok(Box::new(app::App::new(rx,rx_stats,sink)))))
}
