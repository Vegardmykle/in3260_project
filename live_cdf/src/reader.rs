use crate::stats;
use crate::stats::{ParNokkel, ParOppdatering};
use csv::ReaderBuilder;
use liveplot::{PlotSink, Trace};
use serde::Deserialize;
use std::collections::{BTreeMap, VecDeque};
use std::io;
use std::sync::mpsc::Sender;
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
struct Par {
    verdier: VecDeque<(i64, f64)>,
    trace: Trace,
    endret: bool,
}

pub fn start(sink: PlotSink, tx: Sender<ParOppdatering>) {
    std::thread::spawn(move || {
        let mut rdr = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(io::stdin());
        //key = (avsender mac, mottaker_mac), Par(verdier(timestamp, latens),trace/sink)
        let mut lat: BTreeMap<(String, String), Par> = BTreeMap::new();
        //timestamp
        let mut nyeste: i64 = 0;
        //timestamp, latens
        let mut sortet_v: Vec<(i64, f64)> = Vec::new();
        let mut sist_beregnet: Option<Instant> = None;
        //deserialize venter på data
        'lesing: for result in rdr.deserialize::<TRow>() {
            match result {
                Ok(row) => {
                    let key: (String, String) = (row.ta, row.ra);
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
                    if let Some(t) = sist_beregnet
                        && t.elapsed() <= INTERVAL
                    {
                        continue; //aventer utregning
                    }
                    sist_beregnet = Some(Instant::now());

                    let grense = nyeste - VINDU_US;
                    for (k, par) in lat.iter_mut() {
                        par.fjern_eldre_enn(grense);
                        if !par.endret {
                            continue;
                        }
                        par.endret = false;
                        if let Some(oppdatering) = par.beregn(k, &mut sortet_v)
                            && let Err(e) = tx.send(oppdatering)
                        {
                            eprintln!("Error sending parOppdatering: {:?}", e);
                            break 'lesing;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{}", e);
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
    fn fjern_eldre_enn(&mut self, grense: i64) {
        while let Some(&(t, _)) = self.verdier.front() {
            if t < grense {
                self.verdier.pop_front();
                self.endret = true;
            } else {
                break;
            }
        }
    }
    fn beregn(&self, k: &ParNokkel, buf: &mut Vec<(i64, f64)>) -> Option<ParOppdatering> {
        self.sort_latens(buf);
        let punkter = stats::cdf_beregning(buf);
        if let Some(stats) = stats::stats_beregning(buf) {
            let par_oppdatering: ParOppdatering = ParOppdatering {
                par_mac: k.clone(),
                trace: self.trace.clone(),
                punkter,
                pairstats: stats,
            };
            Some(par_oppdatering)
        } else {
            None
        }
    }
}
