# Gateway Capability Index

This section adds a capability-first documentation view on top of the live gateway OpenAPI inventory.

It exists for discovery and operator navigation only. It does not change the public mirror contract:

- clients still switch only the gateway `base_url`
- official provider paths stay unchanged
- OpenAPI tag truth remains the source of truth for the live router

Use this index when you want to browse the gateway by workload family instead of by raw OpenAPI tag list.

## Capability Families

| Family | Shared default family first | Provider or alias directories |
|---|---|---|
| [Code](/api-reference/gateway-capabilities/code) | `code.openai` on `/v1/*` | `openai`, `claude`, `gemini` |
| [Images](/api-reference/gateway-capabilities/images) | `images.openai` on `/v1/images/*` | `openai`, `kling`, `aliyun`, `volcengine`, `nanobanana`, `midjourney` |
| [Video](/api-reference/gateway-capabilities/video) | `video.openai` on `/v1/videos*` | `openai`, `sora2`, `kling`, `aliyun`, `google-veo`, `minimax`, `vidu`, `volcengine` |
| [Music](/api-reference/gateway-capabilities/music) | `music.openai` on `/v1/music*` | `openai`, `google`, `minimax`, `suno` |

## How To Read This Section

- Shared default family first: if OpenAI already defines a public standard route for that workload, SDKWork keeps that route as the shared public contract.
- Provider directory next: when no shared OpenAI route exists, SDKWork mirrors the provider's official path directly.
- Alias directory: some names are useful as discovery labels even when the live router does not publish a separate OpenAPI tag for them.

Examples:

- `images/nanobanana` is documented as an alias page, but the live protocol family is `code.gemini` on `/v1beta/models/{model}:generateContent`.
- `video/sora2` is documented as an alias page, but the live protocol family is `video.openai` on `/v1/videos*`.
- `images/midjourney` is documented as an unpublished capability because Midjourney does not currently expose a mirrorable official API surface that fits the base-URL-only rule.

## Related Docs

- [Gateway API](/api-reference/gateway-api)
- [API Compatibility](/reference/api-compatibility)
- [Full Compatibility Matrix](/api/compatibility-matrix)
