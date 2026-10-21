use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::{error::Error, thread};

#[derive(Serialize, Deserialize)]
pub struct TRow {
    pub first_seen_pcap_ts: i64,
    pub last_seen_pcap_ts: i64,
    pub ra: String,
    pub ta: String,
    pub da: String,
    pub sa: String,
    pub len: i32,
    pub tid: i32,
    pub seq_nr: i32,
    pub retries: i32,
    pub rts_cts_prelude_pcap_delta: i32,
    pub latency_pcap_delta: i32,
    pub reordering_delay_pcap_delta: Option<i32>,
    pub missed_first_frame: bool,
    pub wifi_standard: String,
    pub from_ds: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let filnavn = std::env::args().nth(1);
    let mut leser = ReaderBuilder::new()
        .has_headers(true)
        .from_path(filnavn.unwrap())?;
    let mut skriver = WriterBuilder::new().from_writer(std::io::stdout());
    let mut forrige: Option<i64> = None;
    for result in leser.deserialize::<TRow>() {
        match result {
            Ok(rad) => {
                if let Some(f) = forrige {
                    if rad.first_seen_pcap_ts > f {
                        let diff = rad.first_seen_pcap_ts - f;
                        thread::sleep(core::time::Duration::from_micros(diff as u64));
                    }
                }
                forrige = max(forrige, Some(rad.first_seen_pcap_ts));
                skriver.serialize(rad)?;
                skriver.flush()?;
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }
    Ok(())
}
