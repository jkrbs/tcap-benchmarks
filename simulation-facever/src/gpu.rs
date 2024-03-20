use std::{sync::Arc, time::Duration};

use tcap::{object::tcap::object::{MemoryObject, RequestObject}, service::tcap::Service};
use tokio::{sync::Mutex, time};
use log::info;
use crate::{CPU_CLOCK_SPEED, FRONTEND_TO_GPU_MEM_CAP, GPU_CAP, GPU_TO_FRONTEND_MEM_CAP};

pub(crate) async fn gpu(
    debug: bool,
    service: Service,
    transfer_size: u64,
    frontend_address: String,
) {
    let frontend = frontend_address.clone();
    let buf = Vec::from([0 as u8; 2_usize.pow(12)]);
    let mem_obj = Arc::new(Mutex::new(MemoryObject::new(buf).await));
    let mem_cap = service.create_capability_with_id(GPU_TO_FRONTEND_MEM_CAP).await;
    mem_cap.lock().await.bind_mem(mem_obj).await;
    mem_cap.lock().await.delegate(frontend_address.clone().as_str().into()).await.unwrap();

    let s = service.clone();
    let obj = Arc::new(Mutex::new(
        RequestObject::new(Box::new(move |_| {
            std::thread::sleep(Duration::from_nanos(13 * transfer_size * CPU_CLOCK_SPEED));

            Ok(())
        }))
        .await,
    ));
    let cap = service.create_capability_with_id(GPU_CAP).await;
    cap.lock().await.bind_req(obj).await;

    cap.lock().await.delegate(frontend.as_str().into()).await.unwrap();
}
