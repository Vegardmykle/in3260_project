use crate::stats;
use csv::ReaderBuilder;
use liveplot::{PlotPoint, PlotSink, Trace};
use serde::Deserialize;
use std::collections::{BTreeMap, VecDeque};
use std::io;
use std::sync::mpsc::Sender;
use crate::stats::{stats_beregning, Stats};
use std::time::{Duration, Instant};


const INTERVAL: Duration = Duration::from_millis(100);
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
    endret:bool,
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
        let mut sist_beregnet: Option<Instant> = None;
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
                            endret: true,
                        });
                        par.verdier.push_back((row.timestamp, row.latens_us as f64));
                        par.endret = true;
                    }
                    if let Some(t) = sist_beregnet {
                        if t.elapsed() < INTERVAL {
                            continue; //aventer utregning
                        }
                    }
                    sist_beregnet = Some(Instant::now());

                    let grense = nyeste - VINDU_US;
                    for (k,par) in lat.iter_mut() {
                        while let Some(&(t, _)) = par.verdier.front() {
                            if t < grense {
                                par.verdier.pop_front();
                                par.endret = true;
                            } else {
                                break;
                            }
                        }
                        if !par.endret {continue;}
                        par.endret = false;
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
        //fungerer bra på nesten sorterte lister
        buf.sort_by(|a, b| a.1.total_cmp(&b.1));
    }
}
