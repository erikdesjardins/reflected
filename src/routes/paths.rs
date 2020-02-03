use std::sync::Arc;

use hyper::header::HOST;
use hyper::{Body, Request, Response, StatusCode};

use crate::body::ArcBody;
use crate::err::Error;
use crate::file::write_to_mmap;
use crate::routes::State;

pub async fn get(req: Request<Body>, state: &State) -> Result<Response<ArcBody>, Error> {
    let file = state.files.read().await.get(req.uri().path()).cloned();
    Ok(match file {
        Some(file) => {
            log::info!("GET {} -> [found {} bytes]", req.uri(), file.len());
            Response::new(ArcBody::from_arc(file))
        }
        None => {
            log::info!("GET {} -> [not found]", req.uri());
            let path = req.uri().path().trim_start_matches('/');
            let host = req
                .headers()
                .get(HOST)
                .and_then(|h| h.to_str().ok())
                .unwrap_or("example.com");
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
            resp
        }
    })
}

pub async fn post(req: Request<Body>, state: &State) -> Result<Response<ArcBody>, Error> {
    log::info!("POST {} -> [start upload]", req.uri());
    let (parts, body) = req.into_parts();
    let file = write_to_mmap(body).await?;
    log::info!("POST {} -> [uploaded {} bytes]", parts.uri, file.len());
    state
        .files
        .write()
        .await
        .insert(parts.uri.path().to_string(), Arc::new(file));
    Ok(Response::new(ArcBody::empty()))
}

pub async fn delete(req: Request<Body>, state: &State) -> Result<Response<ArcBody>, Error> {
    let file = state.files.write().await.remove(req.uri().path());
    Ok(match file {
        Some(file) => {
            log::info!("DELETE {} -> [deleted {} bytes]", req.uri(), file.len());
            Response::new(ArcBody::empty())
        }
        None => {
            log::info!("DELETE {} -> [not found]", req.uri());
            let mut resp = Response::new(ArcBody::empty());
            *resp.status_mut() = StatusCode::NOT_FOUND;
            resp
        }
    })
}
