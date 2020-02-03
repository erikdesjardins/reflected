use std::collections::BTreeMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use hyper::header::{HeaderValue, CONTENT_LENGTH, HOST};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use tokio::runtime;
use tokio::sync::RwLock;

use crate::err::Error;
use crate::file::write_to_mmap_and_leak;

pub fn run(addr: &SocketAddr) -> Result<(), Error> {
    let mut runtime = runtime::Builder::new()
        .basic_scheduler()
        .enable_all()
        .build()?;

    let files: Arc<RwLock<BTreeMap<String, &'static [u8]>>> = Default::default();

    let make_svc = make_service_fn(move |_| {
        let files = Arc::clone(&files);
        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                let files = Arc::clone(&files);
                async move {
                    let resp = match *req.method() {
                        Method::GET => {
                            let file = files.read().await.get(req.uri().path()).copied();
                            match file {
                                Some(file) => {
                                    log::info!(
                                        "GET  {} -> [found {} bytes]",
                                        req.uri(),
                                        file.len()
                                    );
                                    let mut resp = Response::new(Body::from(file));
                                    resp.headers_mut().insert(
                                        CONTENT_LENGTH,
                                        HeaderValue::from_str(&file.len().to_string()).unwrap(),
                                    );
                                    resp
                                }
                                None => {
                                    log::info!("GET  {} -> [not found]", req.uri());
                                    let path = match req.uri().path().trim_start_matches('/') {
                                        "" => "file.txt",
                                        p => p,
                                    };
                                    let host =
                                        match req.headers().get(HOST).and_then(|h| h.to_str().ok())
                                        {
                                            None => "example.com",
                                            Some(h) => h,
                                        };
                                    let mut resp = Response::new(Body::from(
                                        format!(concat!(
                                            "<!DOCTYPE html>",
                                            "<html>",
                                            "<head></head>",
                                            "<body>",
                                            "<code>curl -Of -X POST {host}/{path} --data-binary @- < {path}</code>",
                                            "<p/>",
                                            "<span id='info'>or </span>",
                                            "<input",
                                            " type='file'",
                                            " onchange='disabled = true, info.replaceWith(`uploading...`), fetch(location, {{ method: `POST`, body: files[0] }}).then(() => this.replaceWith(`done`))'",
                                            "/>",
                                            "</body>",
                                            "</html>",
                                        ), path = path, host = host)
                                    ));
                                    *resp.status_mut() = StatusCode::NOT_FOUND;
                                    resp
                                }
                            }
                        }
                        Method::POST => {
                            log::info!("POST {} -> [start upload]", req.uri());
                            let (parts, body) = req.into_parts();
                            // leaking appears to be the only (efficient) way to create a response,
                            // since AsRef<[u8]>, i.e. from Arc<Mmap>, is not enough
                            let file = write_to_mmap_and_leak(body).await?;
                            log::info!("POST {} -> [uploaded {} bytes]", parts.uri, file.len());
                            files
                                .write()
                                .await
                                .insert(parts.uri.path().to_string(), file);
                            Response::new(Body::empty())
                        }
                        _ => {
                            log::warn!("{} {} -> [method not allowed]", req.method(), req.uri());
                            let mut resp = Response::new(Body::empty());
                            *resp.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
                            resp
                        }
                    };
                    Ok::<_, Error>(resp)
                }
            }))
        }
    });

    let server = runtime.enter(|| Server::try_bind(&addr))?.serve(make_svc);

    runtime.block_on(server)?;

    Ok(())
}
