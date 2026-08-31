use std::{ collections::{BTreeMap, VecDeque}, io};
use csv::ReaderBuilder;
use liveplot::{channel_plot, run_liveplot, LivePlotConfig, PlotPoint, PlotSink, Trace};
use serde::Deserialize;

const  VINDU_US:i64 = 60_000_000;
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
        
        let mut nyeste:i64 = 0;
        let mut sortet_v: Vec<(i64,f64)> = Vec::new();
        //deserialize venter på data
        'lesing: for result in rdr.deserialize::<Row>(){
            match result {
                
                Ok(row)=> {
                    let key:(String,String) = (row.avsender_mac,row.mottaker_mac);
                    nyeste = nyeste.max(row.timestamp);
                    {
                        let par = lat.entry(key).or_insert_with_key(|k|Par {
                            verdier:VecDeque::new(),
                            trace: sink.create_trace(format!("{} at {} -> {}",row.ssid,k.0,k.1), None),
                        });
                        par.verdier.push_back((row.timestamp, row.latens_us as f64));
                    }
                    
                    let grense = nyeste - VINDU_US; 
                    for par in lat.values_mut(){
                        while let Some(&(t,_)) = par.verdier.front(){
                            if t<grense{
                                par.verdier.pop_front();
                            } else{
                                break;
                            }    
                        }
                        if !cdf_beregning(par, &sink,&mut sortet_v){
                            break 'lesing;
                        }
                    }
                    
                    
                    
                    
                    
                },Err(e)=>{eprintln!("error:{}",e);continue;}
            };
            
        }
        
        
    });
    
    run_liveplot(rx, LivePlotConfig::default())
}

fn cdf_beregning(
    par:&Par, sink:&PlotSink,
    sortet_v:&mut Vec<(i64,f64)>
)->bool{
    sortet_v.clear();
    sortet_v.extend(par.verdier.iter().copied());

    sortet_v.sort_by(|a,b|a.1.total_cmp(&b.1));
    let n = sortet_v.len();
    let mut punkter = Vec::with_capacity(n);
    //sortert vector med tid og latens
    for (i,(_tid,latens)) in sortet_v.iter().enumerate(){
        //deler på 1 mill for å få det i ms istendefor dager
        punkter.push(PlotPoint{x: *latens/1e6,y: ((i+1)as f64 /n as f64)});
        
    }
    if let Err(e) = sink.set_data(&par.trace, punkter){
        eprintln!("{}",e);
        return false;
    }
    true

}