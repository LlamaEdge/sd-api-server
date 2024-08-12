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

### Image Editing

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
