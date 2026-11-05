use crate::stats::{ParNokkel, ParOppdatering};
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use liveplot::data::scope::{AxisType, ValueFormat};
use liveplot::{LivePlotPanel, PlotCommand, PlotSink};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::mpsc::Receiver;
use std::time::Duration;
use felles::LatensType;
use liveplot::config::ScopeButton;
use liveplot::TraceRef;

//tilstand gui, hva appen tar vare på mellom hver kjøring
pub struct App {
    panel: LivePlotPanel,
    sink: PlotSink,
    rx_stats: Receiver<ParOppdatering>,
    stats_map: BTreeMap<(ParNokkel,LatensType), ParOppdatering>,
    valgte: BTreeSet<ParNokkel>,
}

fn ms(us: f64) -> String {
    format!("{:.1}", us / 1000.0)
}
//slik at tall står under hverandre med likt mellomrom
fn tall(ui: &mut egui::Ui, tekst: String) {
    ui.monospace(tekst);
}

fn sirkel(ui: &mut egui::Ui, farge: egui::Color32){
    let (rect,_) = ui.allocate_exact_size(egui::vec2(10.0,10.0),egui::Sense::hover());
    ui.painter().circle_filled(rect.center(),5.0,farge);
}

//kjøres bare ved oppstart
impl App {
    pub fn new(
        rx: Receiver<PlotCommand>,
        rx_stats: Receiver<ParOppdatering>,
        sink: PlotSink,
    ) -> Self {
        //lager en app hvor panelet er dette
        let mut panel = LivePlotPanel::new(rx);
        panel.traces_data.max_points = 100_000_00; //lat løsning setter max punknter fra 10_000 til 10_000_000
        for scope in panel.liveplot_panel.get_data_mut() {
            scope.force_hide_legend = true;
            //x er latens i ms, ikke klokkeslett, så vanlig tallakse istedenfor tidsakse
            scope.x_axis.axis_type = AxisType::Value(ValueFormat {
                unit: Some("ms".to_string()),
                ..Default::default()
            });
            scope.x_axis.name = Some("Latens".to_string());
            scope.x_axis.value_decimals = 1;
            scope.y_axis.value_decimals = 2;
            //tilpass aksene til dataene fra start (skrus av ved zoom/pan, "Fit" skrur på igjen)
            scope.x_axis.auto_fit = true;
            scope.y_axis.auto_fit = true;
            panel.top_bar_buttons = Some(
                ScopeButton::all_defaults()
                    .into_iter()
                    .filter(|b| *b != ScopeButton::Traces)
                    .collect(),
            );
        }
        Self {
            panel,
            sink,
            rx_stats,
            stats_map: BTreeMap::new(),
            valgte: BTreeSet::new(),
        }
    }
}
impl App {
    fn vis_tabell(&mut self, ui: &mut egui::Ui) {
        ui.style_mut().interaction.selectable_labels = false;
        let mut klikk: Option<ParNokkel> = None;

        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .sense(egui::Sense::click()) //gjør den trykkbar
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::exact(14.0))//farge
            .column(Column::auto().at_least(140.0)) // Fra
            .column(Column::auto().at_least(140.0)) // Til
            .column(Column::auto().at_least(140.0)) // Type
            .columns(Column::auto().at_least(60.0), 5) // Median, Avg, P95, P99, Max
            .header(22.0, |mut header| {
                for tittel in ["","Fra", "Til","Type", "Median", "Avg", "P95", "P99", "Max"] {
                    header.col(|ui| {
                        ui.strong(tittel);
                    });
                }
            })
            .body(|mut body| {
                for ((par,typ), m) in &self.stats_map {
                    let s = &m.pairstats;
                    let farge = self.panel.traces_data
                        .get_trace(&TraceRef::from(m.trace.name.as_str()))
                        .map(|t| t.look.color);
                    body.row(20.0, |mut row| {
                        row.set_selected(self.valgte.contains(par));
                        row.col(|ui| {
                            if let Some(f) = farge{
                                sirkel(ui, f);
                            }
                        });
                        row.col(|ui| {
                            ui.monospace(&par.0);
                        });
                        row.col(|ui| {
                            ui.monospace(&par.1);
                        });
                        row.col(|ui| tall(ui, format!("{:?}", typ)));                        
                        row.col(|ui| tall(ui, ms(s.median)));
                        row.col(|ui| tall(ui, ms(s.avg)));
                        row.col(|ui| tall(ui, ms(s.p95)));
                        row.col(|ui| tall(ui, ms(s.p99)));
                        row.col(|ui| tall(ui, ms(s.max)));
                        if row.response().clicked() {
                            klikk = Some(par.clone());
                        }
                    });
                }
            });
        //endrer valg (markert ikke markert)
        if let Some(k) = klikk {
            if !self.valgte.remove(&k) {
                self.valgte.insert(k);
            }
            self.oppdater_synlyghet();
        }
    }
    fn er_synlig(&self, k: &ParNokkel) -> bool {
        self.valgte.is_empty() || self.valgte.contains(k)
    }
    fn tegn_par(&self, m: &ParOppdatering) {
        let res = if self.er_synlig(&m.par_mac) {
            self.sink.set_data(&m.trace, m.punkter.clone())
        } else {
            self.sink.clear_data(&m.trace)
        };
        if let Err(e) = res {
            eprintln!("{e}")
        }
    }
    //oppdaterer synligheten slik at den enten vises eller ikke vises for en eller flere traces valgt
    fn oppdater_synlyghet(&self) {
        for m in self.stats_map.values() {
            self.tegn_par(m);
        }
    }
}

//App kan kjøres som vindu
impl eframe::App for App {
    //kjører hver frame
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        while let Ok(msg) = self.rx_stats.try_recv() {
            self.tegn_par(&msg);
            self.stats_map.insert((msg.par_mac.clone(),msg.latens_type), msg);
        }
        egui::Panel::left("stats").show(ui, |ui| self.vis_tabell(ui));
        egui::CentralPanel::default().show(ui, |ui| self.panel.update_embedded(ui));
        ui.ctx().request_repaint_after(Duration::from_millis(16)); //60fps samme som liveplot sin update
    }
}
