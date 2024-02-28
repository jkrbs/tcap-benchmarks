use tcap::{capabilities::tcap::CapType, service::tcap::Service};
use log::error;
pub async fn client(iterations: u128, service: Service, remote: String, test: bool, size: usize) {
    let end_cap = service.create_remote_capability_with_id(remote.clone(), 300).await;

    for _ in 0..iterations {
        let mem_cap = service.create_remote_capability_with_id(remote.clone(), 200).await;
        mem_cap.lock().await.cap_type = CapType::Memory;
        let buf = mem_cap.lock().await.get_buffer().await;
        if test {
            let mut expected = 0;
            buf.lock().await.data().iter().for_each(|x| {
                if *x != expected {
                    error!("error in transmission. val != 0");
                }
                expected += 3;
            });
        }
        service.delete_capability(mem_cap).await;

    }

    end_cap.lock().await.request_invoke_no_wait().await.unwrap();
}