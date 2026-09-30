use liveplot::{LivePlotConfig, channel_plot, run_liveplot};
mod app;
mod reader;
mod stats;

//run with
//% cd csv_streamer && cargo build && cd ..
//cd live_cdf && ../csv_streamer/target/debug/csv_streamer | cargo run

fn main() -> eframe::Result<()> {
    let (sink, rx) = channel_plot();
    reader::start(sink);
    run_liveplot(rx, LivePlotConfig::default())
}
