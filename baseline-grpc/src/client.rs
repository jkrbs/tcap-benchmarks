use log::debug;
use simple::ping_pong_client::PingPongClient;
use simple::Request;
pub mod simple {
    tonic::include_proto!("simple");
}

pub async fn client(remote: String, iterations: u128) -> Result<(), ()> {
    let mut client = PingPongClient::connect(remote).await.unwrap();

    for _ in 0..iterations {
        let request = tonic::Request::new(Request { buf: 10 });

        let _ = client.pong(request).await.unwrap();
        debug!("received response to request");
    }

    Ok(())
}
