#![feature(async_closure)]

use clap::Parser;
use log::*;
use simple_logger::SimpleLogger;
use std::sync::Arc;
use std::time::Instant;
use tcap::{
    capabilities::tcap::Capability, config::Config, object::tcap::object::RequestObject,
    service::tcap::Service,
};
use tokio::runtime::Handle;
use tokio::sync::Mutex;

mod chain;
mod star;

use crate::chain::chain_benchmark;
use crate::star::star_benchmark;

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Error)
        .init()
        .unwrap();

    let steps: u128 = 10000;

    let service_config1 = Config {
        interface: "lo".to_string(),
        address: "127.0.0.1:1234".to_string(),
        switch_addr: "127.0.0.1:9999".to_string(),
    };
    let service1 = Service::new(service_config1).await;

    let s = service1.clone();
    let service_thread1 = tokio::spawn(async move {
        let _ = s.run().await.unwrap();
    });

    let service_config2 = Config {
        interface: "lo".to_string(),
        address: "127.0.0.1:1235".to_string(),
        switch_addr: "127.0.0.1:9999".to_string(),
    };
    let service2 = Service::new(service_config2).await;

    let s = service2.clone();
    let service_thread2 = tokio::spawn(async move {
        let _ = s.run().await.unwrap();
    });

    let start = Instant::now();
    star_benchmark(steps, vec![service1.clone(), service2.clone()]).await;
    let micros = start.elapsed().as_micros();
    println!("Elapsed star: {:?}µs, avg: {:?}µs", micros, micros / steps);

    let start = Instant::now();
    chain_benchmark(steps, vec![service1.clone(), service2.clone()]).await;
    let micros = start.elapsed().as_micros();
    println!("Elapsed chain: {:?}µs, avg: {:?}µs", micros, micros / steps);

    service1.terminate().await;
    service_thread1.abort();

    service2.terminate().await;
    service_thread2.abort();
}
