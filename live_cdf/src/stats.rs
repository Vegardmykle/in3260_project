use liveplot::{PlotPoint, Trace};
use felles::LatensType;

pub type ParNokkel = (String, String);
pub struct Stats {
    pub p95: f64,
    pub p99: f64,
    pub avg: f64,
    pub median: f64,
    pub max: f64,
}
pub struct ParOppdatering {
    pub par_mac: ParNokkel,
    pub latens_type: LatensType,
    pub trace: Trace,
    pub punkter: Vec<PlotPoint>,
    pub pairstats: Option<Stats>, //None = serien er tom
}
pub fn stats_beregning(sortet_v: &[i32]) -> Option<Stats> {
    if sortet_v.is_empty() {
        return None;
    }
    let len = sortet_v.len();
    //O(n) tid på sum, summeres i i64 for å unngå overflyt
    let sum: i64 = sortet_v.iter().map(|&v| v as i64).sum();

    Some(Stats {
        p95: sortet_v[((len - 1) as f64 * 0.95).round() as usize] as f64,
        p99: sortet_v[((len - 1) as f64 * 0.99).round() as usize] as f64,
        avg: sum as f64 / len as f64,
        median: sortet_v[((len - 1) as f64 * 0.50).round() as usize] as f64,
        max: sortet_v[len - 1] as f64,
    })
}

pub fn cdf_beregning(sortet_v: &[i32]) -> Vec<PlotPoint> {
    let n = sortet_v.len();
    let mut punkter = Vec::with_capacity(n);
    //latenser sortert stigende
    for (i, latens) in sortet_v.iter().enumerate() {
        //latens er i µs, deler på 1000 for å få ms (x-aksen har enhet ms)
        punkter.push(PlotPoint {
            x: *latens as f64 / 1e3,
            y: (i + 1) as f64 / n as f64,
        });
    }
    punkter
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_beregning_empty() {
        let empty_data: Vec<i32> = vec![];
        let result = stats_beregning(&empty_data);
        assert!(result.is_none());
    }
    //Sjekker oppførsel med kun ett element i listen
    #[test]
    fn test_stats_beregning_single_element() {
        let single_data = vec![50];
        let result = stats_beregning(&single_data).unwrap();

        // Når det bare er ett element, vil alle persentiler, median, avg og max være det samme elementet
        assert_eq!(result.p95, 50.0);
        assert_eq!(result.p99, 50.0);
        assert_eq!(result.median, 50.0);
        assert_eq!(result.avg, 50.0);
        assert_eq!(result.max, 50.0);
    }
    #[test]
    fn test_stats_beregning_normal_range() {
        // Enkel liste med 10 elementer: verdiene er 0, 1, 2 ... 9
        let data: Vec<i32> = (0..10).collect();

        let result = stats_beregning(&data).unwrap();

        // Formel-sjekk for indekser: (10 - 1) = 9
        // median: (9 * 0.50).round() = 5.0 -> indeks 5 -> verdi 5.0
        // p95:    (9 * 0.95).round() = 9.0 -> indeks 9 -> verdi 9.0
        // p99:    (9 * 0.99).round() = 9.0 -> indeks 9 -> verdi 9.0
        assert_eq!(result.median, 5.0);
        assert_eq!(result.p95, 9.0);
        assert_eq!(result.p99, 9.0);

        // Gjennomsnitt av 0 til 9 er 45 / 10 = 4.5
        assert_eq!(result.avg, 4.5);

        // Siste element i en sortert liste er maks (indeks 9 -> verdi 9.0)
        assert_eq!(result.max, 9.0);
    }
    #[test]
    fn test_stats_beregning_hundred_elements() {
        // 100 elementer: verdiene går fra 1 til 100
        let data: Vec<i32> = (1..=100).collect();

        let result = stats_beregning(&data).unwrap();

        // Formel-sjekk for indekser: (100 - 1) = 99
        // median: (99 * 0.50).round() = 50.0 -> indeks 50 -> verdi 51.0
        // p95:    (99 * 0.95).round() = 94.0 -> indeks 94 -> verdi 95.0
        // p99:    (99 * 0.99).round() = 98.0 -> indeks 98 -> verdi 99.0
        assert_eq!(result.median, 51.0);
        assert_eq!(result.p95, 95.0);
        assert_eq!(result.p99, 99.0);
        assert_eq!(result.max, 100.0);
    }
}
