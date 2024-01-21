use std::future::Future;
use std::ops::{AddAssign, Deref};
use std::{clone, sync::Arc, time::Duration};

use log::*;
use simple_logger::SimpleLogger;
use tcap::{
    capabilities::tcap::Capability, config::Config, object::tcap::object::RequestObject,
    service::tcap::Service,
};
use tokio::sync::Notify;
use tokio::{sync::Mutex, time::Instant};

use tokio::runtime::Handle;

use lazy_static::lazy_static;

lazy_static! {
    static ref COUNTER: Mutex<u128> = Mutex::new(0 as u128);
}

pub async fn star_benchmark(steps: u128, services: Vec<Service>) {
    let small_return_handler = Arc::new(Mutex::new(
        RequestObject::new(Box::new(
            move |caps: Vec<Option<Arc<Mutex<Capability>>>>| {
                let handler = async move |caps: tcap::tcap::HandlerParameters| {
                    COUNTER.lock().await.add_assign(1);
                };

                tokio::runtime::Handle::current().spawn(handler(caps));

                return Ok::<(), ()>(());
            },
        ))
        .await,
    ));

    let small_return_handler_cap = services[0].create_capability().await;
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
    let final_cap = services[0].create_capability().await;
    final_cap.lock().await.bind_req(final_handler).await;

    let star_handler = Arc::new(Mutex::new(
        RequestObject::new(Box::new(
            move |caps: Vec<Option<Arc<Mutex<Capability>>>>| {
                info!("Executing Start Lambda");

                let handler = async move |caps: Vec<Option<Arc<Mutex<Capability>>>>| {
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
                };

                tokio::runtime::Handle::current().spawn(handler(caps));

                Ok(())
            },
        ))
        .await,
    ));

    let start = services[1].create_capability().await;
    start.lock().await.bind_req(star_handler).await;
    small_return_handler_cap
        .lock()
        .await
        .delegate("127.0.0.1:1235".into())
        .await
        .unwrap();
    final_cap
        .lock()
        .await
        .delegate("127.0.0.1:1235".into())
        .await
        .unwrap();

    let star_remote = services[0]
        .create_remote_capability_with_id("127.0.0.1:1235".into(), start.lock().await.cap_id)
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
}
