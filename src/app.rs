use std::collections::BTreeMap;
use std::sync::mpsc::Receiver;
use std::time::Duration;
use liveplot::{PlotCommand, LivePlotPanel};
use eframe::egui;
use crate::reader::ParOppdatering;

//tilstand gui, hva appen tar vare på mellom hver kjøring
pub struct App {
    panel: LivePlotPanel,
    rx_stats: Receiver<ParOppdatering>,
    stats_map:BTreeMap<(String,String),ParOppdatering>
}
//kjøres bare ved oppstart
impl App {
    pub fn new(rx:Receiver<PlotCommand>,rx_stats:Receiver<ParOppdatering>) -> Self {
        //lager en app hvor panelet er dette
        Self{panel: LivePlotPanel::new(rx),rx_stats,stats_map:BTreeMap::new()}

    }
}
//App kan kjøres som vindu
impl eframe::App for App{
    //kjører hver frame
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame){
        while let Ok(msg) = self.rx_stats.try_recv() {
            self.stats_map.insert(msg.mac_av_mt.clone(),msg);
        }
        egui::Panel::left("stats").show(ui, |ui| {
            for (key, val) in self.stats_map.iter_mut() {
                ui.label(format!("{} at {} to {}\n \
                P95: {} P99: {} median {} avg {} max {}",
                                 val.trace, key.0, key.1, val.pairstats.p95,val.pairstats.p99,val.pairstats.median,val.pairstats.avg,val.pairstats.max));
            }
        });
        egui::CentralPanel::default().show(ui, |ui| self.panel.update_embedded(ui));
        ui.ctx().request_repaint_after(Duration::from_millis(16));//60fps samme som liveplot sin update


    }

}

