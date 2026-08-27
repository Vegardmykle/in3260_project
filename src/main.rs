use std::{collections::{BTreeMap, VecDeque}, io};
use csv::ReaderBuilder;
use liveplot::{channel_plot, run_liveplot, LivePlotConfig, PlotPoint,Trace};
use serde::Deserialize;

#[derive(Deserialize)]
struct Row{
    timestamp: i64,
    avsender_mac: String,
    mottaker_mac: String,
    ssid:String,
    latens_us: i32,
}
struct Par{
    verdier:VecDeque<(i64, f64)>,
    trace:Trace
}

//run with 
//% cd csv_streamer && cargo build && cd ..
//cd live_cdf && ../csv_streamer/target/debug/csv_streamer | cargo run
fn main() -> eframe::Result<()>{
    let (sink, rx) = channel_plot();
    //let tr = sink.create_trace("cdf", None);
    //kode for lesing av stdin data
    std::thread::spawn(move || {
        let mut  rdr = ReaderBuilder::new().has_headers(false).from_reader(io::stdin() );
        let mut lat: BTreeMap<(String,String), Par> = BTreeMap::new();
        //for uten hash
        //let mut lat: Vec<f64> = Vec::new();
        
        //deserialize venter på data
        for result in rdr.deserialize::<Row>(){
            match result {
                
                Ok(row)=> {
                    let key:(String,String) = (row.avsender_mac,row.mottaker_mac);
                    let navn = format!("{} at {} -> {}",row.ssid,key.0,key.1);

                    let par = lat.entry(key).or_insert_with(||Par {
                        verdier:VecDeque::new(),
                        trace: sink.create_trace(navn, None),
                    });

                    par.verdier.push_back((row.timestamp, row.latens_us as f64));
                    let n = par.verdier.len();
                    let mut sortet_v:Vec<(i64,f64)> =par.verdier.iter().copied().collect(); 
                    
                    sortet_v.sort_by(|a,b|a.1.total_cmp(&b.1));
                        let mut punkter = Vec::with_capacity(n);
                        for (i,(tid,latens)) in sortet_v.iter().enumerate(){
                            punkter.push(PlotPoint{x: *latens/1e6,y: ((i+1)as f64 /n as f64)});

                        }
                    if let Err(e) = sink.set_data(&par.trace, punkter){
                        println!("{}",e);
                        break;
                    }
                   
                    

                },Err(e)=>{eprintln!("error:{}",e);continue;}
            };
    
        }
           

    });

    run_liveplot(rx, LivePlotConfig::default())
}