use std::{sync::Arc, time::Duration};
use tcap::service::tcap::Service;
use tokio::sync::Mutex;
use crate::csv_writer::write_csv;

pub async fn client(no_packets: u128, _delay: u64, service: Service, remote: String) {
    let invalid = service.create_remote_capability_with_id(remote.clone(), 300).await;
    let final_cap = service.create_remote_capability_with_id(remote.clone(), 200).await;
    let packets_per_milisecond: Arc<Mutex<Vec<(u128, u128)>>> = Arc::new(Mutex::new(Vec::new()));
    let packet_rate_tracker = async move |ppm: Arc<Mutex<Vec<(u128, u128)>>>| {
        let mut counter = 0;
        loop {
            let before = service.send_counter.lock().await.clone();
            tokio::time::sleep(Duration::from_millis(1)).await;
            ppm.lock().await.push((counter, service.send_counter.lock().await.clone() - before));
            counter += 1;
        }
    };

    let packet_rate = tokio::runtime::Handle::current().spawn(packet_rate_tracker(packets_per_milisecond.clone()));
    for _ in 0..no_packets {
        let _ = invalid.lock().await.request_invoke_no_wait().await;
    }
    tokio::time::sleep(Duration::from_secs(1)).await; 
    packet_rate.abort();
    write_csv("client-packet-rate.csv".to_string(), packets_per_milisecond).await;
    final_cap.lock().await.request_invoke().await.unwrap();
}
