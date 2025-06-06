// test_hyper.rs
use hyper::server::Server;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let make_service = hyper::service::make_service_fn(|_| async {
        Ok::<_, hyper::Error>(hyper::service::service_fn(|_req| async {
            Ok::<_, hyper::Error>(hyper::Response::new(hyper::Body::from("Hello")))
        }))
    });
    Server::bind(&addr).serve(make_service).await.unwrap();
}