# Endpoints

sd-api-server provides two endpoints for image generation and editing. The following sections describe how to use these endpoints.

## Create Image

```bash
POST http://localhost:{port}/v1/image/generations
```

Creates an image given a prompt.

### Request body

- **model** (string, optional): Name of the model to use for image generation. If not provided, the default model is used.
- **prompt** (string): A text description of the desired image.
- **negative_prompt** (string, optional): A text description of what the image should not contain.
- **n** (integer, optional): Number of images to generate. Default is 1.
- **cfg_scale** (float, optional): Scale factor for the model's configuration. Default is 7.0.
- **sample_method** (string, optional): Sampling method to use. Possible values are `euler`, `euler_a`, `heun`, `dpm2`, `dpm++2s_a`, `dpm++2m`, `dpm++2mv2`, `ipndm`, `ipndm_v`, and `lcm`. Default is `euler_a`.
- **steps** (integer, optional): Number of sample steps to take. Default is 20.
- **size** (integer, optional): Size of the generated image in pixel space. The format is `widthxheight`. Default is 512x512.
- **height** (integer, optional): Height of the generated image in pixel space. Default is 512. If `size` is provided, this field will be ignored.
- **width** (integer, optional): Width of the generated image in pixel space. Default is 512. If `size` is provided, this field will be ignored.
- **control_strength** (float, optional): Control strength for the model. Default is 0.9.
- **seed** (integer, optional): Seed for the random number generator. Negative value means to use random seed. Default is 42.

**Example**

```json
curl -X POST 'http://localhost:8080/v1/images/generations' \
--header 'Content-Type: application/json' \
--data '{
  "model": "sd",
  "prompt": "A painting of a beautiful sunset over a calm lake.",
  "negative_prompt": "No people or animals in the image.",
  "n": 1,
  "cfg_scale": 7.0,
  "sample_method": "euler_a",
  "steps": 20,
  "size": "512x512",
  "control_strength": 0.9,
  "seed": 42
}'
```
