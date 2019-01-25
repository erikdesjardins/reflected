use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

use failure::Error;
use futures::future::Either::{A, B};
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use log::{info, warn};
use memmap::Mmap;
use tokio::prelude::*;
use tokio::runtime::Runtime;

use crate::file::write_to_mmap;

pub fn run(addr: &SocketAddr) -> Result<(), Error> {
    let mut runtime = Runtime::new()?;

    let files: Arc<RwLock<BTreeMap<String, &'static Mmap>>> = Default::default();

    let server = Server::try_bind(&addr)?.serve(move || {
        let files = Arc::clone(&files);

        service_fn(move |req: Request<Body>| match *req.method() {
            Method::GET => {
                let file = files.read().unwrap().get(req.uri().path()).map(|file| *file);
                match file {
                    Some(file) => {
                        info!("GET  {} -> [found {} bytes]", req.uri(), file.len());
                        let resp = Response::new(Body::from(file.as_ref()));
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
                let resp = write_to_mmap(req.into_body())
                    .map(move |mmap| {
                        info!("POST {} -> [uploaded {} bytes]", uri, mmap.len());
                        files.write().unwrap().insert(uri.path().to_string(), Box::leak(Box::new(mmap)));
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