use std::{sync::Arc, time::Duration};

use tcap::{object::tcap::object::RequestObject, service::tcap::Service};
use tokio::{sync::Mutex, time};

use crate::{CPU_CLOCK_SPEED, FS_CAP};


pub(crate) async fn fs(debug: bool, service: Service, frontend: String) {
    let handler = Box::new(move |_| {
        time::sleep(Duration::from_secs(10000/CPU_CLOCK_SPEED)).is_elapsed();
        Ok(())
    });

    let obj = Arc::new(Mutex::new(RequestObject::new(handler).await));
    let cap = service.create_capability_with_id(FS_CAP).await;
    cap.lock().await.bind_req(obj).await;

    cap.lock().await.delegate(frontend.as_str().into()).await.unwrap();
}