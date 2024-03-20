use tokio::sync::{Mutex, Notify};
use std::sync::Arc;

use tcap::{object::tcap::object::{MemoryObject, RequestObject}, service::tcap::Service};
use log::{debug, info};
use crate::{CLIENT_TO_FRONEND_MEM_CAP, FRONTEND_CAP, FS_CAP, CLIENT_END_CAP};

pub(crate) async fn client(debug: bool, service: Service, frontend_address: String) {
    let req_cap = service.create_remote_capability_with_id(frontend_address.clone(), FRONTEND_CAP).await;

    let notifier = Arc::new(Notify::new());
    let n = notifier.clone();
    let end_handler = Box::new(move |_caps| {
        debug!("called end_handler");
        n.notify_waiters();
        Ok(())
    });

    let end_obj = Arc::new(Mutex::new(RequestObject::new(end_handler).await));
    let end_cap = service.create_capability().await;
    end_cap.lock().await.bind_req(end_obj).await;
    end_cap.lock().await.delegate(frontend_address.as_str().into()).await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_micros(20)).await;
    debug!("Sending Request");
    let _ = req_cap.lock().await.request_invoke_with_continuation(vec![end_cap.lock().await.cap_id]).await.unwrap();   

    notifier.notified().await;
    debug!("closing client");
}
