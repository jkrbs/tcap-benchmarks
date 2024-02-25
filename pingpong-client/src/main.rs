use std::sync::Arc;

use simple_logger::SimpleLogger;
use tcap::capabilities::tcap::Capability;
use tcap::config::Config;
use tcap::service::tcap::Service;
use tcap::object::tcap::object::RequestObject;
use log::*;
use tokio::sync::Mutex;
use tokio::time::Instant;

#[tokio::main]
async fn main() {
    SimpleLogger::new().with_level(LevelFilter::Debug).init().unwrap();

    let service_config = Config {
        interface: "veth3".to_string(),
        address: "10.0.3.2:1234".to_string(),
        switch_addr: "10.0.9.2:1234".to_string(),
    };
    let service = Service::new(service_config).await;
    let s = service.clone();
    let service_thread = tokio::spawn(async move {
        let _ = s.run().await.unwrap();
    });

    let pong_server = service.create_remote_capability_with_id("10.0.1.2:1234".to_string(), 100).await;

    let pong_reciever = Arc::new(Mutex::new(RequestObject::new(Box::new(move |caps: Vec<Option<Arc<Mutex<Capability>>>>| {
        info!("Received Pong");
        
        Ok(())
    })).await));

    let receiver_cap = service.create_capability().await;
    let _ = receiver_cap.lock().await.bind(pong_reciever).await;
    receiver_cap.lock().await.delegate("10.0.1.2:1234".into()).await.unwrap();

    for _ in 0..100 {
        let now = Instant::now();
        let r = pong_server.lock().await.request_invoke_with_continuation(vec!(receiver_cap.lock().await.cap_id)).await;
        info!("ret: {:?}: {:?}", r, now.elapsed());
    }
    service.terminate().await;
    let _ = service_thread.await;
}
