#![feature(async_closure)]

mod server;
mod client;
mod csv_writer;

use clap::Parser;
use log::LevelFilter;
use tcap::config::Config;
use tcap::service::tcap::Service;
use crate::server::server;
use crate::client::client;
use simple_logger::SimpleLogger;

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

    /// number of packets to be send
    #[arg(short, long)]
    no_packets: u128,

    /// delay between packets in µs
    #[arg(short, long)]
    delay: u64,

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

    let service = Service::new(service_config).await;

    let s = service.clone();
    let service_thread = tokio::spawn(async move {
        let _ = s.run().await.unwrap();
    });

    match &args.mode {
        Mode::Client { .. } => client(args.no_packets, args.delay, service.clone(), args.remote.clone()).await,
        Mode::Server { .. } => server(service.clone()).await,
    };


    service.terminate().await;
    service_thread.abort();
}
