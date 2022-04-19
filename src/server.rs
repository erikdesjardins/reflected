use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use hyper::server::Server;
use hyper::service::{make_service_fn, service_fn};

use crate::err::Error;
use crate::routes::respond_to_request;

pub async fn run(addr: &SocketAddr) -> Result<(), Error> {
    let state = Arc::default();
    let make_svc = make_service_fn(move |_| {
        let state = Arc::clone(&state);
        let svc = service_fn(move |req| {
            let state = Arc::clone(&state);
            async move { respond_to_request(req, &state).await }
        });
        async move { Ok::<_, Infallible>(svc) }
    });

    Server::try_bind(addr)?.serve(make_svc).await?;

    Ok(())
}
