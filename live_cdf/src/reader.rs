use crate::stats;
use crate::stats::{ParNokkel, ParOppdatering};
use csv::ReaderBuilder;
use liveplot::{PlotSink, Trace};
use std::collections::{BTreeMap, VecDeque};
use std::io;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};
use felles::{TRow,LatensType};

const INTERVAL: Duration = Duration::from_millis(100);
const VINDU_US: i64 = 60_000_000;




struct Serie {
    verdier: VecDeque<(i64, f64)>,
    trace: Trace,
    endret: bool,
    latens_type: LatensType,
}
struct Par{
    serier: [Serie;3]
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
                    let key: (String, String) = (row.ta.clone(), row.ra.clone());
                    nyeste = nyeste.max(row.first_seen_pcap_ts);
                    {
                        let par = lat.entry(key).or_insert_with_key(|k| Par::ny(&sink,k));
                        par.legg_til(&row);
                    }
                    if let Some(t) = sist_beregnet
                        && t.elapsed() <= INTERVAL
                    {
                        continue; //aventer utregning
                    }
                    sist_beregnet = Some(Instant::now());

                    let grense = nyeste - VINDU_US;
                    for (k, par) in lat.iter_mut() {
                        for serie in par.serier.iter_mut() {
                            serie.fjern_eldre_enn(grense);
                            if !serie.endret {
                                continue;
                            }
                            serie.endret= false;
                            if let Some(oppdatering) = serie.beregn(k, &mut sortet_v)
                                && let Err(e) = tx.send(oppdatering){
                                eprintln!("Error sending oppdatering {:?}", e);
                                break 'lesing;
                            }
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
    fn ny(sink: &PlotSink, k:&ParNokkel) -> Self {
        Self {
            serier: [
            Serie::ny(sink,format!("RTS/CTS -> {}, {}", k.0, k.1),LatensType::RTSCTS),
            Serie::ny(sink,format!("Gjensending -> {}, {}", k.0, k.1),LatensType::Gjensending),
            Serie::ny(sink,format!("Reordering -> {}, {}", k.0, k.1),LatensType::Reordering)
        ],
        }
    }
    fn legg_til(&mut self,row:&TRow){
        let tid = row.first_seen_pcap_ts;
        //prelude
        let s = &mut self.serier[LatensType::RTSCTS as usize];
        s.legg_til(tid,row.rts_cts_prelude_pcap_delta as f64);
        //gjensending
        let s = &mut self.serier[LatensType::Gjensending as usize];
        s.legg_til(tid,row.latency_pcap_delta as f64);
        //reorder
        let s = &mut self.serier[LatensType::Reordering as usize];
        if let Some(r) = row.reordering_delay_pcap_delta{
        s.legg_til(tid,r as f64);
        }


    }
}
impl Serie {
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
    fn ny(sink:&PlotSink, navn:String,typ:LatensType)->Self{
        Serie{
            verdier: VecDeque::new(),
            trace: sink
                .create_trace(navn, None),
            endret: true,
            latens_type: typ,
        }
    }
    fn legg_til(&mut self,ts:i64,verdi:f64){
        self.verdier.push_back((ts, verdi));
        self.endret = true;
    }
    fn beregn(&self, k: &ParNokkel, buf: &mut Vec<(i64, f64)>) -> Option<ParOppdatering> {
        self.sort_latens(buf);
        let punkter = stats::cdf_beregning(buf);
        if let Some(stats) = stats::stats_beregning(buf) {
            let par_oppdatering: ParOppdatering = ParOppdatering {
                par_mac: k.clone(),
                latens_type: self.latens_type.clone(),
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
