use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use tokio::runtime;

use crate::err::Error;
use crate::routes::respond_to_request;

pub fn run(addr: &SocketAddr) -> Result<(), Error> {
    let mut runtime = runtime::Builder::new()
        .basic_scheduler()
        .enable_all()
        .build()?;

    let state = Arc::default();
    let make_svc = make_service_fn(move |_| {
        let state = Arc::clone(&state);
        let svc = service_fn(move |req| {
            let state = Arc::clone(&state);
            async move { respond_to_request(req, &state).await }
        });
        async move { Ok::<_, Infallible>(svc) }
    });

    let server = runtime.enter(|| Server::try_bind(&addr))?.serve(make_svc);

    runtime.block_on(server)?;

    Ok(())
}
