use std::{
    future::IntoFuture,
    ops::{Add, AddAssign},
    sync::Arc,
    time::Duration,
};

use log::{debug, info};
use tcap::{
    capabilities::tcap::{CapID, Capability},
    config::Config,
    object::tcap::object::RequestObject,
    service::tcap::Service,
};
use tokio::{
    runtime::Handle,
    sync::{Mutex, Notify},
    time::Instant,
};

use lazy_static::lazy_static;

lazy_static! {
    static ref COUNTER: Mutex<u128> = Mutex::new(0 as u128);
    static ref CAPID_ARRAY: Mutex<Vec<CapID>> = Mutex::new(Vec::new());
}

pub async fn chain_benchmark(steps: u128, services: Vec<Service>) {
    info!("Starting Chain Benchmark");
    let intermediate1_handler = move |caps: tcap::tcap::HandlerParameters| {
        let handler = async move |caps: tcap::tcap::HandlerParameters| {
            COUNTER.lock().await.add_assign(1);
            if COUNTER.lock().await.eq(&steps) {
                caps[0]
                    .as_ref()
                    .unwrap()
                    .lock()
                    .await
                    .request_invoke()
                    .await
            } else {
                caps[2]
                    .as_ref()
                    .unwrap()
                    .lock()
                    .await
                    .request_invoke_with_continuation(CAPID_ARRAY.lock().await.to_vec())
                    .await
            }
        };

        tokio::runtime::Handle::current().spawn(handler(caps));

        return Ok::<(), ()>(());
    };

    let intermediate2_handler = move |caps: tcap::tcap::HandlerParameters| {
        let handler = async move |caps: tcap::tcap::HandlerParameters| {
            COUNTER.lock().await.add_assign(1);
            if COUNTER.lock().await.eq(&steps) {
                let s = caps[0]
                    .as_ref()
                    .unwrap()
                    .lock()
                    .await
                    .service
                    .clone()
                    .unwrap();
                let fin = s
                    .lock()
                    .await
                    .create_remote_capability_with_id(
                        "127.0.0.1:1234".into(),
                        caps[0].as_ref().unwrap().lock().await.cap_id,
                    )
                    .await;
                let _ = fin.lock().await.request_invoke().await;
                Ok(())
            } else {
                caps[1]
                    .as_ref()
                    .unwrap()
                    .lock()
                    .await
                    .request_invoke_with_continuation(CAPID_ARRAY.lock().await.to_vec())
                    .await
            }
        };

        tokio::runtime::Handle::current().spawn(handler(caps));

        return Ok::<(), ()>(());
    };

    let start_handler = |caps: tcap::tcap::HandlerParameters| {
        let handler = async move |fun: Arc<Mutex<Capability>>,
                                  caps: tcap::tcap::HandlerParameters| {
            fun.lock()
                .await
                .request_invoke_with_continuation(CAPID_ARRAY.lock().await.to_vec())
                .await
                .unwrap();
        };

        tokio::runtime::Handle::current().spawn(handler(caps[1].clone().unwrap(), caps));
        Ok(())
    };

    let n = Arc::new(Notify::new());
    let not = n.clone();
    let final_handler = move |caps: tcap::tcap::HandlerParameters| {
        not.notify_waiters();
        Ok(())
    };

    let intermediate1 = Arc::new(Mutex::new(
        RequestObject::new(Box::new(intermediate1_handler)).await,
    ));
    let intermediate2 = Arc::new(Mutex::new(
        RequestObject::new(Box::new(intermediate2_handler)).await,
    ));
    let start = Arc::new(Mutex::new(
        RequestObject::new(Box::new(start_handler)).await,
    ));
    let fin = Arc::new(Mutex::new(
        RequestObject::new(Box::new(final_handler)).await,
    ));

    let intermediate1_cap = services[0].create_capability().await;
    intermediate1_cap.lock().await.bind_req(intermediate1).await;

    let intermediate2_cap = services[1].create_capability().await;
    intermediate2_cap.lock().await.bind_req(intermediate2).await;

    let start_cap = services[1].create_capability().await;
    start_cap.lock().await.bind_req(start).await;

    let final_cap = services[0].create_capability().await;
    final_cap.lock().await.bind_req(fin).await;

    //delegating all caps

    intermediate1_cap
        .lock()
        .await
        .delegate("127.0.0.1:1235".into())
        .await
        .unwrap();
    intermediate2_cap
        .lock()
        .await
        .delegate("127.0.0.1:1234".into())
        .await
        .unwrap();

    start_cap
        .lock()
        .await
        .delegate("127.0.0.1:1234".into())
        .await
        .unwrap();
    final_cap
        .lock()
        .await
        .delegate("127.0.0.1:1235".into())
        .await
        .unwrap();

    info!("Invoking start");
    let start_id = start_cap.lock().await.cap_id;

    let start0 = services[0]
        .create_remote_capability_with_id("127.0.0.1:1235".to_string(), start_id)
        .await;
    CAPID_ARRAY.lock().await.push(final_cap.lock().await.cap_id);
    CAPID_ARRAY
        .lock()
        .await
        .push(intermediate1_cap.lock().await.cap_id);
    CAPID_ARRAY
        .lock()
        .await
        .push(intermediate2_cap.lock().await.cap_id);
    start0
        .lock()
        .await
        .request_invoke_with_continuation(CAPID_ARRAY.lock().await.to_vec())
        .await
        .unwrap();

    let _ = n.notified().await;
}
