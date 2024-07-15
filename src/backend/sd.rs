use hyper::{body::to_bytes, Body, Method, Request, Response};

pub(crate) async fn image_generation(mut req: Request<Body>) -> Response<Body> {
    unimplemented!("image_generation")
}

pub(crate) async fn image_edit(mut req: Request<Body>) -> Response<Body> {
    unimplemented!("image_edit")
}

pub(crate) async fn image_create_variation(mut req: Request<Body>) -> Response<Body> {
    unimplemented!("image_create_variation")
}
