# Stable-Diffusion-API-Server

This project is a RESTful API server that provides image generation and editing services based on Stable Diffusion models. The APIs are compatible with OpenAI APIs of [image generation and editing](https://platform.openai.com/docs/api-reference/images).

> [!NOTE]
> The project is still under active development. The existing features still need to be improved and more features will be added in the future.

## Quick Start

### Setup

- Install WasmEdge v0.14.1-rc.4

  ```bash
  curl -sSf https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install_v2.sh | bash -s -- -v 0.14.1-rc.4
  ```

- Deply `wasmedge_stablediffusion` plugin

  > For the purpose of demonstration, we will use the stable diffusion plugin for Mac Apple Silicon. You can find the plugin for other platforms [Releases/0.14.1-rc.4](https://github.com/WasmEdge/WasmEdge/releases/tag/0.14.1-rc.4)

  ```bash
  # Download stable diffusion plugin for Mac Apple Silicon
  curl -LO https://github.com/WasmEdge/WasmEdge/releases/download/0.14.1-rc.4/WasmEdge-plugin-wasmedge_stablediffusion-0.14.1-rc.4-darwin_arm64.tar.gz

  # Unzip the plugin to $HOME/.wasmedge/plugin
  tar -xzf WasmEdge-plugin-wasmedge_stablediffusion-0.14.1-rc.4-darwin_arm64.tar.gz -C $HOME/.wasmedge/plugin

  # remove wasi_nn-ggml plugin if exists
  rm $HOME/.wasmedge/plugin/libwasmedgePluginWasiNN.dylib
  ```

### Run sd-api-server

- Download the stable diffusion model

  ```bash
  curl -LO https://huggingface.co/second-state/stable-diffusion-v-1-4-GGUF/resolve/main/stable-diffusion-v1-4-Q8_0.gguf
  ```

  The available stable diffusion models:

  - [second-state/stable-diffusion-v-1-4-GGUF](https://huggingface.co/second-state/stable-diffusion-v-1-4-GGUF)
  - [second-state/stable-diffusion-v1-5-GGUF](https://huggingface.co/second-state/stable-diffusion-v1-5-GGUF)
  - [second-state/stable-diffusion-2-1-GGUF](https://huggingface.co/second-state/stable-diffusion-2-1-GGUF)

- Download sd-api-server.wasm

  ```bash
  curl -LO https://github.com/LlamaEdge/sd-api-server/releases/latest/download/sd-api-server.wasm
  ```

- Start the server

  ```bash
  wasmedge --dir .:. sd-api-server.wasm --model-name sd-v1.4 --model stable-diffusion-v1-4-Q8_0.gguf
  ```

  > `sd-api-server` will use `8080` port by default. You can change the port by adding `--socket-addr <ip-address>:<port>`.

### Usage

#### Image Generation

- Send a request for image generation

  ```bash
  curl -X POST 'http://localhost:8080/v1/images/generations' \
    --header 'Content-Type: application/json' \
    --data '{
        "model": "sd-v1.4",
        "prompt": "A cute baby sea otter"
    }'
  ```

  If the request is handled successfully, the server will return a JSON response like the following:

  ```json
  {
    "created": 1723431133,
    "data": [
        {
            "url": "/archives/file_74f514a2-8d33-4f9d-bcc0-42e8db14ecbc/output.png",
            "prompt": "A cute baby sea otter"
        }
    ]
  }
  ```

- Preview the generated image

<div align=center>
<img src="image/otter.png" alt="A cute baby sea otter" width="60%" />
</div>

#### Image Editing

- Send a request for image editing

  ```bash
  curl --location 'http://localhost:10086/v1/images/edits' \
    --form 'image=@"otter.png"' \
    --form 'prompt="A cute baby sea otter with blue eyes"'
  ```

  If the request is handled successfully, the server will return a JSON response like the following:

  ```json
  {
    "created": 1723432689,
    "data": [
        {
            "url": "/archives/file_554e4d53-6072-4988-83e6-fe684655a734/output.png",
            "prompt": "A cute baby sea otter with blue eyes"
        }
    ]
  }
  ```

- Preview the edited image

<div align=center>
<img src="image/otter_blue_eyes.png" alt="A cute baby sea otter with blue eyes" width="60%" />
</div>

## Build

```bash
cargo build --target wasm32-wasip1 --release
```

`sd-api-server.wasm` will be generated in `target/wasm32-wasip1/release/`.

## CLI Options

```bash
$ wasmedge target/wasm32-wasip1/release/sd-api-server.wasm -h

LlamaEdge-Stable-Diffusion API Server

Usage: sd-api-server.wasm [OPTIONS] --model-name <MODEL_NAME> <--model <MODEL>|--diffusion-model <DIFFUSION_MODEL>>

Options:
  -m, --model-name <MODEL_NAME>
          Sets the model name
  -m, --model <MODEL>
          Path to full model [default: ]
      --diffusion-model <DIFFUSION_MODEL>
          Path to the standalone diffusion model file [default: ]
      --vae <VAE>
          Path to vae [default: ]
      --clip-l <CLIP_L>
          Path to the clip-l text encoder [default: ]
      --t5xxl <T5XXL>
          Path to the the t5xxl text encoder [default: ]
      --threads <THREADS>
          Number of threads to use during computation [default: 1]
      --socket-addr <SOCKET_ADDR>
          Socket address of LlamaEdge API Server instance [default: 0.0.0.0:8080]
  -h, --help
          Print help
  -V, --version
          Print version
```
