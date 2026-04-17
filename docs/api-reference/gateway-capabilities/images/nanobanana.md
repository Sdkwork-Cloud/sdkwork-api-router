# images/nanobanana

## Capability Alias

- Alias status: documented alias only
- Live mirror family: `code.gemini`
- Public path family: `/v1beta/models/{model}:generateContent`

## Why This Page Exists

- Many operators search for Nano Banana under image tooling first.
- The live router must still preserve Google's official Gemini path so clients only change `base_url`.
- SDKWork therefore documents Nano Banana here as an image capability alias without inventing a separate `images.nanobanana` OpenAPI tag.

## Use This Live Route

- `POST /v1beta/models/{model}:generateContent`
- `POST /v1beta/models/{model}:streamGenerateContent?alt=sse`

## Related Docs

- [code/gemini](../code/gemini)
- [Gateway API](/api-reference/gateway-api)
