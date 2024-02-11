#![feature(async_closure)]

mod client;
mod frontend;
mod fs;
mod gpu;
mod storage;

use client::client;
use frontend::frontend;
use fs::fs;
use gpu::gpu;
use storage::storage;

use clap::Parser;
use csv::Writer;
use log::*;
use simple_logger::SimpleLogger;
use std::{collections::HashMap, time::Duration};
use std::fs::OpenOptions;
use std::sync::Arc;
use std::time::Instant;
use tcap::{
    capabilities::tcap::CapID,
    config::Config,
    service::{self, tcap::Service},
};
use tokio::{sync::Mutex, time};

pub(crate) static CPU_CLOCK_SPEED: u64 = 2100000000;

pub(crate) static FRONTEND_END_CAP: CapID = 50;
pub(crate) static FRONTEND_CAP: CapID = 100;
pub(crate) static FS_CAP: CapID = 200;
pub(crate) static STORAGE_CAP: CapID = 300;
pub(crate) static GPU_CAP: CapID = 400;

pub(crate) static FRONTEND_TO_GPU_MEM_CAP: CapID = 450;
pub(crate) static FRONTEND_TO_STORAGE_MEM_CAP: CapID = 350;
pub(crate) static CLIENT_TO_FRONEND_MEM_CAP: CapID = 50;
pub(crate) static STORAGE_TO_FRONEND_MEM_CAP: CapID = 375;
pub(crate) static GPU_TO_FRONTEND_MEM_CAP: CapID = 475;

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
    transfer_size: u64,

    /// set debug print level
    #[arg(long, action)]
    debug: bool,
}

#[derive(clap::Subcommand, Clone, Debug)]
enum Mode {
    Others {
        /// The Network Interface to bind
        #[arg(short, long)]
        interface: String,

        /// Address to bind to (including port number)
        #[arg(short, long)]
        address: String,

        /// Address of the switch control plane (including port number)
        #[arg(short, long)]
        switch_addr: String,

        /// address of the frontend service
        #[arg(short, long)]
        frontend_address: String,
    },
    Frontend {
        /// The Network Interface to bind
        #[arg(short, long)]
        interface: String,

        /// Address to bind to (including port number)
        #[arg(short, long)]
        address: String,

        /// Address of the switch control plane (including port number)
        #[arg(short, long)]
        switch_addr: String,
        
        /// Address of the service running with Other
        #[arg(short, long)]
        remote: String,
    },
}

async fn write_csv(mode: String, iterations: u128, times: Vec<(u64, u128)>) {
    let file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(format!(
            "app-facever-{:?}-microsec-iterations{:?}-{:?}.csv",
            mode,
            iterations,
            Service::get_compilation_commit()
        ))
        .unwrap();
    let mut wtr = Writer::from_writer(file);

    times.iter().for_each(|v| {
        wtr.write_record([
            v.0.to_string().as_str(),
            v.1.to_string().as_str(),
        ])
        .unwrap();
    });
    wtr.flush().unwrap();
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
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
    match args.mode {
        Mode::Others {
            interface,
            address,
            switch_addr,
            frontend_address,
        } => {
            let service = Service::new(Config {
                interface,
                address,
                switch_addr,
            })
            .await;

            let s = service.clone();
            let service_thread = tokio::spawn(async move {
                s.clone().run().await.unwrap();
            });
            fs(args.debug, service.clone(), frontend_address.clone()).await;
            storage(
                args.debug,
                service.clone(),
                args.transfer_size,
                frontend_address.clone(),
            )
            .await;
            gpu(
                args.debug,
                service.clone(),
                args.transfer_size,
                frontend_address.clone(),
            )
            .await;
            info!("wiating 5 seconds for client to run");
            time::sleep(Duration::from_secs(5)).await;

            let mut times: Vec<(u64, u128)> = vec![];
            for _ in 0..args.iterations {
                let start = Instant::now();
                client(args.debug, service.clone(), frontend_address.clone()).await;
                let time = start.elapsed();
                info!("Time: {} µs, transfer_size: {} KiB", time.as_micros(), args.transfer_size);
                service.reset().await;
                times.push((args.transfer_size, time.as_micros()));
            }
            write_csv("Client".to_string(), args.iterations, times).await;
            info!("done");
            let end_cap = service.create_remote_capability_with_id(frontend_address, FRONTEND_END_CAP).await;
            end_cap.lock().await.request_invoke_no_wait().await.unwrap();
            service_thread.abort();
        }
        Mode::Frontend {
            interface,
            address,
            switch_addr,
            remote
        } => {
            let service = Service::new(Config {
                interface,
                address,
                switch_addr,
            })
            .await;

            let s = service.clone();
            let service_thread = tokio::spawn(async move {
                s.clone().run().await.unwrap();
            });

            frontend(args.debug, service.clone(), args.transfer_size, (remote.clone(), remote.clone(), remote.clone())).await;
            service_thread.abort();
        }
    };
}
