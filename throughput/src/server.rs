use std::sync::Arc;
use tcap::service::tcap::Service;
use tcap::object::tcap::object::{MemoryObject, RequestObject};
use tokio::sync::{Mutex, Notify};

pub async fn server(service: Service) {
    let mut buf =  Vec::<u8>::new();
    let mut val = 0;
    for i in 0..2_u64.pow(12) {
        buf.push(val);
        val += 3;
    }

    let mem_reg = Arc::new(Mutex::new(MemoryObject::new(buf).await));
    let mem_cap = service.create_capability_with_id(200).await;
    mem_cap.lock().await.bind_mem(mem_reg).await;

    let notifier = Arc::new(Notify::new());
    let n = notifier.clone();
    let end_handler = Box::new(move |_caps| {
        n.notify_waiters();
        Ok(())
    });

    let end_obj = Arc::new(Mutex::new(RequestObject::new(end_handler).await));
    let end_cap = service.create_capability_with_id(300).await;
    end_cap.lock().await.bind_req(end_obj).await;

    notifier.notified().await;
}