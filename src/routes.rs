use std::collections::BTreeMap;
use std::sync::Arc;

use hyper::{Body, Method, Request, Response, StatusCode};
use memmap2::Mmap;
use tokio::sync::RwLock;

use crate::body::ArcBody;
use crate::err::Error;

mod index;
mod paths;

#[derive(Default)]
pub struct State {
    files: RwLock<BTreeMap<String, Arc<Mmap>>>,
}

pub async fn respond_to_request(
    req: Request<Body>,
    state: &State,
) -> Result<Response<ArcBody>, Error> {
    match *req.method() {
        Method::GET if req.uri().path() == "/" => index::get(req, state).await,
        Method::GET => paths::get(req, state).await,
        Method::POST => paths::post(req, state).await,
        Method::DELETE => paths::delete(req, state).await,
        _ => {
            log::warn!("{} {} -> [method not allowed]", req.method(), req.uri());
            let mut resp = Response::new(ArcBody::empty());
            *resp.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
            Ok(resp)
        }
    }
}
