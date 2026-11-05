use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TRow {
    pub first_seen_pcap_ts: i64,
    pub last_seen_pcap_ts: i64,
    pub ra: String,
    pub ta: String,
    pub da: String,
    pub sa: String,
    pub len: u16,
    pub tid: u8,
    pub seq_nr: u16,
    pub retries: i32,
    pub rts_cts_prelude_pcap_delta: i32, //rtscts brude den være optonal?
    pub latency_pcap_delta: i32,
    pub reordering_delay_pcap_delta: Option<i32>,
    pub missed_first_frame: bool,
    pub wifi_standard: String,
    pub from_ds: bool,
}
#[derive(Debug,Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LatensType{
    RTSCTS,
    Gjensending,
    Reordering
}
//navnet som vises i tabellen
impl std::fmt::Display for LatensType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let navn = match self {
            LatensType::RTSCTS => "RTS/CTS",
            LatensType::Gjensending => "Gjensending",
            LatensType::Reordering => "Reordering",
        };
        write!(f, "{navn}")
    }
}

//fun for sum viser sum av latens per node
