use crate::{error, utils::gen_image_id};
use endpoints::{
    files::FileObject,
    images::{ImageCreateRequest, ImageEditRequest, ImageVariationRequest, ResponseFormat},
};
use hyper::{body::to_bytes, Body, Method, Request, Response};
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

    let res = if req.method() == Method::POST {
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
        let mut image_request: ImageCreateRequest = match serde_json::from_slice(&body_bytes) {
            Ok(image_request) => image_request,
            Err(e) => {
                let err_msg = format!("Fail to deserialize image create request: {msg}", msg = e);

                // log
                error!(target: "stdout", "{}", &err_msg);

                return error::bad_request(err_msg);
            }
        };

        // check if the user id is provided
        if image_request.user.is_none() {
            image_request.user = Some(gen_image_id())
        };
        let id = image_request.user.clone().unwrap();

        // log user id
        info!(target: "stdout", "user: {}", image_request.user.clone().unwrap());

        match llama_core::images::image_generation(&mut image_request).await {
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
        }
    } else {
        let err_msg = "Invalid HTTP Method.";

        // log
        error!(target: "stdout", "{}", &err_msg);

        error::internal_server_error(err_msg)
    };

    // log
    info!(target: "stdout", "Send the image generation response.");

    res
}

pub(crate) async fn image_edit_handler(req: Request<Body>) -> Response<Body> {
    // log
    info!(target: "stdout", "Handling the coming image generation request");

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
