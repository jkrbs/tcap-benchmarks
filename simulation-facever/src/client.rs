use tokio::sync::{Mutex, Notify};
use std::sync::Arc;

use tcap::{object::tcap::object::{MemoryObject, RequestObject}, service::tcap::Service};
use log::info;
use crate::{CLIENT_TO_FRONEND_MEM_CAP, FRONTEND_CAP, FS_CAP};

pub(crate) async fn client(debug: bool, service: Service, frontend_address: String) {
    let buf = Vec::from([0 as u8; 1024]);
    let mem_obj = Arc::new(Mutex::new(MemoryObject::new(buf).await));
    let mem_cap = service.create_capability_with_id(CLIENT_TO_FRONEND_MEM_CAP).await;
    mem_cap.lock().await.bind_mem(mem_obj).await;
    // mem_cap.lock().await.delegate(frontend_address.as_str().into()).await.unwrap();
    let req_cap = service.create_remote_capability_with_id(frontend_address.clone(), FRONTEND_CAP).await;

    let notifier = Arc::new(Notify::new());
    let n = notifier.clone();
    let end_handler = Box::new(move |_caps| {
        n.notify_waiters();
        Ok(())
    });

    let end_obj = Arc::new(Mutex::new(RequestObject::new(end_handler).await));
    let end_cap = service.create_capability().await;
    end_cap.lock().await.bind_req(end_obj).await;
    end_cap.lock().await.delegate(frontend_address.as_str().into()).await.unwrap();
    info!("Sending Request");
    let _ = req_cap.lock().await.request_invoke_with_continuation(vec![end_cap.lock().await.cap_id]).await.unwrap();   

    notifier.notified().await;
}