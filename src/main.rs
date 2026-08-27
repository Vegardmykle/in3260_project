use chrono::Utc;
use csv::WriterBuilder;
use rand::RngExt;
use serde::Serialize;
use std::{error::Error, thread};



fn random_latency(min:i32,max:i32) -> i32{
    let mut rng = rand::rng();
    rng.random_range(min..=max)
}
#[derive(Serialize)]
struct Row<'a> {
    timestamp: i64,
    avsender_mac: &'a str,
    mottaker_mac: &'a str,
    ssid:&'a str,
    latens_us: i32,
}

fn example() -> Result<(), Box<dyn Error>> {
    let mut wtr = WriterBuilder::new()
    .has_headers(false)
    .from_writer(std::io::stdout());
        
wtr.serialize(Row{
    timestamp: Utc::now().timestamp_micros(),
    avsender_mac: "F0:20:FF:AD:D5:40",
    mottaker_mac: "00:0A:52:06:A0:B0",
    ssid:"test-123",
    latens_us:random_latency(10,5000000),
})?;
wtr.serialize(Row{
    timestamp: Utc::now().timestamp_micros(),
    avsender_mac: "00:0A:52:06:A0:B0",
    mottaker_mac: "F0:20:FF:AD:D5:40",
    ssid:"test-123",
    latens_us:random_latency(10,5000000),
})?;

    wtr.flush()?;

    Ok(())
}
fn main()->Result<(), Box<dyn Error>>{

    let mut rng = rand::rng(); 
    loop {
        example()?;
        let ms = rng.random_range(1000..5000);
        let dur = std::time::Duration::from_millis(10); 
        thread::sleep(dur);
    }
}