# ControlNet

This example demonstrates how to use the `sd-api-server` to generate images using the stable diffusion model with the openpose control model.

## Setup

- Install WasmEdge v0.14.1

  ```bash
  curl -sSf https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install_v2.sh | bash -s -- -v 0.14.1
  ```

  If the installation is successful, WasmEdge will be installed in `$HOME/.wasmedge`.

- Deploy `wasmedge_stablediffusion` plugin

  > [!NOTE]
  > For the purpose of demonstration, we will use the stable diffusion plugin for Mac Apple Silicon. You can find the plugin for other platforms [Releases/0.14.1](https://github.com/WasmEdge/WasmEdge/releases/tag/0.14.1)

  ```bash
  # Download stable diffusion plugin for Mac Apple Silicon
  curl -LO https://github.com/WasmEdge/WasmEdge/releases/download/0.14.1/WasmEdge-plugin-wasmedge_stablediffusion-metal-0.14.1-darwin_arm64.tar.gz

  # Unzip the plugin to $HOME/.wasmedge/plugin
  tar -xzf WasmEdge-plugin-wasmedge_stablediffusion-metal-0.14.1-darwin_arm64.tar.gz -C $HOME/.wasmedge/plugin

  rm $HOME/.wasmedge/plugin/libwasmedgePluginWasiNN.dylib
  ```

## Run sd-api-server

- Download stable-diffusion model

  ```bash
  curl -LO https://huggingface.co/second-state/stable-diffusion-v-1-4-GGUF/resolve/main/stable-diffusion-v1-4-Q4_0.gguf
  ```

- Download openpose control model

  ```bash
  curl -LO https://huggingface.co/webui/ControlNet-modules-safetensors/resolve/main/control_openpose-fp16.safetensors
  ```

- Run sd-api-server

  ```bash
  wasmedge --dir .:. \
    sd-api-server.wasm \
    --model-name sd-v1.4 \
    --model stable-diffusion-v1-4-Q8_0.gguf \
    --control-net control_openpose-fp16.safetensors \
    --task text2image
  ```

## Usage

- Download the control image

  ```bash
  curl -LO https://raw.githubusercontent.com/LlamaEdge/sd-api-server/refs/heads/main/image/control_2.png
  ```

- Send a POST request to the server to generate an image

  ```bash
  curl --location 'http://localhost:10086/v1/images/generations' \
  --form 'control_image=@"control_2.png"' \
  --form 'prompt="1girl, high quality, masterpiece, anime style, white dress, sun hat"' \
  --form 'cfg_scale="7"' \
  --form 'sample_method="euler_a"' \
  --form 'steps="20"' \
  --form 'seed="-1"'
  ```

  If the request is handled successfully, the server will respond with a JSON object containing the URL of the generated image.

  ```json
  {
    "created": 1728910462,
    "data": [
        {
            "url": "/archives/file_cb247eeb-2507-4a95-9a78-2f4c02a6300e/output.png",
            "prompt": "1girl, high quality, masterpiece, anime style, white dress, sun hat"
        }
    ]
  }
  ```

  The generated image looks like this:

  <div align=center>
  <img src="../image/girl_controlnet.png" alt="1girl, high quality, masterpiece, anime style, white dress, sun hat" width="60%" />
  </div>
