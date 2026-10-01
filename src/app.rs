use std::sync::mpsc::Receiver;
use std::time::Duration;
use liveplot::{PlotCommand, LivePlotPanel};
use eframe::egui;

//tilstand gui, hva appen tar vare på mellom hver kjøring
pub struct App {
    panel: LivePlotPanel,
}
//kjøres bare ved oppstart
impl App {
    pub fn new(rx:Receiver<PlotCommand>) -> Self {
        //lager en app hvor panelet er dette
        Self{panel: LivePlotPanel::new(rx)}

    }
}
//App kan kjøres som vindu
impl eframe::App for App{
    //kjører hver frame
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame){
        egui::CentralPanel::default().show(ui, |ui| self.panel.update_embedded(ui));
        ui.ctx().request_repaint_after(Duration::from_millis(16));//60fps samme som liveplot sin update


    }

}

