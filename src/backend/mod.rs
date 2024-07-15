pub(crate) mod sd;

use crate::error;
use hyper::{Body, Request, Response};

pub(crate) async fn handle_sd_request(req: Request<Body>) -> Response<Body> {
    match req.uri().path() {
        "/v1/images/generations" => sd::image_generation(req).await,
        "/v1/images/edits" => sd::image_edit(req).await,
        "/v1/images/variations" => sd::image_create_variation(req).await,
        path => error::invalid_endpoint(path),
    }
}
