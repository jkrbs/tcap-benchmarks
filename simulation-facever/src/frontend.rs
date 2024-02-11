use std::{sync::Arc, time::Duration};

use tcap::{capabilities::tcap::CapType, object::tcap::object::{MemoryObject, RequestObject}, service::tcap::Service};
use tokio::{sync::{Mutex, Notify}, time};
use log::info;
use crate::{CLIENT_TO_FRONEND_MEM_CAP, FRONTEND_CAP, FRONTEND_END_CAP, FRONTEND_TO_GPU_MEM_CAP, FRONTEND_TO_STORAGE_MEM_CAP, FS_CAP, GPU_CAP, GPU_TO_FRONTEND_MEM_CAP, STORAGE_CAP, STORAGE_TO_FRONEND_MEM_CAP};

pub(crate) async fn frontend(debug: bool, service: Service, transfer_size:u64, addresses: (String, String, String)) {
    let s = service.clone();
    let (fs_address, storage_address, gpu_address) = addresses.clone();
    

    let obj = Arc::new(Mutex::new(RequestObject::new(Box::new(move |caps: Vec<Option<Arc<Mutex<tcap::capabilities::tcap::Capability>>>>| {
        let notifier = Arc::new(Notify::new());
        let nnn = notifier.clone();

        let handler = async move |caps: Vec<Option<Arc<Mutex<tcap::capabilities::tcap::Capability>>>>,s: Service, transfer_size: u64, fs_address: String, storage_address: String, gpu_address: String, client_address: String| {
            info!("Running Frontend Handler");

            // let buf = [0 as u8; 1024];
            // let mem_obj = Arc::new(Mutex::new(MemoryObject::new(&buf).await));
            // let mem_cap = s.create_capability_with_id(FRONTEND_TO_GPU_MEM_CAP).await;
            // mem_cap.lock().await.bind_mem(mem_obj).await;
            // mem_cap.lock().await.delegate(gpu_address.as_str().into()).await.unwrap();

            // let buf = [0 as u8; 1024];
            // let mem_obj = Arc::new(Mutex::new(MemoryObject::new(&buf).await));
            // let mem_cap = s.create_capability_with_id(FRONTEND_TO_STORAGE_MEM_CAP).await;
            // mem_cap.lock().await.bind_mem(mem_obj).await;
            // mem_cap.lock().await.delegate(gpu_address.as_str().into()).await.unwrap();

            // info!("starting client mem transfer");

            // // copy client buffer to frontend
            // for _ in 0..transfer_size {
            //     let mem_cap = s.create_remote_capability_with_id(client_address.clone(), CLIENT_TO_FRONEND_MEM_CAP).await;
            //     mem_cap.lock().await.cap_type = CapType::Memory;
            //     let _mem_obj = mem_cap.lock().await.get_buffer().await;
            //     s.clone().delete_capability(mem_cap).await;
            // }
            info!("Data Copy Client -> Frontend finished");
            let fs_cap = s.create_remote_capability_with_id(fs_address.clone(), FS_CAP).await;
            let storage_cap = s.create_remote_capability_with_id(storage_address.clone(), STORAGE_CAP).await;
            let gpu_cap = s.create_remote_capability_with_id(gpu_address.clone(), GPU_CAP).await;

            // invoke fs
            let _ = fs_cap.lock().await.request_invoke().await.unwrap();
            info!("Finished FS Invocation");
            // invoke storage
            let _ = storage_cap.lock().await.request_invoke().await.unwrap();
            info!("Finished Storage Invocation");
            // copy buffer from storage
            for _ in 0..transfer_size {
                let mem_cap = s
                    .create_remote_capability_with_id(storage_address.clone(), STORAGE_TO_FRONEND_MEM_CAP)
                    .await;
                mem_cap.lock().await.cap_type = CapType::Memory;
                let _mem_obj = mem_cap.lock().await.get_buffer().await;
                s.clone().delete_capability(mem_cap).await;
            }
            info!("Data Copy Storage -> Frontend finished");
            // invoke gpu
            let _ = gpu_cap.lock().await.request_invoke().await.unwrap();
            info!("Finished GPU Invocation");
            // copy buffer from gpu
            for _ in 0..transfer_size {
                let mem_cap = s
                    .create_remote_capability_with_id(gpu_address.clone(), GPU_TO_FRONTEND_MEM_CAP)
                    .await;
                mem_cap.lock().await.cap_type = CapType::Memory;
                let _mem_obj = mem_cap.lock().await.get_buffer().await;
                s.clone().delete_capability(mem_cap).await;
            }
            info!("Data Copy GPU -> Frontend finished");
            nnn.notify_waiters();
        };
        tokio::runtime::Handle::current().block_on(handler(caps, 
            s.clone(),
            transfer_size,
            fs_address.clone(),
            storage_address.clone(),
            gpu_address.clone(),
            // assume client running on same service as FS
            fs_address.clone()
        ));
        info!("returning");
        Ok(())
    })).await));

    let cap = service.clone().create_capability_with_id(FRONTEND_CAP).await;
    cap.lock().await.bind_req(obj).await;

    let n = Arc::new(Notify::new());
    let not = n.clone();
    let final_handler = move |caps: tcap::tcap::HandlerParameters| {
        not.notify_waiters();
        Ok(())
    };
    let fin = Arc::new(Mutex::new(
        RequestObject::new(Box::new(final_handler)).await,
    ));

    let final_cap = service.create_capability_with_id(FRONTEND_END_CAP).await;
    final_cap.lock().await.bind_req(fin).await;

    info!("wiating 5 seconds for client to run");
    time::sleep(Duration::from_secs(5)).await;

    final_cap
    .lock()
    .await
    .delegate(addresses.0.clone().as_str().into())
    .await
    .unwrap();
    cap
    .lock()
    .await
    .delegate(addresses.0.clone().as_str().into())
    .await
    .unwrap();

    info!("waiting for notification");
    let _ = n.notified().await;
}