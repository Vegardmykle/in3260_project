use csv::{ReaderBuilder, WriterBuilder};
use felles::TRow;
use std::cmp::max;
use std::{error::Error, thread};

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
