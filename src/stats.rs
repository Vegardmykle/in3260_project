use liveplot::PlotPoint;


pub struct Stats {
    pub p95: f64,
    pub p99: f64,
    pub avg: f64,
    pub median: f64,
    pub max: f64,
}
pub fn stats_beregning(sortet_v:&[(i64,f64)])-> Option<Stats>{
    if sortet_v.is_empty() {return None}
    let len = sortet_v.len();
    //O(n) tid på sum
    let sum: f64 = sortet_v.iter().map(|item| item.1).sum();

    Option::from(Stats {
        p95: sortet_v[((len-1) as f64 * 0.95).round() as usize].1,
        p99: sortet_v[((len-1) as f64 * 0.99).round() as usize].1,
        avg: sum / (len as f64),
        median: sortet_v[((len-1) as f64 * 0.50).round() as usize].1,
        max: sortet_v[len-1].1,
    })




}

pub fn cdf_beregning(sortet_v:&[(i64,f64)]) -> Vec<PlotPoint> {
    let n = sortet_v.len();
    let mut punkter = Vec::with_capacity(n);
    //sortert vector med tid og latens
    for (i, (_tid, latens)) in sortet_v.iter().enumerate() {
        //deler på 1 mill for å få det i sek istendefor dager
        punkter.push(PlotPoint {
            x: *latens / 1e6,
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
        let empty_data: Vec<(i64, f64)> = vec![];
        let result = stats_beregning(&empty_data);
        assert!(result.is_none());
    }
    //Sjekker oppførsel med kun ett element i listen
    #[test]
    fn test_stats_beregning_single_element() {
        let single_data = vec![(100, 50.0)];
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
        // Enkel liste med 10 elementer: verdiene er 0.0, 1.0, 2.0 ... 9.0
        let data: Vec<(i64, f64)> = (0..10).map(|i| (i, i as f64)).collect();

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
        // 100 elementer: verdiene går fra 1.0 til 100.0
        let data: Vec<(i64, f64)> = (1..=100).map(|i| (i, i as f64)).collect();

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

