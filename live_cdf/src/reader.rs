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
const VINDU_US: i64 = 60_000_000; //60 sek




struct Serie {
    verdier: VecDeque<(i64, i32)>, //(tidsstempel, latens i µs)
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
        //latens, sorteres stigende (gjenbrukes for å slippe allokering)
        let mut sortert_v: Vec<i32> = Vec::new();
        let mut sist_beregnet: Option<Instant> = None;
        //deserialize venter på data
        'lesing: for result in rdr.deserialize::<TRow>() {
            match result {
                Ok(row) => {
                    let key: (String, String) = (row.ta.clone(), row.ra.clone());
                    nyeste = nyeste.max(row.first_seen_pcap_ts);
                    let par = lat.entry(key).or_insert_with_key(|k| Par::ny(&sink,k));
                    par.legg_til(&row);

                    if let Some(t) = sist_beregnet && t.elapsed() <= INTERVAL {
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
                            if let Err(e) = tx.send(serie.beregn(k, &mut sortert_v)) {
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
        if let Some(r) = row.rts_cts_prelude_pcap_delta{
        s.legg_til(tid,r);
        }
        //gjensending
        let s = &mut self.serier[LatensType::Gjensending as usize];
        s.legg_til(tid,row.latency_pcap_delta);
        //reorder
        let s = &mut self.serier[LatensType::Reordering as usize];
        if let Some(r) = row.reordering_delay_pcap_delta{
        s.legg_til(tid,r);
        }


    }
}
impl Serie {
    fn sort_latens(&self, buf: &mut Vec<i32>) {
        buf.clear();
        buf.extend(self.verdier.iter().map(|&(_tid, latens)| latens));
        //ustabil sortering holder siden like latenser ikke kan skilles fra hverandre
        buf.sort_unstable();
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
    fn legg_til(&mut self,ts:i64,verdi:i32){
        self.verdier.push_back((ts, verdi));
        self.endret = true;
    }
    //sender alltid en oppdatering, også når serien er tom, slik at GUI-et kan tømme plott og tabell
    fn beregn(&self, k: &ParNokkel, buf: &mut Vec<i32>) -> ParOppdatering {
        self.sort_latens(buf);
        ParOppdatering {
            par_mac: k.clone(),
            latens_type: self.latens_type.clone(),
            trace: self.trace.clone(),
            punkter: stats::cdf_beregning(buf),     //tom når buf er tom
            pairstats: stats::stats_beregning(buf), //None når buf er tom
        }
    }
}
