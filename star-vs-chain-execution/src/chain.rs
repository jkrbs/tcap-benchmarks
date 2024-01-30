use std::{
    future::IntoFuture,
    ops::{Add, AddAssign, MulAssign},
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
    static ref CLIENT: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    static ref SERVER: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
}

pub async fn chain_benchmark_client(steps: u128, service: Service, remote: String) {
    CLIENT.lock().await.clear();
    SERVER.lock().await.clear();
    CAPID_ARRAY.lock().await.clear();
    COUNTER.lock().await.mul_assign(0);
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
                    .request_invoke_with_continuation_no_wait(CAPID_ARRAY.lock().await.to_vec())
                    .await
            }
        };

        tokio::runtime::Handle::current().spawn(handler(caps));
        return Ok::<(), ()>(());
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

    let fin = Arc::new(Mutex::new(
        RequestObject::new(Box::new(final_handler)).await,
    ));

    let intermediate1_cap = service.create_capability_with_id(400).await;
    intermediate1_cap.lock().await.bind_req(intermediate1).await;

   

    let final_cap = service.create_capability_with_id(50).await;
    final_cap.lock().await.bind_req(fin).await;

    //delegating all caps

    intermediate1_cap
        .lock()
        .await
        .delegate(remote.as_str().into())
        .await
        .unwrap();



    final_cap
        .lock()
        .await
        .delegate(remote.as_str().into())
        .await
        .unwrap();

    let start0 = service
        .create_remote_capability_with_id(remote.clone(), 100)
        .await;
    let intermediate2_cap = service
        .create_remote_capability_with_id(remote, 200)
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
        .request_invoke_with_continuation_no_wait(CAPID_ARRAY.lock().await.to_vec())
        .await
        .unwrap();

    let _ = n.notified().await;
}

pub async fn chain_benchmark_server(steps: u128, service: Service, remote: String) {
    CLIENT.lock().await.clear();
    SERVER.lock().await.clear();
    COUNTER.lock().await.mul_assign(0);
    CAPID_ARRAY.lock().await.clear();

    let notifier = Arc::new(Notify::new());

    let n = notifier.clone();
    let intermediate2_handler = move |caps: tcap::tcap::HandlerParameters| {
        let handler = async move |caps: tcap::tcap::HandlerParameters, n: Arc<Notify>| {
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
                        CLIENT.lock().await.as_str().into(),
                        caps[0].as_ref().unwrap().lock().await.cap_id,
                    )
                    .await;
                let _ = fin.lock().await.request_invoke_no_wait().await;
                n.notify_waiters();
                Ok(())
            } else {
                caps[1]
                    .as_ref()
                    .unwrap()
                    .lock()
                    .await
                    .request_invoke_with_continuation_no_wait(CAPID_ARRAY.lock().await.to_vec())
                    .await
            }
        };

        tokio::runtime::Handle::current().spawn(handler(caps, n.clone()));
        return Ok::<(), ()>(());
    };
    let intermediate2 = Arc::new(Mutex::new(
        RequestObject::new(Box::new(intermediate2_handler)).await,
    ));
  
    let intermediate2_cap = service.create_capability_with_id(200).await;
    intermediate2_cap.lock().await.bind_req(intermediate2).await;


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
    let start = Arc::new(Mutex::new(
        RequestObject::new(Box::new(start_handler)).await,
    ));
    let start_cap = service.create_capability_with_id(100).await;
    start_cap.lock().await.bind_req(start).await;

   
    let final_cap = service.create_remote_capability_with_id(remote.clone(), 50).await;
    let intermediate1_cap = service.create_remote_capability_with_id(remote.clone(), 400).await;

    
    CLIENT.lock().await.push_str(remote.as_str());
    CAPID_ARRAY.lock().await.push(final_cap.lock().await.cap_id);
    CAPID_ARRAY
        .lock()
        .await
        .push(intermediate1_cap.lock().await.cap_id);
    CAPID_ARRAY
        .lock()
        .await
        .push(intermediate2_cap.lock().await.cap_id);

    notifier.notified().await;
}