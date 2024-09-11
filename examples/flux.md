# FLUX.1

This example demonstrates how to use the `sd-api-server` to generate images using the FLUX.1-Schnell model.

## Setup

- Install WasmEdge v0.14.1

  ```bash
  curl -sSf https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install_v2.sh | bash -s -- -v 0.14.1-rc.4
  ```

  If the installation is successful, WasmEdge will be installed in `$HOME/.wasmedge`.

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

## Run sd-api-server

- Download FLUX.1.Schnell model

  ```bash
  # download the model
  curl -LO https://huggingface.co/second-state/FLUX.1-schnell-GGUF/resolve/main/flux1-schnell-Q8_0.gguf

  # download vae file
  curl -LO https://huggingface.co/second-state/FLUX.1-schnell-GGUF/resolve/main/ae.safetensors

  # download clip_l encoder
  curl -LO https://huggingface.co/second-state/FLUX.1-schnell-GGUF/resolve/main/clip_l.safetensors

  # download t5xxl encoder
  curl -LO https://huggingface.co/second-state/FLUX.1-schnell-GGUF/resolve/main/t5xxl_fp16.safetensors
  ```

- Start the server

  ```bash
  wasmedge --dir .:. sd-api-server.wasm \
    --model-name flux1-schnell \
    --diffusion-model flux1-schnell-Q8_0.gguf \
    --vae ae.safetensors \
    --clip-l clip_l.safetensors \
    --t5xxl t5xxl_fp16.safetensors
  ```

  > `sd-api-server` will use `8080` port by default. You can change the port by adding `--socket-addr <ip-address>:<port>`.

## Usage

### Image Generation

- Send a request for image generation

  ```bash
  curl -X POST 'http://localhost:8080/v1/images/generations' \
    --header 'Content-Type: application/json' \
    --data '{
        "model": "flux1-schnell",
        "prompt": "a lovely cat holding a sign says '\''flux.cpp'\''",
        "cfg_scale": 1.0,
        "sample_method": "euler",
        "steps": 4
    }'
  ```

  If the request is handled successfully, the server will return a JSON response like the following:

  ```json
  {
    "created": 1725984300,
    "data": [
        {
            "url": "/archives/file_07a68d86-85fd-442c-a336-5e0a1212f4f0/output.png",
            "prompt": "a lovely cat holding a sign says 'flux.cpp'"
        }
    ]
  }
  ```

  The path shown in the "url" field is the relative path to the generated image. For example, assume that the `sd-api-server.wasm` file is located in the directory of `/path/to/sd-api-server.wasm`, the generated image can be accessed in `/path/to/archives/file_07a68d86-85fd-442c-a336-5e0a1212f4f0/output.png`.
