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
use std::fs::OpenOptions;

use std::time::Instant;
use tcap::{
    capabilities::tcap::CapID,
    config::Config,
    service::{self, tcap::Service},
};

pub(crate) static CPU_CLOCK_SPEED: u64 = 210/2;

pub(crate) static FRONTEND_END_CAP: CapID = 50;
pub(crate) static CLIENT_END_CAP: CapID = 51;
pub(crate) static FS_END_CAP: CapID = 51;

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
    /// The Network Interface to bind
    #[arg(long)]
    interface: String,

    /// Address to bind to (including port number)
    #[arg(short, long)]
    address: String,

    /// Address of the switch control plane (including port number)
    #[arg(short, long)]
    switch_addr: String,
    /// set debug print level
    #[arg(long, action)]
    debug: bool,
}

#[derive(clap::Subcommand, Clone, Debug)]
enum Mode {
    Others {
        /// address of the frontend service
        #[arg(short, long)]
        frontend_address: String,
    },
    Frontend {
        /// Address of the service running the FS
        #[arg(short, long)]
        fs: String,

        /// Address of the service running the Storage
        #[arg(short, long)]
        storage: String,

        /// Address of the service running the GPU
        #[arg(short, long)]
        gpu: String,
    },
    FS {
        /// Address of the service running with Other
        #[arg(short, long)]
        frontend_address: String,
    },

    GPU {
        /// Address of the service running with Other
        #[arg(short, long)]
        frontend_address: String,
    },

    Storage {
        /// Address of the service running with Other
        #[arg(short, long)]
        frontend_address: String,
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
        wtr.write_record([v.0.to_string().as_str(), v.1.to_string().as_str()])
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

    let service = Service::new(Config {
        interface: args.interface,
        address: args.address,
        switch_addr: args.switch_addr,
    })
    .await;

    let s = service.clone();
    let service_thread = tokio::spawn(async move {
        s.clone().run().await.unwrap();
    });

    match args.mode {
        Mode::Others { frontend_address } => {
            let s = service.clone();
            let fa = frontend_address.clone();
            tokio::spawn(async move{
                fs(args.debug, s.clone(), fa.clone()).await;
            });
            let s = service.clone();
            let fa = frontend_address.clone();
            tokio::spawn(async move {
            gpu(
                args.debug,
                s.clone(),
                args.transfer_size,
                fa.clone(),
            )
            .await;
            });
            let s = service.clone();
            let fa = frontend_address.clone();
            tokio::spawn(async move {
                    storage(
                        args.debug,
                        s.clone(),
                        args.transfer_size,
                        fa.clone(),
                    )
                    .await;
            });
            for _ in 0..args.iterations {
                let mut times: Vec<(u64, u128)> = vec![];
                let start = Instant::now();
                client(args.debug, service.clone(), frontend_address.clone()).await;
                let time = start.elapsed();
                info!(
                    "Time: {} µs, transfer_size: {} KiB",
                    time.as_micros() - 20,
                    args.transfer_size
                );
                service.reset().await;
                times.push((args.transfer_size, time.as_micros()-20));
                write_csv("Client".to_string(), args.iterations, times).await;
            }

            let end_cap = service
                .create_remote_capability_with_id(frontend_address, FRONTEND_END_CAP)
                .await;
            end_cap.lock().await.request_invoke_no_wait().await.unwrap();
        }
        Mode::FS { frontend_address } => {
            fs(args.debug, service.clone(), frontend_address.clone()).await;
        }

        Mode::GPU { frontend_address } => {
            gpu(
                args.debug,
                service.clone(),
                args.transfer_size,
                frontend_address.clone(),
            )
            .await;
        }

        Mode::Storage { frontend_address } => {
            storage(
                args.debug,
                service.clone(),
                args.transfer_size,
                frontend_address.clone(),
            )
            .await;
        }

        Mode::Frontend { fs, storage, gpu } => {
            frontend(
                args.debug,
                service.clone(),
                args.transfer_size,
                (fs.clone(), storage.clone(), gpu.clone()),
            )
            .await;
        }
    };
    service_thread.abort();
}
