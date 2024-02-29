use std::{sync::Arc, time::Duration};

use tcap::{object::tcap::object::RequestObject, service::tcap::Service};
use tokio::{sync::{Mutex, Notify}, time};
use log::info;
use crate::{CPU_CLOCK_SPEED, FS_CAP, FS_END_CAP};


pub(crate) async fn fs(debug: bool, service: Service, frontend: String) {
    let handler = Box::new(move |_| {
        info!("Running FS");
        time::sleep(Duration::from_nanos(10000)).is_elapsed();
        Ok(())
    });

    let obj = Arc::new(Mutex::new(RequestObject::new(handler).await));
    let cap = service.create_capability_with_id(FS_CAP).await;
    cap.lock().await.bind_req(obj).await;

    // cap.lock().await.delegate(frontend.as_str().into()).await.unwrap();

    let n = Arc::new(Notify::new());
    let not = n.clone();
    let final_handler = move |_caps: tcap::tcap::HandlerParameters| {
        info!("Killing FS Service");
        not.notify_waiters();
        Ok(())
    };
    let fin = Arc::new(Mutex::new(
        RequestObject::new(Box::new(final_handler)).await,
    ));

    let final_cap = service.create_capability_with_id(FS_END_CAP).await;
    final_cap.lock().await.bind_req(fin).await;
    info!("FS Service is available");
    n.notified().await;
}