use core::num;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use simple_logger::SimpleLogger;
use tcap::capabilities::tcap::Capability;
use tcap::config::Config;
use tcap::service::tcap::Service;
use tcap::object::tcap::object::RequestObject;
use log::*;
use tokio::sync::{Mutex, Notify};
use tokio::time::Instant;
use csv::{Writer, ByteRecord};

async fn create_receiver_handler(service: Service) -> (Arc<Notify>, Arc<Mutex<Capability>>) {
    let notify = Arc::new(Notify::new());
    let n = notify.clone();
    let pong_reciever = Arc::new(Mutex::new(RequestObject::new(Box::new(move |cap: Option<Arc<Mutex<Capability>>>| {
        info!("Received Pong");
        notify.notify_waiters();
        Ok(())
    })).await));
    let receiver_cap = service.create_capability().await;
    let _ = receiver_cap.lock().await.bind(pong_reciever).await;

    return (n, receiver_cap);
}

#[tokio::main]
async fn main() {
    let num_caps = [1,10,100,1000,10000,100000];
    let remote = "127.0.0.1:1231";
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();

    let service_config = Config {
        interface: "lo".to_string(),
        address: "127.0.0.1:1234".to_string(),
        switch_addr: "10.0.0.1".to_string(),
    };
    let service = Service::new(service_config).await;
    let s = service.clone();
    let service_thread = tokio::spawn(async move {
        let _ = s.run().await.unwrap();
    });

    let pong_server = service.create_remote_capability_with_id(remote.to_string(), 100).await;
    let mut times = HashMap::<i32, Vec<Duration>>::new();
    
    for c in num_caps {
        let (notify, cap) = create_receiver_handler(service.clone()).await;
        cap.lock().await.delegate(remote.into()).await.unwrap();
        
        let mut time_vec = Vec::new();

        let mut cap_vec = Vec::new();
        for _ in 1..c {
            let cap = service.create_capability().await;
            cap.lock().await.delegate(remote.into()).await.unwrap();
            cap_vec.push(cap);
        }

        for _ in 0..100 {
            let now = Instant::now();
            let r = pong_server.lock().await.request_invoke_with_continuation(Some(cap.clone())).await;
            let time = now.elapsed();
            time_vec.push(time);
        }
        let sum: u64 = Iterator::sum(time_vec.iter().map(|t| {t.as_micros() as u64}));
        info!("c: {:?}, time avg: {:?}", c , sum/(time_vec.len() as u64));
        times.insert(c, time_vec);

        for c in cap_vec {
            c.lock().await.revoke(service.clone()).await.unwrap();
        }
    }
    
    service.terminate().await;

    let _ = service_thread.await;

    let mut wtr = Writer::from_path(format!("latency-bench-max-{:?}.csv", num_caps)).unwrap();
    let mut keys = times.keys().collect::<Vec<&i32>>();
    keys.sort();
    keys.iter().for_each(|key| {
        let sum: u64 = Iterator::sum(times.get(key).unwrap().iter().map(|t| {t.as_micros() as u64}));
        let avg = (sum/(times.get(key).unwrap().len() as u64)) as i32;

        wtr.write_record([key.to_string().as_str(), avg.to_string().as_str()]).unwrap();
    });
    wtr.flush().unwrap();
}
