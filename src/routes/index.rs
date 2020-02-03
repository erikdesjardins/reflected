use hyper::{Body, Request, Response};

use crate::body::ArcBody;
use crate::err::Error;
use crate::routes::State;

pub async fn get(req: Request<Body>, state: &State) -> Result<Response<ArcBody>, Error> {
    let files = state.files.read().await;
    log::info!("GET {} -> [listing {} files]", req.uri(), files.len());
    let files_listing = files
        .iter()
        .map(|(path, file)| {
            format!(
                "<div><a href='{path}'>{path}</a> ({len} bytes)</div>",
                path = path,
                len = file.len()
            )
        })
        .collect::<Vec<_>>()
        .join("");
    Ok(Response::new(ArcBody::new(format!(
        concat!(
            "<!DOCTYPE html>",
            "<html>",
            "<head></head>",
            "<body>",
            "<p>visit a path to upload a file</p>",
            "{files_listing}",
            "</body>",
            "</html>",
        ),
        files_listing = files_listing
    ))))
}
