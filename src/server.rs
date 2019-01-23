use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

use bytes::Bytes;
use failure::Error;
use futures::future::Either::{A, B};
use futures::{future, Future, Stream};
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use log::{info, warn};

use tokio::runtime::Runtime;

pub fn run(addr: &SocketAddr) -> Result<(), Error> {
    let mut runtime = Runtime::new()?;

    let files: Arc<RwLock<BTreeMap<String, Bytes>>> = Default::default();

    let server = Server::try_bind(&addr)?.serve(move || {
        let files = Arc::clone(&files);

        service_fn(move |req: Request<Body>| match *req.method() {
            Method::GET => {
                let file = files.read().unwrap().get(req.uri().path()).map(Bytes::clone);
                match file {
                    Some(file) => {
                        info!("GET  {} -> [found {} bytes]", req.uri(), file.len());
                        let resp = Response::new(Body::from(file));
                        A(future::ok(resp))
                    }
                    None => {
                        info!("GET  {} -> [not found]", req.uri());
                        let mut resp = Response::new(Body::from(
                            r#"<html><input type="file" onchange="fetch(location, { method: 'POST', body: files[0] }).then(() => this.replaceWith('Done'))"/></html>"#,
                        ));
                        *resp.status_mut() = StatusCode::NOT_FOUND;
                        A(future::ok(resp))
                    }
                }
            }
            Method::POST => {
                let uri = req.uri().clone();
                info!("POST {} -> [start upload]", uri);
                let files = Arc::clone(&files);
                let resp = req.into_body().concat2().map(move |file| {
                    info!("POST {} -> [uploaded {} bytes]", uri, file.len());
                    files.write().unwrap().insert(uri.path().to_string(), file.into_bytes());
                    Response::new(Body::empty())
                });
                B(resp)
            }
            _ => {
                warn!("{} {}", req.method(), req.uri());
                let mut resp = Response::new(Body::empty());
                *resp.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
                A(future::ok(resp))
            }
        })
    });

    runtime.block_on(server)?;

    Ok(())
}
