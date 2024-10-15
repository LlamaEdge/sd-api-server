# FLUX.1-dev with LoRA Model

This example demonstrates how to use the `sd-api-server` to generate images using the FLUX.1-Dev model.

> [!TIP]
> The following commands are also applicable to the `flux.1-schell` model. [second-state/FLUX.1-schnell-GGUF](https://huggingface.co/second-state/FLUX.1-schnell-GGUF) provides the `flux.1-schell` gguf model and other relevant files.

## Setup

- Install WasmEdge v0.14.1

  ```bash
  curl -sSf https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install_v2.sh | bash -s -- -v 0.14.1
  ```

  If the installation is successful, WasmEdge will be installed in `$HOME/.wasmedge`.

- Deply `wasmedge_stablediffusion` plugin

  > [!NOTE]
  > For the purpose of demonstration, we will use the stable diffusion plugin for Mac Apple Silicon. You can find the plugin for other platforms [Releases/0.14.1](https://github.com/WasmEdge/WasmEdge/releases/tag/0.14.1)

  ```bash
  # Download stable diffusion plugin for Mac Apple Silicon
  curl -LO https://github.com/WasmEdge/WasmEdge/releases/download/0.14.1/WasmEdge-plugin-wasmedge_stablediffusion-0.14.1-darwin_arm64.tar.gz

  # Unzip the plugin to $HOME/.wasmedge/plugin
  tar -xzf WasmEdge-plugin-wasmedge_stablediffusion-0.14.1-darwin_arm64.tar.gz -C $HOME/.wasmedge/plugin

  rm $HOME/.wasmedge/plugin/libwasmedgePluginWasiNN.dylib
  ```

## Run sd-api-server

- Download FLUX.1-dev model

  ```bash
  # download the model
  curl -LO https://huggingface.co/second-state/FLUX.1-dev-GGUF/resolve/main/flux1-dev-Q4_0.gguf

  # download vae file
  curl -LO https://huggingface.co/second-state/FLUX.1-dev-GGUF/resolve/main/ae.safetensors

  # download clip_l encoder
  curl -LO https://huggingface.co/second-state/FLUX.1-dev-GGUF/resolve/main/clip_l.safetensors

  # download t5xxl encoder
  curl -LO https://huggingface.co/second-state/FLUX.1-dev-GGUF/resolve/main/t5xxl-Q8_0.gguf

  # download lora model
  mkdir lora-models
  curl -L https://huggingface.co/XLabs-AI/flux-lora-collection/resolve/main/realism_lora_comfy_converted.safetensors -o lora-models/realism_lora_comfy_converted.safetensors
  ```

- Download `sd-api-server.wasm`

  ```bash
  curl -LO https://github.com/LlamaEdge/sd-api-server/releases/latest/download/sd-api-server.wasm
  ```

- Start server with LoRA model

  Assume that the LoRA model is located in the `lora-models` sub-directory of the currect directory.

  ```bash
  wasmedge --dir .:. \
    --dir lora-models:lora-models \
    sd-api-server.wasm \
    --model-name flux1-dev \
    --diffusion-model flux1-dev-Q4_0.gguf \
    --vae ae.safetensors \
    --clip-l clip_l.safetensors \
    --t5xxl t5xxl-Q8_0.gguf \
    --lora-model-dir lora-models
  ```

  > [!TIP]
  > `sd-api-server` will use `8080` port by default. You can change the port by adding `--port <port>`.

  - Reduce the memory usage

    In the default setting, the server will create `text-to-image` and `image-to-image` contexts for the model. `text-to-image` context is responsible for image generation tasks, while `image-to-image` context for image edits. If you only need one of them, you can specify the context type by adding `--context-type <context-type>`. For example, if you only need the `text-to-image` context, you can start the server with the following command:

    ```bash
    wasmedge --dir .:. \
      --dir lora-models:lora-models \
      sd-api-server.wasm \
      --model-name flux1-dev \
      --diffusion-model flux1-dev-Q4_0.gguf \
      --vae ae.safetensors \
      --clip-l clip_l.safetensors \
      --t5xxl t5xxl-Q8_0.gguf \
      --lora-model-dir lora-models \
      --task text2image
    ```

## Usage

### Image Generation

- Send a request for image generation

  ```bash
  curl -X POST 'http://localhost:8080/v1/images/generations' \
    --header 'Content-Type: application/json' \
    --data '{
        "model": "flux1-dev",
        "prompt": "4 marbles on a table each containing a a different element ,1 with soil and plants, 1 with sea waves, 1 with a raging fire, 1 with a tornado , on a old piece of wood, hyper realistic, 4k, f1.8, boketh, depth of field, refraction, reflections on wood, photograph, glowing, glowing particles exiting the marble<lora:realism_lora_comfy_converted:1>",
        "cfg_scale": 1.0,
        "sample_method": "euler",
        "steps": 20
    }'
  ```

  > [!NOTE]
  > The time taken to generate an image depends on the performance of the hardware.

  If the request is handled successfully, the server will return a JSON response like the following:

  ```json
  {
    "created": 1725984300,
    "data": [
        {
            "url": "/archives/file_07a68d86-85fd-442c-a336-5e0a1212f4f0/output.png",
            "prompt": "4 marbles on a table each containing a a different element ,1 with soil and plants, 1 with sea waves, 1 with a raging fire, 1 with a tornado , on a old piece of wood, hyper realistic, 4k, f1.8, boketh, depth of field, refraction, reflections on wood, photograph, glowing, glowing particles exiting the marble"
        }
    ]
  }
  ```

  The path shown in the "url" field is the relative path to the generated image. For example, assume that the `sd-api-server.wasm` file is located in the directory of `/path/to/sd-api-server.wasm`, the generated image can be accessed in `/path/to/archives/file_07a68d86-85fd-442c-a336-5e0a1212f4f0/output.png`.

- The generated image

  The following shows two images generated on `Apple Silicon M1 Pro` with `32GB` memory : (left) generated with Comfy realism LoRA model, (right) generated without LoRA model.

  <div align=center>
  <img src="../image/balls_lora.png" alt="balls with lora" width="50%" /><img src="../image/balls.png" alt="balls" width="50%" />
  </div>
