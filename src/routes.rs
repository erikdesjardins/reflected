use std::collections::BTreeMap;
use std::sync::Arc;

use hyper::header::HOST;
use hyper::{Body, Method, Request, Response, StatusCode};
use memmap::Mmap;
use tokio::sync::RwLock;

use crate::body::ArcBody;
use crate::err::Error;
use crate::file::write_to_mmap;

#[derive(Default)]
pub struct State {
    files: RwLock<BTreeMap<String, Arc<Mmap>>>,
}

pub async fn respond_to_request(
    req: Request<Body>,
    state: &State,
) -> Result<Response<ArcBody>, Error> {
    match *req.method() {
        Method::GET => {
            let file = state.files.read().await.get(req.uri().path()).cloned();
            match file {
                Some(file) => {
                    log::info!("GET {} -> [found {} bytes]", req.uri(), file.len());
                    let resp = Response::new(ArcBody::from_arc(file));
                    Ok(resp)
                }
                None => {
                    log::info!("GET {} -> [not found]", req.uri());
                    let path = match req.uri().path().trim_start_matches('/') {
                        "" => "file.txt",
                        p => p,
                    };
                    let host = match req.headers().get(HOST).and_then(|h| h.to_str().ok()) {
                        None => "example.com",
                        Some(h) => h,
                    };
                    let mut resp = Response::new(ArcBody::new(
                        format!(concat!(
                            "<!DOCTYPE html>",
                            "<html>",
                            "<head></head>",
                            "<body>",
                            "<code>curl -o /dev/null -X POST {host}/{path} --data-binary @- < {path}</code>",
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
                    Ok(resp)
                }
            }
        }
        Method::POST => {
            log::info!("POST {} -> [start upload]", req.uri());
            let (parts, body) = req.into_parts();
            // leaking appears to be the only (efficient) way to create a response,
            // since AsRef<[u8]>, i.e. from Arc<Mmap>, is not enough
            let file = write_to_mmap(body).await?;
            log::info!("POST {} -> [uploaded {} bytes]", parts.uri, file.len());
            state
                .files
                .write()
                .await
                .insert(parts.uri.path().to_string(), Arc::new(file));
            Ok(Response::new(ArcBody::empty()))
        }
        Method::DELETE => {
            let file = state.files.write().await.remove(req.uri().path());
            match file {
                Some(file) => {
                    log::info!("DELETE {} -> [deleted {} bytes]", req.uri(), file.len());
                    Ok(Response::new(ArcBody::empty()))
                }
                None => {
                    log::info!("DELETE {} -> [not found]", req.uri());
                    let mut resp = Response::new(ArcBody::empty());
                    *resp.status_mut() = StatusCode::NOT_FOUND;
                    Ok(resp)
                }
            }
        }
        _ => {
            log::warn!("{} {} -> [method not allowed]", req.method(), req.uri());
            let mut resp = Response::new(ArcBody::empty());
            *resp.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
            Ok(resp)
        }
    }
}
