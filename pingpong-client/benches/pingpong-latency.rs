use std::sync::Arc;

use simple_logger::SimpleLogger;
use tcap::capabilities::tcap::Capability;
use tcap::config::Config;
use tcap::service::tcap::Service;
use tcap::object::tcap::object::RequestObject;
use log::*;
use tokio::sync::Mutex;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
// This is a struct that tells Criterion.rs to use the "futures" crate's current-thread executor
use criterion::async_executor::FuturesExecutor;



fn pingpong_benchmark(c: &mut Criterion) { 
    let runtime = tokio::runtime::Builder::new_current_thread().enable_io()
    .build()
    .unwrap();

let (pong_server, receiver_cap, service_thread) = runtime.block_on(async {
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

    let pong_reciever = Arc::new(Mutex::new(RequestObject::new(Box::new(move |cap: Option<Arc<Mutex<Capability>>>| {
        info!("Received Pong");
        
        Ok(())
    })).await));

    let receiver_cap = service.create_capability().await;
    let _ = receiver_cap.lock().await.bind(pong_reciever).await;
    receiver_cap.lock().await.delegate("127.0.0.1:1231".into()).await.unwrap();
    let pong_server = service.create_remote_capability_with_id("127.0.0.1:1231".to_string(), 100).await;


    (pong_server, receiver_cap, service_thread)
});
    
    c.bench_function("1", |b|{
        b.to_async(FuturesExecutor).iter(|| async {
            let _ = pong_server.lock().await.request_invoke_with_continuation(Some(receiver_cap.clone())).await;
        });
    });
    runtime.block_on( async {
        let _ = service_thread.await;
    });
    
}

criterion_group!(benches, pingpong_benchmark);
criterion_main!(benches);