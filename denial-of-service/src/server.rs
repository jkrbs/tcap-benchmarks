use csv::Writer;
use tcap::object::tcap::object::RequestObject;
use tcap::service::tcap::Service;
use std::fs::OpenOptions;
use tokio::sync::{Mutex, Notify};
use std::{sync::Arc, time::Duration};

async fn write_csv(path: String, rate: Arc<Mutex<Vec<(u128, u128)>>>) {
    let file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(path)
        .unwrap();
    let mut wtr = Writer::from_writer(file);
    wtr.write_record(["time(s)", "packets"]).unwrap();
    rate.lock().await.iter().for_each(|v| {
        wtr.write_record([
            v.0.to_string().as_str(),
            v.1.to_string().as_str()
        ])
        .unwrap();
    });
    wtr.flush().unwrap();
}

/**
 * Server will reply with CapInvalid. It will just provide a handler, to leave this function.
 */
pub async fn server(service: Service) {


    let notifier = Arc::new(Notify::new());
    let n = notifier.clone();
    let end = Arc::new(Mutex::new(RequestObject::new(Box::new(move |_| {
        n.notify_waiters();
        Ok(())
    })).await));


    let end_cap = service.create_capability_with_id(200).await;
    end_cap.lock().await.bind_req(end).await;

    let packets_per_milisecond: Arc<Mutex<Vec<(u128, u128)>>> = Arc::new(Mutex::new(Vec::new()));
    let packet_rate_tracker = async move |ppm: Arc<Mutex<Vec<(u128, u128)>>>| {
        let mut counter = 0;
        loop {
            let before = service.recv_counter.lock().await.clone();
            tokio::time::sleep(Duration::from_secs(1)).await;
            ppm.lock().await.push((counter, service.recv_counter.lock().await.clone() - before));
            counter += 1;
        }
    };
    let packet_rate = tokio::runtime::Handle::current().spawn(packet_rate_tracker(packets_per_milisecond.clone()));
    notifier.notified().await;
    packet_rate.abort();
    write_csv("server-packet-rate.csv".to_string(), packets_per_milisecond).await;
}