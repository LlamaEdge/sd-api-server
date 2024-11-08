use crate::{error, utils::gen_image_id, SOCKET_ADDRESS};
use endpoints::{
    files::{DeleteFileStatus, FileObject},
    images::{ImageCreateRequest, ImageEditRequest, ImageVariationRequest, ResponseFormat},
};
use hyper::{body::to_bytes, header::CONTENT_TYPE, Body, Method, Request, Response};
use multipart::server::{Multipart, ReadEntry, ReadEntryResult};
use multipart_2021 as multipart;
use std::{
    fs::{self, File},
    io::{Cursor, Read, Write},
    path::Path,
    time::SystemTime,
};

pub(crate) async fn image_generation_handler(mut req: Request<Body>) -> Response<Body> {
    // log
    info!(target: "stdout", "Handling the coming image generation request");

    let scheme_str = match is_https(&req) {
        true => "https",
        false => "http",
    };

    if req.method().eq(&hyper::http::Method::OPTIONS) {
        let result = Response::builder()
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "*")
            .header("Access-Control-Allow-Headers", "*")
            .header("Content-Type", "application/json")
            .body(Body::empty());

        match result {
            Ok(response) => return response,
            Err(e) => {
                let err_msg = e.to_string();

                // log
                error!(target: "stdout", "{}", &err_msg);

                return error::internal_server_error(err_msg);
            }
        }
    }

    let content_type = req
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|ct| ct.to_str().ok());

    if let Some(content_type) = content_type {
        if content_type.starts_with("multipart/") {
            // Handle multipart request
            info!(target: "stdout", "Handling multipart request");
            // Your multipart handling code here
        } else {
            // Handle command request
            info!(target: "stdout", "Handling command request");
            // Your command handling code here
        }
    } else {
        // Handle request with no Content-Type header
        info!(target: "stdout", "Handling request with no Content-Type header");
        // Your handling code here
    }

    let mut image_request = match content_type {
        Some(content_type) if content_type.starts_with("multipart/") => {
            let boundary = "boundary=";

            let boundary = req.headers().get("content-type").and_then(|ct| {
                let ct = ct.to_str().ok()?;
                let idx = ct.find(boundary)?;
                Some(ct[idx + boundary.len()..].to_string())
            });

            let req_body = req.into_body();
            let body_bytes = match to_bytes(req_body).await {
                Ok(body_bytes) => body_bytes,
                Err(e) => {
                    let err_msg = format!("Fail to read buffer from request body. {}", e);

                    // log
                    error!(target: "stdout", "{}", &err_msg);

                    return error::internal_server_error(err_msg);
                }
            };

            let cursor = Cursor::new(body_bytes.to_vec());

            let mut multipart = Multipart::with_body(cursor, boundary.unwrap());

            let mut image_request = ImageCreateRequest::default();
            while let ReadEntryResult::Entry(mut field) = multipart.read_entry_mut() {
                match &*field.headers.name {
                    "prompt" => match field.is_text() {
                        true => {
                            let mut prompt = String::new();

                            if let Err(e) = field.data.read_to_string(&mut prompt) {
                                let err_msg = format!("Failed to read the prompt. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }

                            image_request.prompt = prompt;
                        }
                        false => {
                            let err_msg =
                                "Failed to get the prompt. The prompt field in the request should be a text field.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    },
                    "negative_prompt" => match field.is_text() {
                        true => {
                            let mut negative_prompt = String::new();

                            if let Err(e) = field.data.read_to_string(&mut negative_prompt) {
                                let err_msg = format!("Failed to read the prompt. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }

                            image_request.prompt = negative_prompt;
                        }
                        false => {
                            let err_msg =
                                "Failed to get the negative prompt. The negative prompt field in the request should be a text field.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    },
                    "model" => match field.is_text() {
                        true => {
                            let mut model = String::new();

                            if let Err(e) = field.data.read_to_string(&mut model) {
                                let err_msg = format!("Failed to read the model. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }

                            image_request.model = model;
                        }
                        false => {
                            let err_msg =
                                "Failed to get the model name. The model field in the request should be a text field.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    },
                    "n" => match field.is_text() {
                        true => {
                            let mut n = String::new();

                            if let Err(e) = field.data.read_to_string(&mut n) {
                                let err_msg = format!("Failed to read the number of images. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }

                            match n.parse::<u64>() {
                                Ok(n) => image_request.n = Some(n),
                                Err(e) => {
                                    let err_msg = format!(
                                        "Failed to parse the number of images. Reason: {}",
                                        e
                                    );

                                    // log
                                    error!(target: "stdout", "{}", &err_msg);

                                    return error::bad_request(err_msg);
                                }
                            }
                        }
                        false => {
                            let err_msg =
                            "Failed to get the number of images. The n field in the request should be a text field.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    },
                    "size" => {
                        match field.is_text() {
                            true => {
                                let mut size = String::new();

                                if let Err(e) = field.data.read_to_string(&mut size) {
                                    let err_msg = format!("Failed to read the size. {}", e);

                                    // log
                                    error!(target: "stdout", "{}", &err_msg);

                                    return error::internal_server_error(err_msg);
                                }

                                // image_request.size = Some(size);

                                let parts: Vec<&str> = size.split('x').collect();
                                if parts.len() != 2 {
                                    let err_msg = "Invalid size format. The correct format is `HeightxWidth`. Example: 256x256";

                                    // log
                                    error!(target: "stdout", "{}", err_msg);

                                    return error::bad_request(err_msg);
                                }
                                image_request.height = Some(parts[0].parse().unwrap());
                                image_request.width = Some(parts[1].parse().unwrap());
                            }
                            false => {
                                let err_msg =
                                "Failed to get the size. The size field in the request should be a text field.";

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }
                        }
                    }
                    "response_format" => match field.is_text() {
                        true => {
                            let mut response_format = String::new();

                            if let Err(e) = field.data.read_to_string(&mut response_format) {
                                let err_msg = format!("Failed to read the response format. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }

                            match response_format.parse::<ResponseFormat>() {
                                Ok(format) => image_request.response_format = Some(format),
                                Err(e) => {
                                    let err_msg = format!(
                                        "Failed to parse the response format. Reason: {}",
                                        e
                                    );

                                    // log
                                    error!(target: "stdout", "{}", &err_msg);

                                    return error::bad_request(err_msg);
                                }
                            }
                        }
                        false => {
                            let err_msg =
                                "Failed to get the response format. The response format field in the request should be a text field.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    },
                    "user" => match field.is_text() {
                        true => {
                            let mut user = String::new();

                            if let Err(e) = field.data.read_to_string(&mut user) {
                                let err_msg = format!("Failed to read the user. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }

                            image_request.user = Some(user);
                        }
                        false => {
                            let err_msg =
                                "Failed to get the user. The user field in the request should be a text field.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    },
                    "cfg_scale" => match field.is_text() {
                        true => {
                            let mut cfg_scale = String::new();

                            if let Err(e) = field.data.read_to_string(&mut cfg_scale) {
                                let err_msg = format!("Failed to read the cfg_config. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }

                            match cfg_scale.parse::<f32>() {
                                Ok(scale) => image_request.cfg_scale = Some(scale),
                                Err(e) => {
                                    let err_msg = format!(
                                        "Failed to parse the number of images. Reason: {}",
                                        e
                                    );

                                    // log
                                    error!(target: "stdout", "{}", &err_msg);

                                    return error::bad_request(err_msg);
                                }
                            }
                        }
                        false => {
                            let err_msg =
                                "Failed to get the cfg_config. The cfg_config field in the request should be a text field.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    },
                    "sample_method" => match field.is_text() {
                        true => {
                            let mut sample_method = String::new();

                            if let Err(e) = field.data.read_to_string(&mut sample_method) {
                                let err_msg = format!("Failed to read the sample_method. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }

                            image_request.sample_method = Some(sample_method.as_str().into());
                        }
                        false => {
                            let err_msg =
                                "Failed to get the sample_method. The sample_method field in the request should be a text field.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    },
                    "steps" => match field.is_text() {
                        true => {
                            let mut steps = String::new();

                            if let Err(e) = field.data.read_to_string(&mut steps) {
                                let err_msg = format!("Failed to read the steps. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }

                            match steps.parse::<usize>() {
                                Ok(steps) => image_request.steps = Some(steps),
                                Err(e) => {
                                    let err_msg =
                                        format!("Failed to parse the steps. Reason: {}", e);

                                    // log
                                    error!(target: "stdout", "{}", &err_msg);

                                    return error::bad_request(err_msg);
                                }
                            }
                        }
                        false => {
                            let err_msg =
                                "Failed to get the steps. The steps field in the request should be a text field.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    },
                    "height" => match field.is_text() {
                        true => {
                            let mut height = String::new();

                            if let Err(e) = field.data.read_to_string(&mut height) {
                                let err_msg = format!("Failed to read the height. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }

                            match height.parse::<usize>() {
                                Ok(height) => image_request.height = Some(height),
                                Err(e) => {
                                    let err_msg =
                                        format!("Failed to parse the height. Reason: {}", e);

                                    // log
                                    error!(target: "stdout", "{}", &err_msg);

                                    return error::bad_request(err_msg);
                                }
                            }
                        }
                        false => {
                            let err_msg =
                                "Failed to get the height. The height field in the request should be a text field.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    },
                    "width" => match field.is_text() {
                        true => {
                            let mut width = String::new();

                            if let Err(e) = field.data.read_to_string(&mut width) {
                                let err_msg = format!("Failed to read the width. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }

                            match width.parse::<usize>() {
                                Ok(width) => image_request.width = Some(width),
                                Err(e) => {
                                    let err_msg =
                                        format!("Failed to parse the width. Reason: {}", e);

                                    // log
                                    error!(target: "stdout", "{}", &err_msg);

                                    return error::bad_request(err_msg);
                                }
                            }
                        }
                        false => {
                            let err_msg =
                                "Failed to get the width. The width field in the request should be a text field.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    },
                    "control_strength" => match field.is_text() {
                        true => {
                            let mut control_strength = String::new();

                            if let Err(e) = field.data.read_to_string(&mut control_strength) {
                                let err_msg = format!("Failed to read the control_strength. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }

                            match control_strength.parse::<f32>() {
                                Ok(control_strength) => {
                                    image_request.control_strength = Some(control_strength)
                                }
                                Err(e) => {
                                    let err_msg = format!(
                                        "Failed to parse the control_strength. Reason: {}",
                                        e
                                    );

                                    // log
                                    error!(target: "stdout", "{}", &err_msg);

                                    return error::bad_request(err_msg);
                                }
                            }
                        }
                        false => {
                            let err_msg =
                                "Failed to get the control_strength. The control_strength field in the request should be a text field.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    },
                    "control_image" => {
                        let filename = match field.headers.filename {
                            Some(filename) => filename,
                            None => {
                                let err_msg =
                                    "Failed to upload the image file. The filename is not provided.";

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }
                        };

                        // get the image data
                        let mut buffer = Vec::new();
                        let size_in_bytes = match field.data.read_to_end(&mut buffer) {
                            Ok(size_in_bytes) => size_in_bytes,
                            Err(e) => {
                                let err_msg = format!("Failed to read the image file. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }
                        };

                        // create a file id for the image file
                        let id = format!("file_{}", uuid::Uuid::new_v4());

                        // save the file
                        let path = Path::new("archives");
                        if !path.exists() {
                            fs::create_dir(path).unwrap();
                        }
                        let file_path = path.join(&id);
                        if !file_path.exists() {
                            fs::create_dir(&file_path).unwrap();
                        }
                        let mut file = match File::create(file_path.join(&filename)) {
                            Ok(file) => file,
                            Err(e) => {
                                let err_msg = format!(
                                    "Failed to create archive document {}. {}",
                                    &filename, e
                                );

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }
                        };
                        file.write_all(&buffer[..]).unwrap();

                        // log
                        info!(target: "stdout", "file_id: {}, file_name: {}, size in bytes: {}", &id, &filename, size_in_bytes);

                        let created_at =
                            match SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                                Ok(n) => n.as_secs(),
                                Err(_) => {
                                    let err_msg = "Failed to get the current time.";

                                    // log
                                    error!(target: "stdout", "{}", err_msg);

                                    return error::internal_server_error(err_msg);
                                }
                            };

                        // create a file object
                        image_request.control_image = Some(FileObject {
                            id,
                            bytes: size_in_bytes as u64,
                            created_at,
                            filename,
                            object: "file".to_string(),
                            purpose: "assistants".to_string(),
                        });
                    }
                    "seed" => match field.is_text() {
                        true => {
                            let mut seed = String::new();

                            if let Err(e) = field.data.read_to_string(&mut seed) {
                                let err_msg = format!("Failed to read the seed. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }

                            match seed.parse::<i32>() {
                                Ok(seed) => image_request.seed = Some(seed),
                                Err(e) => {
                                    let err_msg =
                                        format!("Failed to parse the seed. Reason: {}", e);

                                    // log
                                    error!(target: "stdout", "{}", &err_msg);

                                    return error::bad_request(err_msg);
                                }
                            }
                        }
                        false => {
                            let err_msg =
                                "Failed to get the seed. The seed field in the request should be a text field.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    },
                    unsupported_field => {
                        let err_msg = format!("Unsupported field: {}", unsupported_field);

                        // log
                        error!(target: "stdout", "{}", &err_msg);

                        return error::bad_request(err_msg);
                    }
                }
            }

            image_request
        }
        _ => {
            if req.method() == Method::POST {
                info!(target: "stdout", "Prepare the image generation request.");

                // parse request
                let body_bytes = match to_bytes(req.body_mut()).await {
                    Ok(body_bytes) => body_bytes,
                    Err(e) => {
                        let err_msg = format!("Fail to read buffer from request body. {}", e);

                        // log
                        error!(target: "stdout", "{}", &err_msg);

                        return error::internal_server_error(err_msg);
                    }
                };
                let image_request: ImageCreateRequest = match serde_json::from_slice(&body_bytes) {
                    Ok(image_request) => image_request,
                    Err(e) => {
                        let err_msg =
                            format!("Fail to deserialize image create request: {msg}", msg = e);

                        // log
                        error!(target: "stdout", "{}", &err_msg);

                        return error::bad_request(err_msg);
                    }
                };

                image_request
            } else {
                let err_msg = "Invalid HTTP Method.";

                // log
                error!(target: "stdout", "{}", &err_msg);

                return error::internal_server_error(err_msg);
            }
        }
    };

    if image_request.user.is_none() {
        image_request.user = Some(gen_image_id())
    };
    let id = image_request.user.clone().unwrap();

    // log user id
    info!(target: "stdout", "user: {}", image_request.user.clone().unwrap());

    let res = match llama_core::images::image_generation(&mut image_request).await {
        Ok(mut images_response) => {
            if Some(ResponseFormat::Url) == image_request.response_format {
                for image_object in images_response.data.iter_mut() {
                    let segments: Vec<&str> =
                        image_object.url.as_ref().unwrap().split("/").collect();
                    match segments.as_slice() {
                        [_, _, id, ..] => {
                            // get the socket address of request
                            let socket_address = SOCKET_ADDRESS.get().unwrap();

                            image_object.url = Some(format!(
                                "{}://{}/v1/files/download/{}",
                                scheme_str, socket_address, id
                            ))
                        }
                        _ => {
                            let err_msg = "Failed to parse the url from the image response.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    }
                }
            }

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
                            error!(target: "stdout", "{}", &err_msg);

                            error::internal_server_error(err_msg)
                        }
                    }
                }
                Err(e) => {
                    let err_msg =
                        format!("Fail to serialize the `ListImagesResponse` instance. {}", e);

                    // log
                    error!(target: "stdout", "{}", &err_msg);

                    error::internal_server_error(err_msg)
                }
            }
        }
        Err(e) => {
            let err_msg = format!("Failed to get image generations. Reason: {}", e);

            // log
            error!(target: "stdout", "{}", &err_msg);

            error::internal_server_error(err_msg)
        }
    };

    // log
    info!(target: "stdout", "Send the image generation response.");

    res
}

pub(crate) async fn image_edit_handler(req: Request<Body>) -> Response<Body> {
    // log
    info!(target: "stdout", "Handling the coming image generation request");

    let scheme_str = match is_https(&req) {
        true => "https",
        false => "http",
    };

    if req.method().eq(&hyper::http::Method::OPTIONS) {
        let result = Response::builder()
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "*")
            .header("Access-Control-Allow-Headers", "*")
            .header("Content-Type", "application/json")
            .body(Body::empty());

        match result {
            Ok(response) => return response,
            Err(e) => {
                let err_msg = e.to_string();

                // log
                error!(target: "stdout", "{}", &err_msg);

                return error::internal_server_error(err_msg);
            }
        }
    }

    let res = match *req.method() {
        Method::POST => {
            let boundary = "boundary=";

            let boundary = req.headers().get("content-type").and_then(|ct| {
                let ct = ct.to_str().ok()?;
                let idx = ct.find(boundary)?;
                Some(ct[idx + boundary.len()..].to_string())
            });

            let req_body = req.into_body();
            let body_bytes = match to_bytes(req_body).await {
                Ok(body_bytes) => body_bytes,
                Err(e) => {
                    let err_msg = format!("Fail to read buffer from request body. {}", e);

                    // log
                    error!(target: "stdout", "{}", &err_msg);

                    return error::internal_server_error(err_msg);
                }
            };

            let cursor = Cursor::new(body_bytes.to_vec());

            let mut multipart = Multipart::with_body(cursor, boundary.unwrap());

            let mut image_request = ImageEditRequest::default();
            while let ReadEntryResult::Entry(mut field) = multipart.read_entry_mut() {
                match &*field.headers.name {
                    "image" => {
                        let filename = match field.headers.filename {
                            Some(filename) => filename,
                            None => {
                                let err_msg =
                                    "Failed to upload the image file. The filename is not provided.";

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }
                        };

                        // get the image data
                        let mut buffer = Vec::new();
                        let size_in_bytes = match field.data.read_to_end(&mut buffer) {
                            Ok(size_in_bytes) => size_in_bytes,
                            Err(e) => {
                                let err_msg = format!("Failed to read the image file. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }
                        };

                        // create a file id for the image file
                        let id = format!("file_{}", uuid::Uuid::new_v4());

                        // save the file
                        let path = Path::new("archives");
                        if !path.exists() {
                            fs::create_dir(path).unwrap();
                        }
                        let file_path = path.join(&id);
                        if !file_path.exists() {
                            fs::create_dir(&file_path).unwrap();
                        }
                        let mut file = match File::create(file_path.join(&filename)) {
                            Ok(file) => file,
                            Err(e) => {
                                let err_msg = format!(
                                    "Failed to create archive document {}. {}",
                                    &filename, e
                                );

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }
                        };
                        file.write_all(&buffer[..]).unwrap();

                        // log
                        info!(target: "stdout", "file_id: {}, file_name: {}, size in bytes: {}", &id, &filename, size_in_bytes);

                        let created_at =
                            match SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                                Ok(n) => n.as_secs(),
                                Err(_) => {
                                    let err_msg = "Failed to get the current time.";

                                    // log
                                    error!(target: "stdout", "{}", err_msg);

                                    return error::internal_server_error(err_msg);
                                }
                            };

                        // create a file object
                        image_request.image = FileObject {
                            id,
                            bytes: size_in_bytes as u64,
                            created_at,
                            filename,
                            object: "file".to_string(),
                            purpose: "assistants".to_string(),
                        };
                    }
                    "prompt" => match field.is_text() {
                        true => {
                            let mut prompt = String::new();

                            if let Err(e) = field.data.read_to_string(&mut prompt) {
                                let err_msg = format!("Failed to read the prompt. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }

                            image_request.prompt = prompt;
                        }
                        false => {
                            let err_msg =
                                "Failed to get the prompt. The prompt field in the request should be a text field.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    },
                    "negative_prompt" => match field.is_text() {
                        true => {
                            let mut negative_prompt = String::new();

                            if let Err(e) = field.data.read_to_string(&mut negative_prompt) {
                                let err_msg = format!("Failed to read the prompt. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }

                            image_request.prompt = negative_prompt;
                        }
                        false => {
                            let err_msg =
                                "Failed to get the negative prompt. The negative prompt field in the request should be a text field.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    },
                    "mask" => {
                        let filename = match field.headers.filename {
                            Some(filename) => filename,
                            None => {
                                let err_msg =
                                    "Failed to upload the image mask file. The filename is not provided.";

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }
                        };

                        // get the image data
                        let mut buffer = Vec::new();
                        let size_in_bytes = match field.data.read_to_end(&mut buffer) {
                            Ok(size_in_bytes) => size_in_bytes,
                            Err(e) => {
                                let err_msg = format!("Failed to read the image mask file. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }
                        };

                        // create a file id for the image file
                        let id = format!("file_{}", uuid::Uuid::new_v4());

                        // save the file
                        let path = Path::new("archives");
                        if !path.exists() {
                            fs::create_dir(path).unwrap();
                        }
                        let file_path = path.join(&id);
                        if !file_path.exists() {
                            fs::create_dir(&file_path).unwrap();
                        }
                        let mut file = match File::create(file_path.join(&filename)) {
                            Ok(file) => file,
                            Err(e) => {
                                let err_msg = format!(
                                    "Failed to create archive document {}. {}",
                                    &filename, e
                                );

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }
                        };
                        file.write_all(&buffer[..]).unwrap();

                        // log
                        info!(target: "stdout", "file_id: {}, file_name: {}, size in bytes: {}", &id, &filename, size_in_bytes);

                        let created_at =
                            match SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                                Ok(n) => n.as_secs(),
                                Err(_) => {
                                    let err_msg = "Failed to get the current time.";

                                    // log
                                    error!(target: "stdout", "{}", err_msg);

                                    return error::internal_server_error(err_msg);
                                }
                            };

                        // create a file object
                        image_request.mask = Some(FileObject {
                            id,
                            bytes: size_in_bytes as u64,
                            created_at,
                            filename,
                            object: "file".to_string(),
                            purpose: "assistants".to_string(),
                        });
                    }
                    "model" => match field.is_text() {
                        true => {
                            let mut model = String::new();

                            if let Err(e) = field.data.read_to_string(&mut model) {
                                let err_msg = format!("Failed to read the model. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }

                            image_request.model = model;
                        }
                        false => {
                            let err_msg =
                                "Failed to get the model name. The model field in the request should be a text field.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    },
                    "n" => match field.is_text() {
                        true => {
                            let mut n = String::new();

                            if let Err(e) = field.data.read_to_string(&mut n) {
                                let err_msg = format!("Failed to read the number of images. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }

                            match n.parse::<u64>() {
                                Ok(n) => image_request.n = Some(n),
                                Err(e) => {
                                    let err_msg = format!(
                                        "Failed to parse the number of images. Reason: {}",
                                        e
                                    );

                                    // log
                                    error!(target: "stdout", "{}", &err_msg);

                                    return error::bad_request(err_msg);
                                }
                            }
                        }
                        false => {
                            let err_msg =
                            "Failed to get the number of images. The n field in the request should be a text field.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    },
                    "size" => {
                        match field.is_text() {
                            true => {
                                let mut size = String::new();

                                if let Err(e) = field.data.read_to_string(&mut size) {
                                    let err_msg = format!("Failed to read the size. {}", e);

                                    // log
                                    error!(target: "stdout", "{}", &err_msg);

                                    return error::internal_server_error(err_msg);
                                }

                                // image_request.size = Some(size);

                                let parts: Vec<&str> = size.split('x').collect();
                                if parts.len() != 2 {
                                    let err_msg = "Invalid size format. The correct format is `HeightxWidth`. Example: 256x256";

                                    // log
                                    error!(target: "stdout", "{}", err_msg);

                                    return error::bad_request(err_msg);
                                }
                                image_request.height = Some(parts[0].parse().unwrap());
                                image_request.width = Some(parts[1].parse().unwrap());
                            }
                            false => {
                                let err_msg =
                                "Failed to get the size. The size field in the request should be a text field.";

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }
                        }
                    }
                    "response_format" => match field.is_text() {
                        true => {
                            let mut response_format = String::new();

                            if let Err(e) = field.data.read_to_string(&mut response_format) {
                                let err_msg = format!("Failed to read the response format. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }

                            match response_format.parse::<ResponseFormat>() {
                                Ok(format) => image_request.response_format = Some(format),
                                Err(e) => {
                                    let err_msg = format!(
                                        "Failed to parse the response format. Reason: {}",
                                        e
                                    );

                                    // log
                                    error!(target: "stdout", "{}", &err_msg);

                                    return error::bad_request(err_msg);
                                }
                            }
                        }
                        false => {
                            let err_msg =
                                "Failed to get the response format. The response format field in the request should be a text field.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    },
                    "user" => match field.is_text() {
                        true => {
                            let mut user = String::new();

                            if let Err(e) = field.data.read_to_string(&mut user) {
                                let err_msg = format!("Failed to read the user. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }

                            image_request.user = Some(user);
                        }
                        false => {
                            let err_msg =
                                "Failed to get the user. The user field in the request should be a text field.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    },
                    "cfg_scale" => match field.is_text() {
                        true => {
                            let mut cfg_scale = String::new();

                            if let Err(e) = field.data.read_to_string(&mut cfg_scale) {
                                let err_msg = format!("Failed to read the cfg_config. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }

                            match cfg_scale.parse::<f32>() {
                                Ok(scale) => image_request.cfg_scale = Some(scale),
                                Err(e) => {
                                    let err_msg = format!(
                                        "Failed to parse the number of images. Reason: {}",
                                        e
                                    );

                                    // log
                                    error!(target: "stdout", "{}", &err_msg);

                                    return error::bad_request(err_msg);
                                }
                            }
                        }
                        false => {
                            let err_msg =
                                "Failed to get the cfg_config. The cfg_config field in the request should be a text field.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    },
                    "sample_method" => match field.is_text() {
                        true => {
                            let mut sample_method = String::new();

                            if let Err(e) = field.data.read_to_string(&mut sample_method) {
                                let err_msg = format!("Failed to read the sample_method. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }

                            image_request.sample_method = Some(sample_method.as_str().into());
                        }
                        false => {
                            let err_msg =
                                "Failed to get the sample_method. The sample_method field in the request should be a text field.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    },
                    "steps" => match field.is_text() {
                        true => {
                            let mut steps = String::new();

                            if let Err(e) = field.data.read_to_string(&mut steps) {
                                let err_msg = format!("Failed to read the steps. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }

                            match steps.parse::<usize>() {
                                Ok(steps) => image_request.steps = Some(steps),
                                Err(e) => {
                                    let err_msg =
                                        format!("Failed to parse the steps. Reason: {}", e);

                                    // log
                                    error!(target: "stdout", "{}", &err_msg);

                                    return error::bad_request(err_msg);
                                }
                            }
                        }
                        false => {
                            let err_msg =
                                "Failed to get the steps. The steps field in the request should be a text field.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    },
                    "height" => match field.is_text() {
                        true => {
                            let mut height = String::new();

                            if let Err(e) = field.data.read_to_string(&mut height) {
                                let err_msg = format!("Failed to read the height. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }

                            match height.parse::<usize>() {
                                Ok(height) => image_request.height = Some(height),
                                Err(e) => {
                                    let err_msg =
                                        format!("Failed to parse the height. Reason: {}", e);

                                    // log
                                    error!(target: "stdout", "{}", &err_msg);

                                    return error::bad_request(err_msg);
                                }
                            }
                        }
                        false => {
                            let err_msg =
                                "Failed to get the height. The height field in the request should be a text field.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    },
                    "width" => match field.is_text() {
                        true => {
                            let mut width = String::new();

                            if let Err(e) = field.data.read_to_string(&mut width) {
                                let err_msg = format!("Failed to read the width. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }

                            match width.parse::<usize>() {
                                Ok(width) => image_request.width = Some(width),
                                Err(e) => {
                                    let err_msg =
                                        format!("Failed to parse the width. Reason: {}", e);

                                    // log
                                    error!(target: "stdout", "{}", &err_msg);

                                    return error::bad_request(err_msg);
                                }
                            }
                        }
                        false => {
                            let err_msg =
                                "Failed to get the width. The width field in the request should be a text field.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    },
                    "control_strength" => match field.is_text() {
                        true => {
                            let mut control_strength = String::new();

                            if let Err(e) = field.data.read_to_string(&mut control_strength) {
                                let err_msg = format!("Failed to read the control_strength. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }

                            match control_strength.parse::<f32>() {
                                Ok(control_strength) => {
                                    image_request.control_strength = Some(control_strength)
                                }
                                Err(e) => {
                                    let err_msg = format!(
                                        "Failed to parse the control_strength. Reason: {}",
                                        e
                                    );

                                    // log
                                    error!(target: "stdout", "{}", &err_msg);

                                    return error::bad_request(err_msg);
                                }
                            }
                        }
                        false => {
                            let err_msg =
                                "Failed to get the control_strength. The control_strength field in the request should be a text field.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    },
                    "control_image" => {
                        let filename = match field.headers.filename {
                            Some(filename) => filename,
                            None => {
                                let err_msg =
                                    "Failed to upload the image file. The filename is not provided.";

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }
                        };

                        // get the image data
                        let mut buffer = Vec::new();
                        let size_in_bytes = match field.data.read_to_end(&mut buffer) {
                            Ok(size_in_bytes) => size_in_bytes,
                            Err(e) => {
                                let err_msg = format!("Failed to read the image file. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }
                        };

                        // create a file id for the image file
                        let id = format!("file_{}", uuid::Uuid::new_v4());

                        // save the file
                        let path = Path::new("archives");
                        if !path.exists() {
                            fs::create_dir(path).unwrap();
                        }
                        let file_path = path.join(&id);
                        if !file_path.exists() {
                            fs::create_dir(&file_path).unwrap();
                        }
                        let mut file = match File::create(file_path.join(&filename)) {
                            Ok(file) => file,
                            Err(e) => {
                                let err_msg = format!(
                                    "Failed to create archive document {}. {}",
                                    &filename, e
                                );

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }
                        };
                        file.write_all(&buffer[..]).unwrap();

                        // log
                        info!(target: "stdout", "file_id: {}, file_name: {}, size in bytes: {}", &id, &filename, size_in_bytes);

                        let created_at =
                            match SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                                Ok(n) => n.as_secs(),
                                Err(_) => {
                                    let err_msg = "Failed to get the current time.";

                                    // log
                                    error!(target: "stdout", "{}", err_msg);

                                    return error::internal_server_error(err_msg);
                                }
                            };

                        // create a file object
                        image_request.control_image = Some(FileObject {
                            id,
                            bytes: size_in_bytes as u64,
                            created_at,
                            filename,
                            object: "file".to_string(),
                            purpose: "assistants".to_string(),
                        });
                    }
                    "seed" => match field.is_text() {
                        true => {
                            let mut seed = String::new();

                            if let Err(e) = field.data.read_to_string(&mut seed) {
                                let err_msg = format!("Failed to read the seed. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }

                            match seed.parse::<i32>() {
                                Ok(seed) => image_request.seed = Some(seed),
                                Err(e) => {
                                    let err_msg =
                                        format!("Failed to parse the seed. Reason: {}", e);

                                    // log
                                    error!(target: "stdout", "{}", &err_msg);

                                    return error::bad_request(err_msg);
                                }
                            }
                        }
                        false => {
                            let err_msg =
                                "Failed to get the seed. The seed field in the request should be a text field.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    },
                    "strength" => match field.is_text() {
                        true => {
                            let mut strength = String::new();

                            if let Err(e) = field.data.read_to_string(&mut strength) {
                                let err_msg = format!("Failed to read the strength. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }

                            match strength.parse::<f32>() {
                                Ok(strength) => {
                                    image_request.strength = Some(strength);
                                    info!(target: "stdout", "strength: {}", strength);
                                }
                                Err(e) => {
                                    let err_msg =
                                        format!("Failed to parse the strength. Reason: {}", e);

                                    // log
                                    error!(target: "stdout", "{}", &err_msg);

                                    return error::bad_request(err_msg);
                                }
                            }
                        }
                        false => {
                            let err_msg =
                                "Failed to get the strength. The strength field in the request should be a text field.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    },
                    unsupported_field => {
                        let err_msg = format!("Unsupported field: {}", unsupported_field);

                        // log
                        error!(target: "stdout", "{}", &err_msg);

                        return error::bad_request(err_msg);
                    }
                }
            }

            // log
            info!(target: "stdout", "image edit request: {:?}", &image_request);

            // check if the user id is provided
            if image_request.user.is_none() {
                image_request.user = Some(gen_image_id())
            };
            let id = image_request.user.clone().unwrap();

            // log user id
            info!(target: "stdout", "user: {}", image_request.user.clone().unwrap());

            match llama_core::images::image_edit(&mut image_request).await {
                Ok(mut images_response) => {
                    if Some(ResponseFormat::Url) == image_request.response_format {
                        for image_object in images_response.data.iter_mut() {
                            let segments: Vec<&str> =
                                image_object.url.as_ref().unwrap().split("/").collect();
                            match segments.as_slice() {
                                [_, _, id, ..] => {
                                    // get the socket address of request
                                    let socket_address = SOCKET_ADDRESS.get().unwrap();

                                    image_object.url = Some(format!(
                                        "{}://{}/v1/files/download/{}",
                                        scheme_str, socket_address, id
                                    ))
                                }
                                _ => {
                                    let err_msg =
                                        "Failed to parse the url from the image response.";

                                    // log
                                    error!(target: "stdout", "{}", &err_msg);

                                    return error::internal_server_error(err_msg);
                                }
                            }
                        }
                    }

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
                                    error!(target: "stdout", "{}", &err_msg);

                                    error::internal_server_error(err_msg)
                                }
                            }
                        }
                        Err(e) => {
                            let err_msg = format!(
                                "Fail to serialize the `ListImagesResponse` instance. {}",
                                e
                            );

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            error::internal_server_error(err_msg)
                        }
                    }
                }
                Err(e) => {
                    let err_msg = format!("Failed to get image edit result. Reason: {}", e);

                    // log
                    error!(target: "stdout", "{}", &err_msg);

                    error::internal_server_error(err_msg)
                }
            }
        }
        _ => error::method_not_allowed(req.method()),
    };

    // log
    info!(target: "stdout", "Send the image edit response.");

    res
}

pub(crate) async fn image_variation_handler(req: Request<Body>) -> Response<Body> {
    // log
    info!(target: "stdout", "Handling the coming image variation request");

    if req.method().eq(&hyper::http::Method::OPTIONS) {
        let result = Response::builder()
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "*")
            .header("Access-Control-Allow-Headers", "*")
            .header("Content-Type", "application/json")
            .body(Body::empty());

        match result {
            Ok(response) => return response,
            Err(e) => {
                let err_msg = e.to_string();

                // log
                error!(target: "stdout", "{}", &err_msg);

                return error::internal_server_error(err_msg);
            }
        }
    }

    let res = match *req.method() {
        Method::POST => {
            let boundary = "boundary=";

            let boundary = req.headers().get("content-type").and_then(|ct| {
                let ct = ct.to_str().ok()?;
                let idx = ct.find(boundary)?;
                Some(ct[idx + boundary.len()..].to_string())
            });

            let req_body = req.into_body();
            let body_bytes = match to_bytes(req_body).await {
                Ok(body_bytes) => body_bytes,
                Err(e) => {
                    let err_msg = format!("Fail to read buffer from request body. {}", e);

                    // log
                    error!(target: "stdout", "{}", &err_msg);

                    return error::internal_server_error(err_msg);
                }
            };

            let cursor = Cursor::new(body_bytes.to_vec());

            let mut multipart = Multipart::with_body(cursor, boundary.unwrap());

            let mut image_request = ImageVariationRequest::default();
            while let ReadEntryResult::Entry(mut field) = multipart.read_entry_mut() {
                match &*field.headers.name {
                    "image" => {
                        let filename = match field.headers.filename {
                            Some(filename) => filename,
                            None => {
                                let err_msg =
                                    "Failed to upload the image file. The filename is not provided.";

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }
                        };

                        // get the image data
                        let mut buffer = Vec::new();
                        let size_in_bytes = match field.data.read_to_end(&mut buffer) {
                            Ok(size_in_bytes) => size_in_bytes,
                            Err(e) => {
                                let err_msg = format!("Failed to read the image file. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }
                        };

                        // create a file id for the image file
                        let id = format!("file_{}", uuid::Uuid::new_v4());

                        // save the file
                        let path = Path::new("archives");
                        if !path.exists() {
                            fs::create_dir(path).unwrap();
                        }
                        let file_path = path.join(&id);
                        if !file_path.exists() {
                            fs::create_dir(&file_path).unwrap();
                        }
                        let mut file = match File::create(file_path.join(&filename)) {
                            Ok(file) => file,
                            Err(e) => {
                                let err_msg = format!(
                                    "Failed to create archive document {}. {}",
                                    &filename, e
                                );

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }
                        };
                        file.write_all(&buffer[..]).unwrap();

                        // log
                        info!(target: "stdout", "file_id: {}, file_name: {}, size in bytes: {}", &id, &filename, size_in_bytes);

                        let created_at =
                            match SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                                Ok(n) => n.as_secs(),
                                Err(_) => {
                                    let err_msg = "Failed to get the current time.";

                                    // log
                                    error!(target: "stdout", "{}", err_msg);

                                    return error::internal_server_error(err_msg);
                                }
                            };

                        // create a file object
                        image_request.image = FileObject {
                            id,
                            bytes: size_in_bytes as u64,
                            created_at,
                            filename,
                            object: "file".to_string(),
                            purpose: "assistants".to_string(),
                        };
                    }
                    "model" => match field.is_text() {
                        true => {
                            let mut model = String::new();

                            if let Err(e) = field.data.read_to_string(&mut model) {
                                let err_msg = format!("Failed to read the model. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }

                            image_request.model = model;
                        }
                        false => {
                            let err_msg =
                                "Failed to get the model name. The model field in the request should be a text field.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    },
                    "n" => match field.is_text() {
                        true => {
                            let mut n = String::new();

                            if let Err(e) = field.data.read_to_string(&mut n) {
                                let err_msg = format!("Failed to read the number of images. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }

                            match n.parse::<u64>() {
                                Ok(n) => image_request.n = Some(n),
                                Err(e) => {
                                    let err_msg = format!(
                                        "Failed to parse the number of images. Reason: {}",
                                        e
                                    );

                                    // log
                                    error!(target: "stdout", "{}", &err_msg);

                                    return error::bad_request(err_msg);
                                }
                            }
                        }
                        false => {
                            let err_msg =
                            "Failed to get the number of images. The n field in the request should be a text field.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    },
                    "response_format" => match field.is_text() {
                        true => {
                            let mut response_format = String::new();

                            if let Err(e) = field.data.read_to_string(&mut response_format) {
                                let err_msg = format!("Failed to read the response format. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }

                            match response_format.parse::<ResponseFormat>() {
                                Ok(format) => image_request.response_format = Some(format),
                                Err(e) => {
                                    let err_msg = format!(
                                        "Failed to parse the response format. Reason: {}",
                                        e
                                    );

                                    // log
                                    error!(target: "stdout", "{}", &err_msg);

                                    return error::bad_request(err_msg);
                                }
                            }
                        }
                        false => {
                            let err_msg =
                                "Failed to get the response format. The response format field in the request should be a text field.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    },
                    "size" => match field.is_text() {
                        true => {
                            let mut size = String::new();

                            if let Err(e) = field.data.read_to_string(&mut size) {
                                let err_msg = format!("Failed to read the size. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }

                            image_request.size = Some(size);
                        }
                        false => {
                            let err_msg =
                                "Failed to get the size. The size field in the request should be a text field.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    },
                    "user" => match field.is_text() {
                        true => {
                            let mut user = String::new();

                            if let Err(e) = field.data.read_to_string(&mut user) {
                                let err_msg = format!("Failed to read the user. {}", e);

                                // log
                                error!(target: "stdout", "{}", &err_msg);

                                return error::internal_server_error(err_msg);
                            }

                            image_request.user = Some(user);
                        }
                        false => {
                            let err_msg =
                                "Failed to get the user. The user field in the request should be a text field.";

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            return error::internal_server_error(err_msg);
                        }
                    },
                    _ => unimplemented!("unknown field"),
                }
            }

            // log
            info!(target: "stdout", "image variation request: {:?}", &image_request);

            // check if the user id is provided
            if image_request.user.is_none() {
                image_request.user = Some(gen_image_id())
            };
            let id = image_request.user.clone().unwrap();

            // log user id
            info!(target: "stdout", "user: {}", image_request.user.clone().unwrap());

            match llama_core::images::image_variation(&mut image_request).await {
                Ok(images_response) => {
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
                                    error!(target: "stdout", "{}", &err_msg);

                                    error::internal_server_error(err_msg)
                                }
                            }
                        }
                        Err(e) => {
                            let err_msg = format!(
                                "Fail to serialize the `ListImagesResponse` instance. {}",
                                e
                            );

                            // log
                            error!(target: "stdout", "{}", &err_msg);

                            error::internal_server_error(err_msg)
                        }
                    }
                }
                Err(e) => {
                    let err_msg = format!("Failed to get image edit result. Reason: {}", e);

                    // log
                    error!(target: "stdout", "{}", &err_msg);

                    error::internal_server_error(err_msg)
                }
            }
        }
        _ => error::method_not_allowed(req.method()),
    };

    // log
    info!(target: "stdout", "Send the image variation response.");

    res
}

/// Upload, download, retrieve and delete a file, or list all files.
///
/// - `POST /v1/files`: Upload a file.
/// - `GET /v1/files`: List all files.
/// - `GET /v1/files/{file_id}`: Retrieve a file by id.
/// - `GET /v1/files/{file_id}/content`: Retrieve the content of a file by id.
/// - `GET /v1/files/download/{file_id}`: Download a file by id.
/// - `DELETE /v1/files/{file_id}`: Delete a file by id.
///
pub(crate) async fn files_handler(req: Request<Body>) -> Response<Body> {
    // log
    info!(target: "stdout", "Handling the coming files request");

    let res = if req.method() == Method::POST {
        match llama_core::files::upload_file(req).await {
            Ok(fo) => {
                // serialize chat completion object
                let s = match serde_json::to_string(&fo) {
                    Ok(s) => s,
                    Err(e) => {
                        let err_msg = format!("Failed to serialize file object. {}", e);

                        // log
                        error!(target: "stdout", "{}", &err_msg);

                        return error::internal_server_error(err_msg);
                    }
                };

                // return response
                let result = Response::builder()
                    .header("Access-Control-Allow-Origin", "*")
                    .header("Access-Control-Allow-Methods", "*")
                    .header("Access-Control-Allow-Headers", "*")
                    .header("Content-Type", "application/json")
                    .body(Body::from(s));

                match result {
                    Ok(response) => response,
                    Err(e) => {
                        let err_msg = e.to_string();

                        // log
                        error!(target: "stdout", "{}", &err_msg);

                        error::internal_server_error(err_msg)
                    }
                }
            }
            Err(e) => {
                let err_msg = format!("{}", e);

                // log
                error!(target: "stdout", "{}", &err_msg);

                error::internal_server_error(err_msg)
            }
        }
    } else if req.method() == Method::GET {
        let uri_path = req.uri().path().trim_end_matches('/').to_lowercase();

        // Split the path into segments
        let segments: Vec<&str> = uri_path.split('/').collect();

        match segments.as_slice() {
            ["", "v1", "files"] => list_files(),
            ["", "v1", "files", file_id, "content"] => {
                if !file_id.starts_with("file_") {
                    let err_msg = format!("unsupported uri path: {}", uri_path);

                    // log
                    error!(target: "stdout", "{}", &err_msg);

                    return error::internal_server_error(err_msg);
                }

                retrieve_file_content(file_id)
            }
            ["", "v1", "files", file_id] => {
                if !file_id.starts_with("file_") {
                    let err_msg = format!("unsupported uri path: {}", uri_path);

                    // log
                    error!(target: "stdout", "{}", &err_msg);

                    return error::internal_server_error(err_msg);
                }

                retrieve_file(file_id)
            }
            ["", "v1", "files", "download", file_id] => download_file(file_id),
            _ => {
                let err_msg = format!("unsupported uri path: {}", uri_path);

                // log
                error!(target: "stdout", "{}", &err_msg);

                error::internal_server_error(err_msg)
            }
        }
    } else if req.method() == Method::DELETE {
        let id = req.uri().path().trim_start_matches("/v1/files/");
        let status = match llama_core::files::remove_file(id) {
            Ok(status) => status,
            Err(e) => {
                let err_msg = format!("Failed to delete the target file with id {}. {}", id, e);

                // log
                error!(target: "stdout", "{}", &err_msg);

                DeleteFileStatus {
                    id: id.into(),
                    object: "file".to_string(),
                    deleted: false,
                }
            }
        };

        // serialize status
        let s = match serde_json::to_string(&status) {
            Ok(s) => s,
            Err(e) => {
                let err_msg = format!(
                    "Failed to serialize the status of the file deletion operation. {}",
                    e
                );

                // log
                error!(target: "stdout", "{}", &err_msg);

                return error::internal_server_error(err_msg);
            }
        };

        // return response
        let result = Response::builder()
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "*")
            .header("Access-Control-Allow-Headers", "*")
            .header("Content-Type", "application/json")
            .body(Body::from(s));

        match result {
            Ok(response) => response,
            Err(e) => {
                let err_msg = e.to_string();

                // log
                error!(target: "stdout", "{}", &err_msg);

                error::internal_server_error(err_msg)
            }
        }
    } else if req.method() == Method::OPTIONS {
        let result = Response::builder()
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "*")
            .header("Access-Control-Allow-Headers", "*")
            .header("Content-Type", "application/json")
            .body(Body::empty());

        match result {
            Ok(response) => return response,
            Err(e) => {
                let err_msg = e.to_string();

                // log
                error!(target: "files_handler", "{}", &err_msg);

                return error::internal_server_error(err_msg);
            }
        }
    } else {
        let err_msg = "Invalid HTTP Method.";

        // log
        error!(target: "stdout", "{}", &err_msg);

        error::internal_server_error(err_msg)
    };

    info!(target: "stdout", "Send the files response");

    res
}

fn list_files() -> Response<Body> {
    match llama_core::files::list_files() {
        Ok(file_objects) => {
            // serialize chat completion object
            let s = match serde_json::to_string(&file_objects) {
                Ok(s) => s,
                Err(e) => {
                    let err_msg = format!("Failed to serialize file list. {}", e);

                    // log
                    error!(target: "stdout", "{}", &err_msg);

                    return error::internal_server_error(err_msg);
                }
            };

            // return response
            let result = Response::builder()
                .header("Access-Control-Allow-Origin", "*")
                .header("Access-Control-Allow-Methods", "*")
                .header("Access-Control-Allow-Headers", "*")
                .header("Content-Type", "application/json")
                .body(Body::from(s));

            match result {
                Ok(response) => response,
                Err(e) => {
                    let err_msg = e.to_string();

                    // log
                    error!(target: "stdout", "{}", &err_msg);

                    error::internal_server_error(err_msg)
                }
            }
        }
        Err(e) => {
            let err_msg = format!("Failed to list all files. {}", e);

            // log
            error!(target: "stdout", "{}", &err_msg);

            error::internal_server_error(err_msg)
        }
    }
}

fn retrieve_file(id: impl AsRef<str>) -> Response<Body> {
    match llama_core::files::retrieve_file(id) {
        Ok(fo) => {
            // serialize chat completion object
            let s = match serde_json::to_string(&fo) {
                Ok(s) => s,
                Err(e) => {
                    let err_msg = format!("Failed to serialize file object. {}", e);

                    // log
                    error!(target: "stdout", "{}", &err_msg);

                    return error::internal_server_error(err_msg);
                }
            };

            // return response
            let result = Response::builder()
                .header("Access-Control-Allow-Origin", "*")
                .header("Access-Control-Allow-Methods", "*")
                .header("Access-Control-Allow-Headers", "*")
                .header("Content-Type", "application/json")
                .body(Body::from(s));

            match result {
                Ok(response) => response,
                Err(e) => {
                    let err_msg = e.to_string();

                    // log
                    error!(target: "stdout", "{}", &err_msg);

                    error::internal_server_error(err_msg)
                }
            }
        }
        Err(e) => {
            let err_msg = format!("{}", e);

            // log
            error!(target: "stdout", "{}", &err_msg);

            error::internal_server_error(err_msg)
        }
    }
}

fn retrieve_file_content(id: impl AsRef<str>) -> Response<Body> {
    match llama_core::files::retrieve_file_content(id) {
        Ok(content) => {
            // serialize chat completion object
            let s = match serde_json::to_string(&content) {
                Ok(s) => s,
                Err(e) => {
                    let err_msg = format!("Failed to serialize file content. {}", e);

                    // log
                    error!(target: "stdout", "{}", &err_msg);

                    return error::internal_server_error(err_msg);
                }
            };

            // return response
            let result = Response::builder()
                .header("Access-Control-Allow-Origin", "*")
                .header("Access-Control-Allow-Methods", "*")
                .header("Access-Control-Allow-Headers", "*")
                .header("Content-Type", "application/json")
                .body(Body::from(s));

            match result {
                Ok(response) => response,
                Err(e) => {
                    let err_msg = e.to_string();

                    // log
                    error!(target: "stdout", "{}", &err_msg);

                    error::internal_server_error(err_msg)
                }
            }
        }
        Err(e) => {
            let err_msg = format!("{}", e);

            // log
            error!(target: "stdout", "{}", &err_msg);

            error::internal_server_error(err_msg)
        }
    }
}

fn download_file(id: impl AsRef<str>) -> Response<Body> {
    match llama_core::files::download_file(id) {
        Ok((filename, buffer)) => {
            // get the extension of the file
            let extension = filename.split('.').last().unwrap_or("unknown");
            let content_type = match extension {
                "txt" => "text/plain",
                "json" => "application/json",
                "png" => "image/png",
                "jpg" => "image/jpeg",
                "jpeg" => "image/jpeg",
                "wav" => "audio/wav",
                "mp3" => "audio/mpeg",
                "mp4" => "video/mp4",
                "md" => "text/markdown",
                _ => {
                    let err_msg = format!("Unsupported file extension: {}", extension);

                    // log
                    error!(target: "stdout", "{}", &err_msg);

                    return error::internal_server_error(err_msg);
                }
            };
            let content_disposition = format!("attachment; filename={}", filename);

            // return response
            let result = Response::builder()
                .header("Access-Control-Allow-Origin", "*")
                .header("Access-Control-Allow-Methods", "*")
                .header("Access-Control-Allow-Headers", "*")
                .header("Content-Type", content_type)
                .header("Content-Disposition", content_disposition)
                .body(Body::from(buffer));

            match result {
                Ok(response) => response,
                Err(e) => {
                    let err_msg = e.to_string();

                    // log
                    error!(target: "stdout", "{}", &err_msg);

                    error::internal_server_error(err_msg)
                }
            }
        }
        Err(e) => {
            let err_msg = format!("{}", e);

            // log
            error!(target: "stdout", "{}", &err_msg);

            error::internal_server_error(err_msg)
        }
    }
}

fn is_https(req: &Request<Body>) -> bool {
    // Check both URI scheme and forwarded proto header
    req.uri().scheme_str() == Some("https")
        || req
            .headers()
            .get("X-Forwarded-Proto")
            .and_then(|h| h.to_str().ok())
            == Some("https")
}
