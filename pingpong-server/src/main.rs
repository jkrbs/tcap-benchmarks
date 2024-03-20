use std::sync::Arc;

use simple_logger::SimpleLogger;
use tcap::{service::tcap::Service, object::tcap::object::RequestObject, config::Config, capabilities::tcap::Capability};
use tokio::sync::Mutex;
use log::*;

use tokio::runtime::Handle;


#[tokio::main]
async fn main() {
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();

    let pong_handler = Arc::new(Mutex::new(
        RequestObject::new(Box::new(move |cap: Vec<Option<Arc<Mutex<Capability>>>>| {
            debug!("Executing Request Lambda");
            Ok(())
        }
        ))
        .await
    ));

    let service_config = Config {
        interface: "enp216s0f0".to_string(),
        address: "10.0.1.2:1234".to_string(),
        switch_addr: "10.0.9.2:1234".to_string(),
    };
    let service = Service::new(service_config).await;
    let s = service.clone();
    let service_thread = tokio::spawn(async move {
        let _ = s.run().await.unwrap();
    });

    let pong_cap = service.create_capability_with_id(100).await;
    let _ = pong_cap.lock().await.bind(pong_handler).await;
    
    let _ = service_thread.await;
}
