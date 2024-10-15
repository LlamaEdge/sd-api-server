# Endpoints

sd-api-server provides two endpoints for image generation and editing. The following sections describe how to use these endpoints.

> [!NOTE]
> The project is still under active development. The existing features still need to be improved and more features will be added in the future.

## Create Image

```bash
POST http://localhost:{port}/v1/images/generations
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
- **control_image** (file, optional): Control image to use for image generation.
- **seed** (integer, optional): Seed for the random number generator. Negative value means to use random seed. Default is 42.
- **response_format** (string, optional): Format of the response. Possible values are `url` and `b64_json`. Default is `url`.

### Example

- Text-to-image generation:

  ```bash
  curl -X POST http://localhost:8080/v1/images/generations \
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
    "seed": 42
  }'
  ```

- Text-to-image generation with control net:

  ```bash
  curl --location 'http://localhost:10086/v1/images/generations' \
  --form 'control_image=@"/path/control_image.png"' \
  --form 'prompt="a person"' \
  --form 'cfg_scale="7.0"' \
  --form 'sample_method="euler_a"' \
  --form 'steps="20"' \
  --form 'size="512x512"' \
  --form 'control_strength="0.9"' \
  --form 'seed="42"'
  ```

## Edit Image

```bash
POST http://localhost:{port}/v1/images/edits
```

Creates an edited or extended image given an original image and a prompt.

### Request body

- **model** (string, optional): Name of the model to use for image generation. If not provided, the default model is used.
- **image** (file): Image file to edit.
- **prompt** (string): A text description of the desired image.
- **negative_prompt** (string, optional): A text description of what the image should not contain.
- **n** (integer, optional): Number of images to generate. Default is 1.
- **size** (integer, optional): Size of the generated image in pixel space. The format is `widthxheight`. Default is 512x512.
- **height** (integer, optional): Height of the generated image in pixel space. Default is 512. If `size` is provided, this field will be ignored.
- **width** (integer, optional): Width of the generated image in pixel space. Default is 512. If `size` is provided, this field will be ignored.
- **cfg_scale** (float, optional): Scale factor for the model's configuration. Default is 7.0.
- **sample_method** (string, optional): Sampling method to use. Possible values are `euler`, `euler_a`, `heun`, `dpm2`, `dpm++2s_a`, `dpm++2m`, `dpm++2mv2`, `ipndm`, `ipndm_v`, and `lcm`. Default is `euler_a`.
- **steps** (integer, optional): Number of sample steps to take. Default is 20.
- **control_strength** (float, optional): Control strength for the model. Default is 0.9.
- **control_image** (file, optional): Control image to use for image generation.
- **seed** (integer, optional): Seed for the random number generator. Negative value means to use random seed. Default is 42.
- **strength** (float, optional): Strength of the edit. Default is 0.75.
- **response_format** (string, optional): Format of the response. Possible values are `url` and `b64_json`. Default is `url`.

### Example

```bash
curl --location 'http://localhost:8080/v1/images/edits' \
--form 'image=@"/path/to/image.png"' \
--form 'prompt="modern disney style"' \
--form 'negative_prompt="no animals"' \
--form 'n="1"' \
--form 'height=512' \
--form 'width=512' \
--form 'cfg_scale=7.0' \
--form 'sample_method="euler_a"' \
--form 'steps="20"' \
--form 'control_strength="0.9"' \
--form 'seed="42"'
--form 'strength="0.75"' \
--form 'response_format="url"'
```
