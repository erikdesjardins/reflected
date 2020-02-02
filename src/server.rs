use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

use hyper::header::{HeaderValue, CONTENT_LENGTH, HOST};
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use tokio::prelude::future::Either::{A, B};
use tokio::prelude::*;
use tokio::runtime::Runtime;

use crate::err::Error;
use crate::file::write_to_mmap_and_leak;

pub fn run(addr: &SocketAddr) -> Result<(), Error> {
    let mut runtime = Runtime::new()?;

    let files: Arc<RwLock<BTreeMap<String, &'static [u8]>>> = Default::default();

    let server = Server::try_bind(&addr)?.serve(move || {
        let files = Arc::clone(&files);

        service_fn(move |req: Request<Body>| match *req.method() {
            Method::GET => {
                let file = files.read().unwrap().get(req.uri().path()).map(|file| *file);
                match file {
                    Some(file) => {
                        log::info!("GET  {} -> [found {} bytes]", req.uri(), file.len());
                        let mut resp = Response::new(Body::from(file));
                        resp.headers_mut().insert(CONTENT_LENGTH, HeaderValue::from_str(&file.len().to_string()).unwrap());
                        A(future::ok(resp))
                    }
                    None => {
                        log::info!("GET  {} -> [not found]", req.uri());
                        let path = match req.uri().path().trim_start_matches('/') {
                            "" => "file.txt",
                            p => p,
                        };
                        let host = match req.headers().get(HOST).and_then(|h| h.to_str().ok()) {
                            None => "example.com",
                            Some(h) => h,
                        };
                        let mut resp = Response::new(Body::from(
                            format!(concat!(
                                "<!DOCTYPE html>",
                                "<html>",
                                "<code>curl -Of -X POST {host}/{path} --data-binary @- < {path}</code>",
                                "<p/>",
                                "<span id='info'>or </span>",
                                "<input",
                                " type='file'",
                                " onchange=\"disabled = true, info.replaceWith('uploading...'), fetch(location, {{ method: 'POST', body: files[0] }}).then(() => this.replaceWith('done'))\"",
                                "/>",
                                "</html>",
                            ), path = path, host = host)
                        ));
                        *resp.status_mut() = StatusCode::NOT_FOUND;
                        A(future::ok(resp))
                    }
                }
            }
            Method::POST => {
                log::info!("POST {} -> [start upload]", req.uri());
                let uri = req.uri().clone();
                let files = Arc::clone(&files);
                // leaking appears to be the only (efficient) way to create a response,
                // since AsRef<[u8]>, i.e. from Arc<Mmap>, is not enough
                let resp = write_to_mmap_and_leak(req.into_body())
                    .map(move |file| {
                        log::info!("POST {} -> [uploaded {} bytes]", uri, file.len());
                        files.write().unwrap().insert(uri.path().to_string(), file);
                        Response::new(Body::empty())
                    });
                B(resp)
            }
            _ => {
                log::warn!("{} {} -> [method not allowed]", req.method(), req.uri());
                let mut resp = Response::new(Body::empty());
                *resp.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
                A(future::ok(resp))
            }
        })
    });

    runtime.block_on(server)?;

    Ok(())
}
