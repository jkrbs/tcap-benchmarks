#![feature(async_closure)]
#![feature(slice_pattern)]
use clap::Parser;
use csv::Writer;
use log::*;
use simple_logger::SimpleLogger;
use std::fs::OpenOptions;
use std::sync::Arc;
use std::time::Instant;
use tcap::{config::Config, service::tcap::Service};
use tokio::sync::Mutex;
pub mod client;
pub mod server;

use crate::client::client;
use crate::server::server;

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

    /// remote host
    #[arg(short, long)]
    remote: String,

    /// set scaling evaluation
    #[arg(long, action)]
    scaling: bool,

    /// set debug print level
    #[arg(long, action)]
    debug: bool,
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
        switch_addr: String,
    },
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
    },
}

async fn write_csv(args: Args, times: Arc<Mutex<Vec<(usize, u128)>>>) {
    let scalestr = match args.scaling {
        true => "scaling",
        false => "no-scaling-1024bytes",
    };

    let file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(format!(
            "throughput-{:?}-{:?}-iterations{:?}-{:?}.csv",
            scalestr,
            args.remote,
            args.iterations,
            Service::get_compilation_commit()
        ))
        .unwrap();
    let mut wtr = Writer::from_writer(file);

    // these call require nightly
    // let mut times = times.
    let times = times.lock().await;
    times.iter().for_each(|v| {
            wtr.write_record([
                v.0.to_string().as_str(),
                v.1.to_string().as_str(),
            ])
            .unwrap();
        });
    wtr.flush().unwrap();
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    match args.debug {
        true => SimpleLogger::new()
            .with_level(LevelFilter::Debug)
            .init()
            .unwrap(),
        false => SimpleLogger::new()
            .with_level(LevelFilter::Info)
            .init()
            .unwrap(),
    };
    let service_config = match &args.mode {
        Mode::Client {
            interface,
            address,
            switch_addr,
        } => Config {
            interface: interface.clone(),
            address: address.clone(),
            switch_addr: switch_addr.clone(),
        },
        Mode::Server {
            interface,
            address,
            switch_addr,
        } => Config {
            interface: interface.clone(),
            address: address.clone(),
            switch_addr: switch_addr.clone(),
        },
    };
    
    let service = Service::new(service_config.clone()).await;

    let s = service.clone();
    let service_thread = tokio::spawn(async move {
        let _ = s.run().await.unwrap();
    });

    if args.scaling {
        for size in [10, 100, 256, 512,1024, 2048, 3000, 4000, 5000, 6000, 7000, 2^15, 2^18, 2^20] {
            let times = Arc::new(Mutex::new(Vec::<(usize, u128)>::new()));
            match &args.mode {
                Mode::Client { .. } => {
                    let start = Instant::now();
                    client(args.iterations, service.clone(), args.remote.clone(), args.debug, size).await;
                    times
                .lock()
                .await
                .push((size, start.elapsed().as_micros()));
                }
                Mode::Server { .. } => {
                    server(service.clone()).await
                }
            };

            write_csv(args.clone(), times).await;
        }
    } else {
        let times = Arc::new(Mutex::new(Vec::<(usize, u128)>::new()));
        
        match &args.mode {
            Mode::Client { .. } => {
                let start = Instant::now();
                client(args.iterations, service.clone(), args.remote.clone(), args.debug, 1024).await;
                let micros = start.elapsed().as_micros()/args.iterations;
        times
            .lock()
            .await
            .push((1024, micros));
        let s1packets =
            service.recv_counter.lock().await.clone() + service.send_counter.lock().await.clone();
        println!(
            "Elapsed: {:?}µs, rate: {:?}bytes/µs, number of packets: service1: {:?}",
            micros,
            1024/micros,
            s1packets
        );

            }
            Mode::Server { .. } => {
                server(service.clone()).await
            }
        };
        
        write_csv(args.clone(), times).await;
    }

    service.terminate().await;
    service_thread.abort();

    drop(service);
}
