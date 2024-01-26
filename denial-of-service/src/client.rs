use std::time::Duration;
use tcap::service::tcap::Service;
use tokio::time;

pub async fn client(no_packets: u128, delay: u64, service: Service, remote: String) {
    let invalid = service.create_remote_capability_with_id(remote.clone(), 300).await;
    let final_cap = service.create_remote_capability_with_id(remote.clone(), 200).await;

    for _ in 0..no_packets {
        let _ = invalid.lock().await.request_invoke_no_wait().await;
        time::sleep(Duration::from_micros(delay)).await;
    }

    final_cap.lock().await.request_invoke().await.unwrap();
}