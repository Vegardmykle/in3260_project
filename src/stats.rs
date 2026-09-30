use liveplot::PlotPoint;

pub fn cdf_beregning(sortet_v:&[(i64,f64)]) -> Vec<PlotPoint> {
    let n = sortet_v.len();
    let mut punkter = Vec::with_capacity(n);
    //sortert vector med tid og latens
    for (i, (_tid, latens)) in sortet_v.iter().enumerate() {
        //deler på 1 mill for å få det i sek istendefor dager
        punkter.push(PlotPoint {
            x: *latens / 1e6,
            y: ((i + 1) as f64 / n as f64),
        });
    }
    return punkter;
}
