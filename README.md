# Stable-Diffusion-API-Server

This project is a RESTful API server that provides image generation and editing services based on Stable Diffusion models. The APIs are compatible with OpenAI APIs of [image generation and editing](https://platform.openai.com/docs/api-reference/images).

> [!NOTE]
> The project is still under active development. The existing features still need to be improved and more features will be added in the future.

## Quick Start

### Setup

- Install WasmEdge v0.14.1

  ```bash
  curl -sSf https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install_v2.sh | bash -s -- -v 0.14.1
  ```

- Deply `wasmedge_stablediffusion` plugin

  > For the purpose of demonstration, we will use the stable diffusion plugin for Mac Apple Silicon. You can find the plugin for other platforms [Releases/0.14.1](https://github.com/WasmEdge/WasmEdge/releases/tag/0.14.1)

  ```bash
  # Download stable diffusion plugin for Mac Apple Silicon
  curl -LO https://github.com/WasmEdge/WasmEdge/releases/download/0.14.1/WasmEdge-plugin-wasmedge_stablediffusion-0.14.1-darwin_arm64.tar.gz

  # Unzip the plugin to $HOME/.wasmedge/plugin
  tar -xzf WasmEdge-plugin-wasmedge_stablediffusion-0.14.1-darwin_arm64.tar.gz -C $HOME/.wasmedge/plugin

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
  - [second-state/stable-diffusion-3-medium-GGUF](https://huggingface.co/second-state/stable-diffusion-3-medium-GGUF)

- Download sd-api-server.wasm

  ```bash
  curl -LO https://github.com/LlamaEdge/sd-api-server/releases/latest/download/sd-api-server.wasm
  ```

- Start the server

  ```bash
  wasmedge --dir .:. sd-api-server.wasm --model-name sd-v1.4 --model stable-diffusion-v1-4-Q8_0.gguf
  ```

  > [!TIP]
  > `sd-api-server` will use `8080` port by default. You can change the port by adding `--port <port>`.

  - Reduce the memory usage

    In the default setting, the server support two tasks:  `text2image` for image generations and `image2image` for image edits. If you want to run one of them, you can specify the task type by adding `--task <task-type>`. For example, if you only want to run image generations, then just start the server with the following command:

    ```bash
    wasmedge --dir .:. sd-api-server.wasm --model-name sd-v1.4 --model stable-diffusion-v1-4-Q8_0.gguf --task text2image
    ```

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
  curl --location 'http://localhost:8080/v1/images/edits' \
    --form 'image=@"otter.png"' \
    --form 'prompt="A cute baby sea otter with blue eyes"'
  ```

  If the request is handled successfully, the server will return a JSON response like the below. To preview or download the generated image, copy and paste the URL to your browser.

  ```json
  {
    "created": 1723432689,
    "data": [
        {
            "url": "http://localhost:8080/v1/files/download/file_554e4d53-6072-4988-83e6-fe684655a734"
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

- For **Linux users**

  ```bash
  cargo build --release
  ```

- For **macOS users**

  - Download the `wasi-sdk` from the [official website](https://github.com/WebAssembly/wasi-sdk/releases) and unzip it to the directory you want.

  - Build the project

    ```bash
    export WASI_SDK_PATH=/path/to/wasi-sdk
    export CC="${WASI_SDK_PATH}/bin/clang --sysroot=${WASI_SDK_PATH}/share/wasi-sysroot"
    cargo clean
    cargo update
    cargo build --release
    ```

If the build process is successful, `sd-api-server.wasm` will be generated in `target/wasm32-wasip1/release/`.

### CLI Options

```bash
$ wasmedge target/wasm32-wasip1/release/sd-api-server.wasm -h

LlamaEdge-Stable-Diffusion API Server

Usage: sd-api-server.wasm [OPTIONS] --model-name <MODEL_NAME> <--model <MODEL>|--diffusion-model <DIFFUSION_MODEL>>

Options:
  -m, --model-name <MODEL_NAME>
          Sets the model name
      --model <MODEL>
          Path to full model [default: ]
      --diffusion-model <DIFFUSION_MODEL>
          Path to the standalone diffusion model file [default: ]
      --vae <VAE>
          Path to vae [default: ]
      --clip-l <CLIP_L>
          Path to the clip-l text encoder [default: ]
      --t5xxl <T5XXL>
          Path to the the t5xxl text encoder [default: ]
      --lora-model-dir <LORA_MODEL_DIR>
          Path to the lora model directory
      --control-net <CONTROL_NET>
          Path to control net model
      --control-net-cpu
          Keep controlnet on cpu (for low vram)
      --threads <THREADS>
          Number of threads to use during computation. Default is -1, which means to use all available threads [default: -1]
      --clip-on-cpu
          Keep clip on cpu (for low vram)
      --vae-on-cpu
          Keep vae on cpu (for low vram)
      --task <TASK>
          Task type [default: full] [possible values: text2image, image2image, full]
      --socket-addr <SOCKET_ADDR>
          Socket address of LlamaEdge API Server instance. For example, `0.0.0.0:8080`
      --port <PORT>
          Port number [default: 8080]
      --download-url-prefix <DOWNLOAD_URL_PREFIX>
          Download URL prefix, format: `http(s)://{IPv4_address}:{port}` or `http(s)://{domain}:{port}`
  -h, --help
          Print help (see more with '--help')
  -V, --version
          Print version
```
