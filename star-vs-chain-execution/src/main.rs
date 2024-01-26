#![feature(async_closure)]

use clap::Parser;
use csv::Writer;
use log::*;
use simple_logger::SimpleLogger;
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::sync::Arc;
use std::time::Instant;
use tcap::{config::Config, service::tcap::Service};
pub mod chain;
pub mod star;

use crate::chain::chain_benchmark_client;
use crate::chain::chain_benchmark_server;
use crate::star::star_benchmark_client;
use crate::star::star_benchmark_server;

#[derive(clap::Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    /// client opr server mode
    #[command(subcommand)]
    mode: Mode,

    /// number of iterations to measure
    #[arg(short, long)]
    iterations: u128,

    /// depth of the call chain
    #[arg(short, long)]
    depth: u128,

    /// remote host
    #[arg(short, long)]
    remote: String,

    /// set scaling evaluation
    #[arg(long, action)]
    scaling: bool,

    /// set debug print level
    #[arg(long, action)]
    debug: bool
}

#[derive(clap::Subcommand, Clone, Debug)]
enum Mode {
    Client {    
        /// The Network Interface to bind
        #[arg(short, long)]
        interface: String,
    
        /// Address to bind to (including port number)
        #[arg(short, long)]
        address: String,
    
        /// Address of the switch control plane (including port number)
        #[arg(short, long)]
        switch_addr: String,},
    Server {
        /// The Network Interface to bind
        #[arg(short, long)]
        interface: String,

        /// Address to bind to (including port number)
        #[arg(short, long)]
        address: String,

        /// Address of the switch control plane (including port number)
        #[arg(short, long)]
        switch_addr: String,
    }
}

async fn write_csv(args: Args, times: Arc<Mutex<HashMap::<u8, Vec<(u128, u128)>>>>) {
    let scalestr = match args.scaling {
        true => "scaling",
        false => "no-scaling",
    };

    let file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(format!(
            "star-chain-steps{:?}-{:?}-{:?}-{:?}-iterations{:?}-{:?}.csv",
            args.depth,
            scalestr, 
            args.mode,
            args.remote,
            args.iterations,
            Service::get_compilation_commit()
        ))
        .unwrap();
    let mut wtr = Writer::from_writer(file);

    // these call require nightly
    // let mut times = times.
    let mut times = times.lock().await;
    let mut keys = times.keys().collect::<Vec<&u8>>();
    keys.sort();
    keys.iter().for_each(|key| {
        let mut counter = 0;
        times.get(*key).unwrap().iter().for_each(|v| {
            wtr.write_record([
                key.to_string().as_str(),
                counter.to_string().as_str(),
                v.0.to_string().as_str(),
                v.1.to_string().as_str(),
            ])
            .unwrap();
            counter += 1;
        });
    });
    wtr.flush().unwrap();
}

#[tokio::main]
async fn main() {
    
    let args = Args::parse();
    match args.debug {
    true => 
    SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .init()
        .unwrap(),
    false => 
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap()
    };
    let service_config = match &args.mode {
        Mode::Client { interface, address, switch_addr } => Config { interface: interface.clone(), address: address.clone(), switch_addr: switch_addr.clone() },
        Mode::Server { interface, address, switch_addr } => Config { interface: interface.clone(), address: address.clone(), switch_addr: switch_addr.clone() },
    };

    let times = Arc::new(Mutex::new(HashMap::<u8, Vec<(u128, u128)>>::new()));
    times.lock().await.insert(0, Vec::new());
    times.lock().await.insert(1, Vec::new());

    let a = args.clone();
    let t = times.clone();
    ctrlc::set_handler(move || {
        let writer = async move |a: Args, t: Arc<Mutex<HashMap<u8, Vec<(u128, u128)>>>>| {
        write_csv(a.clone(), t.clone()).await;
    };

    tokio::spawn(writer(a.clone(), t.clone()));
    })
    .expect("Error setting Ctrl-C handler");

    let service = Service::new(service_config.clone()).await;

    let s = service.clone();
    let service_thread = tokio::spawn(async move {
        let _ = s.run().await.unwrap();
    });

    if args.scaling {
        for depth in [10, 100, 1000, 2000, 3000, 4000, 5000, 6000, 7000, 8000, 9000, 10000] {
            let start = Instant::now();
            match &args.mode {
                Mode::Client { .. } => star_benchmark_client(depth, service.clone(), args.remote.clone()).await,
                Mode::Server { .. } => star_benchmark_server(depth, service.clone(), args.remote.clone()).await,
            };
        
            times.lock().await.get_mut(&0).unwrap().push((depth, start.elapsed().as_micros()));

            let start = Instant::now();
            match &args.mode {
                Mode::Client { .. } => chain_benchmark_client(depth/2 -1, service.clone(), args.remote.clone()).await,
                Mode::Server { .. } => chain_benchmark_server(depth/2 - 2, service.clone(), args.remote.clone()).await,
            };
            
            times.lock().await.get_mut(&1).unwrap().push((depth, start.elapsed().as_micros()));
        }
    } else {
        let start = Instant::now();
        match &args.mode {
            Mode::Client { .. } => star_benchmark_client(args.depth, service.clone(), args.remote.clone()).await,
            Mode::Server { .. } => star_benchmark_server(args.depth, service.clone(), args.remote.clone()).await,
        };

        times.lock().await.get_mut(&0).unwrap().push((args.depth, start.elapsed().as_micros()));
        // let s1packets = service1.recv_counter.lock().await.clone() + service1.send_counter.lock().await.clone();
        // println!("Elapsed star: {:?}µs, avg: {:?}µs, number of packets: service1: {:?}", micros, micros / steps, s1packets);

        let start = Instant::now();
        match &args.mode {
            Mode::Client { .. } => chain_benchmark_client(args.depth/2 -1, service.clone(), args.remote.clone()).await,
            Mode::Server { .. } => chain_benchmark_server(args.depth/2 - 2, service.clone(), args.remote.clone()).await,
        };
        
        times.lock().await.get_mut(&1).unwrap().push((args.depth, start.elapsed().as_micros()));
    }
    // let s1packets_now = service1.recv_counter.lock().await.clone() + service1.send_counter.lock().await.clone();
    // println!("Elapsed chain: {:?}µs, avg: {:?}µs, number of packets: service1: {:?}", micros, micros / steps, s1packets_now-s1packets);
    service.terminate().await;
    service_thread.abort();

    drop(service);

    write_csv(args, times).await;
}
