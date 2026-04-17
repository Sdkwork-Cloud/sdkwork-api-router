# video/sora2

## Capability Alias

- Alias status: documented alias only
- Live mirror family: `video.openai`
- Public path family: `/v1/videos*`

## Why This Page Exists

- Many readers search for Sora 2 as a named capability rather than as part of the shared OpenAI video family.
- The live router still keeps OpenAI's official transport and does not publish a separate `video.sora` or `video.sora2` tag.

## Use This Live Route Family

- `GET /v1/videos`
- `POST /v1/videos`
- `GET /v1/videos/{video_id}`
- `GET /v1/videos/{video_id}/content`

## Related Docs

- [video/openai](./openai)
- [Gateway API](/api-reference/gateway-api)
