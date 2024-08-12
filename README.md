# Stable-Diffusion-API-Server

> [!NOTE]
> WasmEdge-0.14.1-rc.1 with `wasmedge_stablediffusion` and `wasmedge_logging` plugins is required to run the server.

## Build

```bash
cargo build --target wasm32-wasip1 --release
```

`sd-api-server.wasm` will be generated in `target/wasm32-wasip1/release/`.

## Run

- Download the stable diffusion model

  ```bash
  curl -LO https://huggingface.co/second-state/stable-diffusion-v-1-4-GGUF/resolve/main/stable-diffusion-v1-4-Q8_0.gguf
  ```

- Start the server

  ```bash
  wasmedge --dir .:. sd-api-server.wasm --model-name sd-v1.4 --gguf stable-diffusion-v1-4-Q8_0.gguf
  ```

  > `sd-api-server` will use `8080` port by default. You can change the port by adding `--socket-addr <ip-address>:<port>`.

## Usage

### Image Generation

- Send a request for image generation

  ```bash
  curl -X POST 'http://localhost:8080/v1/images/generations' \
    --header 'Content-Type: application/json' \
    --data '{
        "model": "sd-v1.4",
        "prompt": "A cute baby sea otter"
    }'
  ```

- Preview the generated image with [Base64.Guru online tool](https://base64.guru/converter/decode/image)

  The following snapshot is an example of the generated image shown in the online tool.


<div align=center>
<img src="image/otter.png" alt="A cute baby sea otter" width="60%" />
</div>

### Image Editing

- Send a request for image editing

  ```bash
  curl --location 'http://localhost:10086/v1/images/edits' \
    --form 'image=@"otter.png"' \
    --form 'prompt="A cute baby sea otter with blue eyes"'
  ```

- Preview the edited image with [Base64.Guru online tool](https://base64.guru/converter/decode/image)

  The following snapshot is an example of the edited image shown in the online tool.

<div align=center>
<img src="image/otter_blue_eyes.png" alt="A cute baby sea otter with blue eyes" width="60%" />
</div>
