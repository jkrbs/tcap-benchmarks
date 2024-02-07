mod client;
mod server;
use std::{
    collections::HashMap,
    fs::OpenOptions,
    time::{Duration, Instant},
};

use clap::Parser;
use client::client;
use csv::Writer;
use log::*;
use server::server;
use simple_logger::SimpleLogger;

#[derive(clap::Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    /// client opr server mode
    #[command(subcommand)]
    mode: Mode,

    /// set debug print level
    #[arg(long, action)]
    debug: bool,
}

#[derive(clap::Subcommand, Clone, Debug)]
enum Mode {
    Client {
        /// number of iterations to measure
        #[arg(short, long)]
        iterations: u128,

        /// remote host
        #[arg(short, long)]
        remote: String,
    },
    Server {
        /// Address to bind to (including port number)
        #[arg(short, long)]
        address: String,
    },
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

    let mut times = HashMap::<i32, Vec<Duration>>::new();

    match args.mode {
        Mode::Client { iterations, remote } => {
            for c in 0..100 {
                times.insert(c, vec![]);

                let start = Instant::now();
                client(remote.clone(), iterations).await.unwrap();
                let end = start.elapsed();
                times.get_mut(&c).unwrap().push(end);

                info!(
                    "c: {:?}, time avg: {:?} µs",
                    c,
                    end.as_micros() / iterations
                );
            }

            let file = OpenOptions::new()
                .write(true)
                .append(true)
                .create(true)
                .open(format!(
                    "grpc-baseline-micro-sec-{:?}-{:?}.csv",
                    iterations,
                    remote.clone().replace('/', "-")
                ))
                .unwrap();

            let mut wtr = Writer::from_writer(file);
            let mut keys = times.keys().collect::<Vec<&i32>>();
            keys.sort();
            keys.iter().for_each(|key| {
                times.get(key).unwrap().iter().for_each(|v| {
                    wtr.write_record([
                        key.to_string().as_str(),
                        iterations.to_string().as_str(),
                        v.as_micros().to_string().as_str(),
                    ])
                    .unwrap();
                });
            });
            wtr.flush().unwrap();
        }
        Mode::Server { address } => {
            server(address).await;
        }
    }
}
