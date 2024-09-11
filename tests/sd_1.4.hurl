# test /v1/images/generations endpoint
POST http://localhost:8080/v1/images/generations
Accept: application/json
Content-Type: application/json
```json
{
    "model": "sd-v1.4",
    "prompt": "A cute baby sea otter"
}
```
HTTP 200
[Asserts]
jsonpath "$.data[0].prompt" == "A cute baby sea otter"
