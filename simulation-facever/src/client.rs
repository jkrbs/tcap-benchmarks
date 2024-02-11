use tokio::sync::Mutex;
use std::sync::Arc;

use tcap::{object::tcap::object::MemoryObject, service::tcap::Service};

use crate::{CLIENT_TO_FRONEND_MEM_CAP, FRONTEND_CAP};

pub(crate) async fn client(debug: bool, service: Service, frontend_address: String) {
    let buf = [0 as u8; 1024];
    let mem_obj = Arc::new(Mutex::new(MemoryObject::new(&buf).await));
    let mem_cap = service.create_capability_with_id(CLIENT_TO_FRONEND_MEM_CAP).await;
    mem_cap.lock().await.bind_mem(mem_obj).await;
    mem_cap.lock().await.delegate(frontend_address.as_str().into()).await.unwrap();
    let req_cap = service.create_remote_capability_with_id(frontend_address, FRONTEND_CAP).await;
    
    let _ = req_cap.lock().await.request_invoke().await.unwrap();   
}