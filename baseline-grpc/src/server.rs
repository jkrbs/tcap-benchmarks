pub mod simple {
    tonic::include_proto!("simple");
}

use log::debug;
use simple::ping_pong_server::{PingPong, PingPongServer};
use simple::{Request as PingRequest, Response as PongResponse};
use tonic::{transport::Server, Request, Response, Status};

#[derive(Default)]
pub struct PP {}

#[tonic::async_trait]
impl PingPong for PP {
    async fn pong(&self, request: Request<PingRequest>) -> Result<Response<PongResponse>, Status> {
        let resp = PongResponse {
            buf: request.get_ref().buf,
        };
        debug!("received request");
        Ok(Response::new(resp))
    }
}

pub async fn server(addr: String) {
    let pp = PP::default();

    Server::builder()
        .add_service(PingPongServer::new(pp))
        .serve(addr.parse().unwrap())
        .await
        .unwrap();
}
