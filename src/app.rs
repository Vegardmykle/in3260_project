use std::collections::BTreeMap;
use std::sync::mpsc::Receiver;
use std::time::Duration;
use liveplot::{PlotCommand, LivePlotPanel};
use eframe::egui;
use crate::reader::ParOppdatering;
use egui_extras::{TableBuilder, Column};

//tilstand gui, hva appen tar vare på mellom hver kjøring
pub struct App {
    panel: LivePlotPanel,
    rx_stats: Receiver<ParOppdatering>,
    stats_map:BTreeMap<(String,String),ParOppdatering>,
    valgt: Option<(String, String)>
}
fn ms(us: f64) -> String {
    format!("{:.1}", us / 1000.0)
}
//slik at tall står under hverandre med likt mellomrom
fn tall(ui: &mut egui::Ui, tekst: String) {
        ui.monospace(tekst);
    }
//kjøres bare ved oppstart
impl App {
    pub fn new(rx:Receiver<PlotCommand>,rx_stats:Receiver<ParOppdatering>) -> Self {
        //lager en app hvor panelet er dette
        Self{panel: LivePlotPanel::new(rx),rx_stats,stats_map:BTreeMap::new(),valgt:None}

    }
}
impl App{
    fn vis_tabell(&mut self, ui: &mut egui::Ui) {
        let mut klikk: Option<(String,String)> = None;

        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .sense(egui::Sense::click())
            .cell_layout(egui::Layout::left_to_right(egui::Align::LEFT))
            .column(Column::auto().at_least(140.0))     // Fra
            .column(Column::auto().at_least(140.0))     // Til
            .columns(Column::auto().at_least(60.0), 5)  // Median, Avg, P95, P99, Max
            .header(22.0, |mut header| {
                for tittel in ["Fra", "Til", "Median", "Avg", "P95", "P99", "Max"] {
                    header.col(|ui| { ui.strong(tittel); });
                }
            })
            .body(|mut body| {
                for (nokkel, m) in &self.stats_map {
                    let s = &m.pairstats;
                    body.row(20.0, |mut row| {
                        row.set_selected(self.valgt.as_ref() == Some(nokkel));
                        row.col(|ui| { ui.monospace(&nokkel.0); });
                        row.col(|ui| { ui.monospace(&nokkel.1); });
                        row.col(|ui| tall(ui, ms(s.median)));
                        row.col(|ui| tall(ui, ms(s.avg)));
                        row.col(|ui| tall(ui, ms(s.p95)));
                        row.col(|ui| tall(ui, ms(s.p99)));
                        row.col(|ui| tall(ui, ms(s.max)));
                        if row.response().clicked() {
                            klikk = Some(nokkel.clone());
                        }
                    });
                }
            });
        if let Some(k) = klikk {
            if self.valgt.as_ref() == Some(&k) {
                self.valgt = None;
            } else {
                self.valgt = Some(k);
            }
        }
    }
}
//App kan kjøres som vindu
impl eframe::App for App{
    //kjører hver frame
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame){
        while let Ok(msg) = self.rx_stats.try_recv() {
            self.stats_map.insert(msg.mac_av_mt.clone(),msg);
        }
        egui::Panel::left("stats").show(ui, |ui| self.vis_tabell(ui));
        egui::CentralPanel::default().show(ui, |ui| self.panel.update_embedded(ui));
        ui.ctx().request_repaint_after(Duration::from_millis(16));//60fps samme som liveplot sin update


    }

}

