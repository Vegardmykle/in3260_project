use chrono::Utc;
use csv::WriterBuilder;
use rand::SeedableRng;
use rand::{RngExt, rngs::StdRng};
use serde::Serialize;
use std::{error::Error, thread};

fn prng_fra_utvalg(rng: &mut StdRng) -> i32 {
    let utvalg: [i32; 3] = [1000000, 2000000, 5000000];
    let index = rng.random_range(0..=utvalg.len() - 1);
    utvalg[index]
}
fn random_latency(min: i32, max: i32) -> i32 {
    let mut rng = rand::rng();
    rng.random_range(min..=max)
}
#[derive(Serialize)]
struct Row<'a> {
    timestamp: i64,
    avsender_mac: &'a str,
    mottaker_mac: &'a str,
    ssid: &'a str,
    latens_us: i32,
}

fn example(rng: &mut StdRng) -> Result<(), Box<dyn Error>> {
    let mut wtr = WriterBuilder::new()
        .has_headers(false)
        .from_writer(std::io::stdout());

    wtr.serialize(Row {
        timestamp: Utc::now().timestamp_micros(),
        avsender_mac: "F0:20:FF:AD:D5:40",
        mottaker_mac: "00:0A:52:06:A0:B0",
        ssid: "TRNG",
        latens_us: random_latency(10, 5000000),
    })?;
    wtr.serialize(Row {
        timestamp: Utc::now().timestamp_micros(),
        avsender_mac: "00:0A:52:06:A0:B0",
        mottaker_mac: "F0:20:FF:AD:D5:40",
        ssid: "PRNG",
        latens_us: prng_fra_utvalg(rng),
    })?;

    wtr.flush()?;

    Ok(())
}
fn main() -> Result<(), Box<dyn Error>> {
    let seed: u64 = 12345;
    let mut rng = StdRng::seed_from_u64(seed);

    loop {
        example(&mut rng)?;
        let ms = 1;
        let dur = std::time::Duration::from_millis(ms);
        thread::sleep(dur);
    }
}
