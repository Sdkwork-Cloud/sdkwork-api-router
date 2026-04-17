# code/gemini

## Live Mirror Family

- OpenAPI tag: `code.gemini`
- Public path family: `/v1beta/models/{model}:*`
- Contract rule: keep Google's official Gemini transport

## Primary Routes

- `POST /v1beta/models/{model}:generateContent`
- `POST /v1beta/models/{model}:streamGenerateContent?alt=sse`
- `POST /v1beta/models/{model}:countTokens`

## Notes

- Image-capable Gemini models such as Nano Banana stay in this family.
- The docs also expose [images/nanobanana](../images/nanobanana) as a capability alias so image-first readers can still find the right live protocol quickly.
