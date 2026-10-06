use crate::stats;
use csv::ReaderBuilder;
use liveplot::{PlotPoint, PlotSink, Trace};
use serde::Deserialize;
use std::collections::{BTreeMap, VecDeque};
use std::io;
use std::sync::mpsc::Sender;
use crate::stats::{stats_beregning, Stats};

const VINDU_US: i64 = 60_000_000;
#[derive(Deserialize)]
struct Row {
    timestamp: i64,
    avsender_mac: String,
    mottaker_mac: String,
    ssid: String,
    latens_us: i32,
}
pub struct Par {
    verdier: VecDeque<(i64, f64)>,
    trace: Trace,
}
pub struct ParOppdatering {
    pub mac_av_mt:(String, String),
    pub trace: Trace,
    pub punkter: Vec<PlotPoint>,
    pub pairstats: Stats,
}
pub fn start(sink: PlotSink,tx:Sender<ParOppdatering>) {
    std::thread::spawn(move || {
        let mut rdr = ReaderBuilder::new()
            .has_headers(false)
            .from_reader(io::stdin());
        //key = (avsender mac, mottaker_mac), Par(verdier(timestamp, latens),trace/sink)
        let mut lat: BTreeMap<(String, String), Par> = BTreeMap::new();
        //timestamp
        let mut nyeste: i64 = 0;
        //timestamp, latens
        let mut sortet_v: Vec<(i64, f64)> = Vec::new();
        //deserialize venter på data
        'lesing: for result in rdr.deserialize::<Row>() {
            match result {
                Ok(row) => {
                    let key: (String, String) = (row.avsender_mac, row.mottaker_mac);
                    nyeste = nyeste.max(row.timestamp);
                    {
                        let par = lat.entry(key).or_insert_with_key(|k| Par {
                            verdier: VecDeque::new(),
                            trace: sink
                                .create_trace(format!("{} at {} -> {}", row.ssid, k.0, k.1), None),
                        });
                        par.verdier.push_back((row.timestamp, row.latens_us as f64));
                    }

                    let grense = nyeste - VINDU_US;
                    for (k,par) in lat.iter_mut() {
                        while let Some(&(t, _)) = par.verdier.front() {
                            if t < grense {
                                par.verdier.pop_front();
                            } else {
                                break;
                            }
                        }
                        par.sort_latens(&mut sortet_v);

                        let punkter = stats::cdf_beregning(&sortet_v);
                        if let Some(stats) = stats_beregning(&sortet_v) {
                            let par_oppdatering: ParOppdatering =ParOppdatering{
                                mac_av_mt: k.clone(),
                                trace: par.trace.clone(),
                                punkter: punkter,
                                pairstats:stats,
                            };
                            if let Err(e) = tx.send(par_oppdatering){
                                eprintln!("Error sending parOppdatering: {:?}", e);
                                break 'lesing;
                            }
                        }

                        //set_data fjerner alle punkter og setter de inn på nytt slipper å fjerne punkter indv fra graf
                        if let Err(e) = sink.set_data(&par.trace, punkter) {
                            eprintln!("{}", e);
                            break 'lesing;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{}", e);
                    continue;
                }
            }
        }
    });
}
impl Par {
    fn sort_latens(&self, buf: &mut Vec<(i64, f64)>) {
        buf.clear();
        buf.extend(self.verdier.iter().copied());
        buf.sort_by(|a, b| a.1.total_cmp(&b.1));
    }
}
