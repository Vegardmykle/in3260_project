use serde::{Deserialize, Serialize};

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
