use crate::{error, utils::gen_image_id};
use endpoints::images::ImageCreateRequest;
use hyper::{body::to_bytes, Body, Method, Request, Response};
use std::{
    pin::Pin,
    sync::Mutex,
    task::{Context, Poll},
    time::SystemTime,
};

pub(crate) async fn image_generation_handler(mut req: Request<Body>) -> Response<Body> {
    // log
    info!(target: "image_generation_handler", "Handling the coming image generation request");

    let res = if req.method() == Method::POST {
        info!(target: "image_generation_handler", "Prepare the image generation request.");

        // parse request
        let body_bytes = match to_bytes(req.body_mut()).await {
            Ok(body_bytes) => body_bytes,
            Err(e) => {
                let err_msg = format!("Fail to read buffer from request body. {}", e);

                // log
                error!(target: "image_generation_handler", "{}", &err_msg);

                return error::internal_server_error(err_msg);
            }
        };
        let mut image_request: ImageCreateRequest = match serde_json::from_slice(&body_bytes) {
            Ok(image_request) => image_request,
            Err(e) => {
                let err_msg = format!("Fail to deserialize image create request: {msg}", msg = e);

                // log
                error!(target: "chat_completions_handler", "{}", &err_msg);

                return error::bad_request(err_msg);
            }
        };

        // check if the user id is provided
        if image_request.user.is_none() {
            image_request.user = Some(gen_image_id())
        };
        let id = image_request.user.clone().unwrap();

        // log user id
        info!(target: "image_generation_handler", "user: {}", image_request.user.clone().unwrap());

        let res = match llama_core::images::image_generation(&mut image_request).await {
            Ok(images_response) => {
                // serialize embedding object
                match serde_json::to_string(&images_response) {
                    Ok(s) => {
                        // return response
                        let result = Response::builder()
                            .header("Access-Control-Allow-Origin", "*")
                            .header("Access-Control-Allow-Methods", "*")
                            .header("Access-Control-Allow-Headers", "*")
                            .header("Content-Type", "application/json")
                            .header("user", id)
                            .body(Body::from(s));
                        match result {
                            Ok(response) => response,
                            Err(e) => {
                                let err_msg = e.to_string();

                                // log
                                error!(target: "embeddings_handler", "{}", &err_msg);

                                error::internal_server_error(err_msg)
                            }
                        }
                    }
                    Err(e) => {
                        let err_msg = format!("Fail to serialize embedding object. {}", e);

                        // log
                        error!(target: "embeddings_handler", "{}", &err_msg);

                        error::internal_server_error(err_msg)
                    }
                }
            }
            Err(e) => {
                let err_msg = format!("Failed to get image generations. Reason: {}", e);

                // log
                error!(target: "image_generation_handler", "{}", &err_msg);

                error::internal_server_error(err_msg)
            }
        };

        res
    } else {
        let err_msg = "Invalid HTTP Method.";

        // log
        error!(target: "files_handler", "{}", &err_msg);

        error::internal_server_error(err_msg)
    };

    // log
    info!(target: "image_generation_handler", "Send the image generation response.");

    res
}

pub(crate) async fn image_edit_handler(mut _req: Request<Body>) -> Response<Body> {
    unimplemented!("image_edit")
}

pub(crate) async fn image_variation_handler(mut _req: Request<Body>) -> Response<Body> {
    unimplemented!("image_create_variation")
}
