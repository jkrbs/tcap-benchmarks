use std::future::Future;
use std::ops::{AddAssign, MulAssign, Deref};
use std::{clone, sync::Arc, time::Duration};

use log::*;
use simple_logger::SimpleLogger;
use tcap::{
    capabilities::tcap::Capability, config::Config, object::tcap::object::RequestObject,
    service::tcap::Service,
};
use tokio::sync::Notify;
use tokio::time;
use tokio::{sync::Mutex, time::Instant};

use tokio::runtime::Handle;

use lazy_static::lazy_static;

lazy_static! {
    static ref COUNTER: Mutex<u128> = Mutex::new(0 as u128);
}

pub async fn star_benchmark_client(steps: u128, service: Service, remote: String) {
        COUNTER.lock().await.mul_assign(0);
        let small_return_handler = Arc::new(Mutex::new(
        RequestObject::new(Box::new(
            move |caps: Vec<Option<Arc<Mutex<Capability>>>>| {
                return Ok::<(), ()>(());
            },
        ))
        .await,
    ));

    let small_return_handler_cap = service.create_capability().await;
    small_return_handler_cap
        .lock()
        .await
        .bind_req(small_return_handler)
        .await;

    let notifier = Arc::new(Notify::new());

    let n = notifier.clone();
    let final_handler = Arc::new(Mutex::new(
        RequestObject::new(Box::new(move |cap: Vec<Option<Arc<Mutex<Capability>>>>| {
                n.notify_waiters();
            Ok(())
        }))
        .await,
    ));
    let final_cap = service.create_capability().await;
    final_cap.lock().await.bind_req(final_handler).await;

    small_return_handler_cap
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

    let star_remote = service
        .create_remote_capability_with_id(remote.as_str().into(), 100)
        .await;

    star_remote
        .lock()
        .await
        .request_invoke_with_continuation(vec![
            small_return_handler_cap.lock().await.cap_id,
            final_cap.lock().await.cap_id,
        ])
        .await
        .unwrap();

    notifier.notified().await;
    info!("Counter Value {:?}", COUNTER.lock().await);
}

pub async fn star_benchmark_server(steps: u128, service: Service, remote: String) {
    COUNTER.lock().await.mul_assign(0);
    let notifier = Arc::new(Notify::new());
    let n = notifier.clone();
    let start_handler = Arc::new(Mutex::new(
        RequestObject::new(Box::new(
            move |caps: Vec<Option<Arc<Mutex<Capability>>>>| {
                info!("Executing Start Lambda");

                let handler = async move |caps: Vec<Option<Arc<Mutex<Capability>>>>, notifier: Arc<Notify>| {
                    if caps[0].is_none() {
                        error!("error in cap transmission: {:?}", caps);
                    }
                    for _ in 0..steps {
                        caps[0]
                            .as_ref()
                            .unwrap()
                            .lock()
                            .await
                            .request_invoke()
                            .await
                            .unwrap();
                    }
                    caps[1]
                        .as_ref()
                        .unwrap()
                        .lock()
                        .await
                        .request_invoke()
                        .await
                        .unwrap();
                    notifier.notify_waiters();
                };

                let wait_for_cont_finish = Arc::new(Notify::new());
                tokio::runtime::Handle::current().spawn(handler(caps, wait_for_cont_finish.clone()));

                let _ = wait_for_cont_finish.notified();
                n.notify_waiters();
                Ok(())
            },
        ))
        .await,
    ));
    let start = service.create_capability_with_id(100).await;
    start.lock().await.bind_req(start_handler).await;

    notifier.notified().await;
}