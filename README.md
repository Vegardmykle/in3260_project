# Live CDF av latens i Wi-Fi
## Bygging

```sh
cargo build --release
```

## Kjøring

Med testdata:

```
./target/release/csv_streamer <testdata.csv> | ./target/release/live_cdf
```

Med ekte data 
```
MÅLEVERKTØY | ./target/release/live_cdf
```

## Inputformat

CSV med header-rad. "header= true" i reader.rs