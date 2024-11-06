#[macro_use]
extern crate log;

mod backend;
mod error;
mod utils;

use anyhow::Result;
use clap::{ArgGroup, Parser, ValueEnum};
use error::ServerError;
use hyper::{
    body::HttpBody,
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use std::{net::SocketAddr, path::PathBuf};
use tokio::net::TcpListener;
use utils::LogLevel;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

// default port
const DEFAULT_PORT: &str = "8080";

#[derive(Debug, Parser)]
#[command(name = "LlamaEdge-StableDiffusion API Server", version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"), about = "LlamaEdge-Stable-Diffusion API Server")]
#[command(group = ArgGroup::new("model_group").multiple(false).required(true).args(&["model", "diffusion_model"]))]
#[command(group = ArgGroup::new("socket_address_group").multiple(false).args(&["socket_addr", "port"]))]
struct Cli {
    /// Sets the model name.
    #[arg(short, long, required = true)]
    model_name: String,
    /// Path to full model
    #[arg(long, default_value = "", group = "model_group")]
    model: String,
    /// Path to the standalone diffusion model file.
    #[arg(long, default_value = "", group = "model_group")]
    diffusion_model: String,
    /// Path to vae
    #[arg(long, default_value = "")]
    vae: String,
    /// Path to the clip-l text encoder
    #[arg(long, default_value = "")]
    clip_l: String,
    /// Path to the the t5xxl text encoder
    #[arg(long, default_value = "")]
    t5xxl: String,
    /// Path to the lora model directory
    #[arg(long)]
    lora_model_dir: Option<String>,
    /// Path to control net model
    #[arg(long)]
    control_net: Option<String>,
    /// Keep controlnet on cpu (for low vram)
    #[arg(long, default_value = "false")]
    control_net_cpu: bool,
    /// Number of threads to use during computation. Default is -1, which means to use all available threads.
    #[arg(long, default_value = "-1")]
    threads: i32,
    /// Keep clip on cpu (for low vram)
    #[arg(long, default_value = "false")]
    clip_on_cpu: bool,
    /// Keep vae on cpu (for low vram)
    #[arg(long, default_value = "false")]
    vae_on_cpu: bool,
    /// Task type.
    #[arg(long, default_value = "full")]
    task: TaskType,
    /// Socket address of LlamaEdge API Server instance. For example, `0.0.0.0:8080`.
    #[arg(long, default_value = None, value_parser = clap::value_parser!(SocketAddr), group = "socket_address_group")]
    socket_addr: Option<SocketAddr>,
    /// Port number
    #[arg(long, default_value = DEFAULT_PORT, value_parser = clap::value_parser!(u16), group = "socket_address_group")]
    port: u16,
}

#[allow(clippy::needless_return)]
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), ServerError> {
    // get the environment variable `LLAMA_LOG`
    let log_level: LogLevel = std::env::var("LLAMA_LOG")
        .unwrap_or("info".to_string())
        .parse()
        .unwrap_or(LogLevel::Info);

    // set global logger
    wasi_logger::Logger::install().expect("failed to install wasi_logger::Logger");
    log::set_max_level(log_level.into());

    // parse the command line arguments
    let cli = Cli::parse();

    // log the version of the server
    info!(target: "stdout", "server version: {}", env!("CARGO_PKG_VERSION"));

    if cli.model_name.is_empty() {
        return Err(ServerError::ArgumentError(
            "The value of the '--model-name' option should not be empty.".into(),
        ));
    }
    // log model name
    info!(target: "stdout", "model_name: {}", cli.model_name);

    // log task type
    info!(target: "stdout", "task: {:?}", &cli.task);

    // log lora model directory
    if let Some(lora_model_dir) = &cli.lora_model_dir {
        info!(target: "stdout", "lora_model_dir: {}", lora_model_dir);
    } else {
        info!(target: "stdout", "lora_model_dir: None");
    }

    // log control net
    if let Some(control_net) = &cli.control_net {
        info!(target: "stdout", "control_net: {}", &control_net);
    }

    // log control net on cpu
    info!(target: "stdout", "control_net_cpu: {}", cli.control_net_cpu);

    // log threads
    info!(target: "stdout", "threads: {}", cli.threads);

    // log clip on cpu
    info!(target: "stdout", "clip_on_cpu: {}", cli.clip_on_cpu);

    // log vae on cpu
    info!(target: "stdout", "vae_on_cpu: {}", cli.vae_on_cpu);

    // Determine which model option is set
    if !cli.model.is_empty() {
        info!(target: "stdout", "model: {}", &cli.model);

        // initialize the stable diffusion context
        llama_core::init_sd_context_with_full_model(
            &cli.model,
            cli.lora_model_dir.as_deref(),
            cli.control_net.as_deref(),
            cli.control_net_cpu,
            cli.clip_on_cpu,
            cli.vae_on_cpu,
            cli.threads,
            cli.task.to_sd_context_type(),
        )
        .map_err(|e| ServerError::Operation(format!("{}", e)))?;
    } else if !cli.diffusion_model.is_empty() {
        // if diffusion_model is not empty, check if diffusion_model is a valid path
        if !PathBuf::from(&cli.diffusion_model).exists() {
            return Err(ServerError::ArgumentError(
                "The path to the diffusion model does not exist.".into(),
            ));
        }
        info!(target: "stdout", "diffusion model: {}", &cli.diffusion_model);

        // if vae is not empty, check if vae is a valid path
        if !cli.vae.is_empty() && !PathBuf::from(&cli.vae).exists() {
            return Err(ServerError::ArgumentError(
                "The path to the vae does not exist.".into(),
            ));
        }
        info!(target: "stdout", "vae: {}", &cli.vae);

        // if clip_l is not empty, check if clip_l is a valid path
        if !cli.clip_l.is_empty() && !PathBuf::from(&cli.clip_l).exists() {
            return Err(ServerError::ArgumentError(
                "The path to the clip-l text encoder does not exist.".into(),
            ));
        }
        info!(target: "stdout", "clip_l: {}", &cli.clip_l);

        // if t5xxl is not empty, check if t5xxl is a valid path
        if !cli.t5xxl.is_empty() && !PathBuf::from(&cli.t5xxl).exists() {
            return Err(ServerError::ArgumentError(
                "The path to the t5xxl text encoder does not exist.".into(),
            ));
        }
        info!(target: "stdout", "t5xxl: {}", &cli.t5xxl);

        // initialize the stable diffusion context
        llama_core::init_sd_context_with_standalone_model(
            &cli.diffusion_model,
            &cli.vae,
            &cli.clip_l,
            &cli.t5xxl,
            cli.lora_model_dir.as_deref(),
            cli.control_net.as_deref(),
            cli.control_net_cpu,
            cli.clip_on_cpu,
            cli.vae_on_cpu,
            cli.threads,
            cli.task.to_sd_context_type(),
        )
        .map_err(|e| ServerError::Operation(format!("{}", e)))?;
    } else {
        return Err(ServerError::ArgumentError(
            "The '--model' or '--diffusion-model' option should be specified.".into(),
        ));
    }

    // socket address
    let addr = match cli.socket_addr {
        Some(addr) => addr,
        None => SocketAddr::from(([0, 0, 0, 0], cli.port)),
    };

    let new_service = make_service_fn(move |conn: &AddrStream| {
        // log socket address
        info!(target: "stdout", "remote_addr: {}, local_addr: {}", conn.remote_addr().to_string(), conn.local_addr().to_string());

        async move { Ok::<_, Error>(service_fn(handle_request)) }
    });

    let tcp_listener = TcpListener::bind(addr).await.unwrap();
    info!(target: "stdout", "Listening on {}", addr);

    let server = Server::from_tcp(tcp_listener.into_std().unwrap())
        .unwrap()
        .serve(new_service);

    match server.await {
        Ok(_) => Ok(()),
        Err(e) => Err(ServerError::Operation(e.to_string())),
    }
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let path_str = req.uri().path();
    let path_buf = PathBuf::from(path_str);
    let mut path_iter = path_buf.iter();
    path_iter.next(); // Must be Some(OsStr::new(&path::MAIN_SEPARATOR.to_string()))
    let root_path = path_iter.next().unwrap_or_default();
    let root_path = "/".to_owned() + root_path.to_str().unwrap_or_default();

    // log request
    {
        let method = hyper::http::Method::as_str(req.method()).to_string();
        let path = req.uri().path().to_string();
        let version = format!("{:?}", req.version());
        if req.method() == hyper::http::Method::POST {
            let size: u64 = match req.headers().get("content-length") {
                Some(content_length) => content_length.to_str().unwrap().parse().unwrap(),
                None => 0,
            };

            info!(target: "stdout", "method: {}, endpoint: {}, http_version: {}, content-length: {}", method, path, version, size);
        } else {
            info!(target: "stdout", "method: {}, endpoint: {}, http_version: {}", method, path, version);
        }
    }

    let response = match root_path.as_str() {
        "/echo" => Response::new(Body::from("echo test")),
        "/v1" => backend::handle_sd_request(req).await,
        _ => error::invalid_endpoint(root_path.as_str()),
    };

    // log response
    {
        let status_code = response.status();
        if status_code.as_u16() < 400 {
            // log response
            let response_version = format!("{:?}", response.version());
            let response_body_size: u64 = response.body().size_hint().lower();
            let response_status = status_code.as_u16();
            let response_is_informational = status_code.is_informational();
            let response_is_success = status_code.is_success();
            let response_is_redirection = status_code.is_redirection();
            let response_is_client_error = status_code.is_client_error();
            let response_is_server_error = status_code.is_server_error();

            info!(target: "stdout", "version: {}, body_size: {}, status: {}, is_informational: {}, is_success: {}, is_redirection: {}, is_client_error: {}, is_server_error: {}", response_version, response_body_size, response_status, response_is_informational, response_is_success, response_is_redirection, response_is_client_error, response_is_server_error);
        } else {
            let response_version = format!("{:?}", response.version());
            let response_body_size: u64 = response.body().size_hint().lower();
            let response_status = status_code.as_u16();
            let response_is_informational = status_code.is_informational();
            let response_is_success = status_code.is_success();
            let response_is_redirection = status_code.is_redirection();
            let response_is_client_error = status_code.is_client_error();
            let response_is_server_error = status_code.is_server_error();

            error!(target: "stdout", "version: {}, body_size: {}, status: {}, is_informational: {}, is_success: {}, is_redirection: {}, is_client_error: {}, is_server_error: {}", response_version, response_body_size, response_status, response_is_informational, response_is_success, response_is_redirection, response_is_client_error, response_is_server_error);
        }
    }

    Ok(response)
}

/// Task type.
#[derive(Clone, Debug, Copy, PartialEq, Eq, ValueEnum)]
enum TaskType {
    /// `text_to_image` task.
    #[value(name = "text2image")]
    TextToImage,
    /// `image_to_image` task.
    #[value(name = "image2image")]
    ImageToImage,
    /// `text_to_image` and `image_to_image` tasks.
    #[value(name = "full")]
    Full,
}
impl TaskType {
    fn to_sd_context_type(self) -> llama_core::StableDiffusionTask {
        match self {
            TaskType::TextToImage => llama_core::StableDiffusionTask::TextToImage,
            TaskType::ImageToImage => llama_core::StableDiffusionTask::ImageToImage,
            TaskType::Full => llama_core::StableDiffusionTask::Full,
        }
    }
}
