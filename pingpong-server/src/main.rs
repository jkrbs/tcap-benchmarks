use std::sync::Arc;

use simple_logger::SimpleLogger;
use tcap::{service::tcap::Service, object::tcap::object::RequestObject, config::Config, capabilities::tcap::Capability};
use tokio::sync::Mutex;
use log::*;

use tokio::runtime::Handle;


#[tokio::main]
async fn main() {
    SimpleLogger::new().init().unwrap();

    let pong_handler = Arc::new(Mutex::new(
        RequestObject::new(Box::new(move |cap: Option<Arc<Mutex<Capability>>>| {
            debug!("Executing Request Lambda");
            // if let Some(c) = cap {

            //     tokio::task::block_in_place(move || {
            //         Handle::current().block_on(async move {
            //             info!("invoke continuation");
            //             let _ = c.lock().await.request_invoke().await;
            //             info!("finished continuation");
            //         });
            //     });
            // }
            Ok(())
        }))
        .await,
    ));

    let service_config = Config {
        interface: "lo".to_string(),
        address: "127.0.0.1:1231".to_string(),
        switch_addr: "10.0.0.1".to_string(),
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
