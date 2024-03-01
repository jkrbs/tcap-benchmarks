use std::{sync::Arc, time::Duration};
use log::info;
use tcap::{object::tcap::object::MemoryObject, service::tcap::Service};

use tcap::object::tcap::object::RequestObject;
use tokio::{sync::Mutex, time};

use crate::{CPU_CLOCK_SPEED, FRONTEND_TO_STORAGE_MEM_CAP, STORAGE_CAP, STORAGE_TO_FRONEND_MEM_CAP};

pub(crate) async fn storage(
    debug: bool,
    service: Service,
    transfer_size: u64,
    frontend_address: String,
) {
    let frontend = frontend_address.clone();
    let buf = Vec::from([0 as u8; 1024]);
    let mem_obj = Arc::new(Mutex::new(MemoryObject::new(buf).await));
    let mem_cap = service.create_capability_with_id(STORAGE_TO_FRONEND_MEM_CAP).await;
    mem_cap.lock().await.bind_mem(mem_obj).await;
   // mem_cap.lock().await.delegate(frontend_address.clone().as_str().into()).await.unwrap();

    let s = service.clone();
    let obj = Arc::new(Mutex::new(
        RequestObject::new(Box::new(move |_| {
            std::thread::sleep(Duration::from_nanos(
                3 * transfer_size * CPU_CLOCK_SPEED
            ));

            Ok(())
        }))
        .await,
    ));
    let cap = service.create_capability_with_id(STORAGE_CAP).await;
    cap.lock().await.bind_req(obj).await;

    cap.lock().await.delegate(frontend.as_str().into()).await.unwrap();
}
