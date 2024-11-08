pub(crate) mod sd;

use crate::error;
use hyper::{Body, Request, Response};

pub(crate) async fn handle_sd_request(req: Request<Body>) -> Response<Body> {
    match req.uri().path() {
        "/v1/images/generations" => sd::image_generation_handler(req).await,
        "/v1/images/edits" => sd::image_edit_handler(req).await,
        "/v1/images/variations" => sd::image_variation_handler(req).await,
        "/v1/files" => sd::files_handler(req).await,
        path => {
            if path.starts_with("/v1/files/") {
                sd::files_handler(req).await
            } else {
                error::invalid_endpoint(path)
            }
        }
    }
}
