# Gateway Capability Matrix

This page compresses the capability-first docs into one table.

It is still documentation-only. Live routing truth stays with the current gateway implementation, `/openapi.json`, and the compatibility docs.

| Capability | Docs directory | Live family or tag | Public path family | State | Notes |
|---|---|---|---|---|---|
| audio default | `audio` | `audio.openai` | `/v1/audio/*` | active | shared default audio contract |
| code openai | `code/openai` | `code.openai` | `/v1/*` | active | shared OpenAI and Codex family |
| code claude | `code/claude` | `code.claude` | `/v1/messages*` | active | official Claude mirror family |
| code gemini | `code/gemini` | `code.gemini` | `/v1beta/models/{model}:*` | active | official Gemini mirror family |
| images openai | `images/openai` | `images.openai` | `/v1/images/*` | active | shared default image contract |
| images kling | `images/kling` | `images.kling` | `/api/v1/services/aigc/image-generation/generation` and `/api/v1/tasks/{task_id}` | active | shared DashScope image transport |
| images aliyun | `images/aliyun` | `images.aliyun` | `/api/v1/services/aigc/image-generation/generation` and `/api/v1/tasks/{task_id}` | active | shared DashScope image transport |
| images volcengine | `images/volcengine` | `images.volcengine` | `/api/v3/images/generations` | active | official Volcengine Ark image transport |
| images.nanobanana | `images/nanobanana` | alias to `code.gemini` | `/v1beta/models/{model}:generateContent` | alias | documented as an image-first alias, not a separate OpenAPI tag |
| images.midjourney | `images/midjourney` | none | none | unpublished | no mirrorable official Midjourney API under the base-URL-only rule |
| video openai | `video/openai` | `video.openai` | `/v1/videos*` | active | shared default video contract |
| video.sora2 | `video/sora2` | alias to `video.openai` | `/v1/videos*` | alias | Sora 2 and Sora 2 Pro stay on the shared OpenAI video transport |
| video kling | `video/kling` | `video.kling` | `/api/v1/services/aigc/video-generation/video-synthesis` and `/api/v1/tasks/{task_id}` | active | shared DashScope video transport |
| video aliyun | `video/aliyun` | `video.aliyun` | `/api/v1/services/aigc/video-generation/video-synthesis` and `/api/v1/tasks/{task_id}` | active | shared DashScope video transport |
| video google-veo | `video/google-veo` | `video.google-veo` | `/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predictLongRunning` and `:fetchPredictOperation` | active | official Vertex AI Veo transport |
| video minimax | `video/minimax` | `video.minimax` | `/v1/video_generation`, `/v1/query/video_generation`, `/v1/files/retrieve` | active | official MiniMax video transport |
| video vidu | `video/vidu` | `video.vidu` | `/ent/v2/*` | active | official Vidu transport |
| video volcengine | `video/volcengine` | `video.volcengine` | `/api/v1/contents/generations/tasks*` | active | official Volcengine video transport |
| music openai | `music/openai` | `music.openai` | `/v1/music*` | active | shared default music contract |
| music google | `music/google` | `music.google` | `/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predict` | active | official Google music transport |
| music minimax | `music/minimax` | `music.minimax` | `/v1/music_generation` and `/v1/lyrics_generation` | active | official MiniMax music transport |
| music suno | `music/suno` | `music.suno` | `/api/v1/generate*` and `/api/v1/lyrics*` | active | official Suno music transport |

## Reading The State Column

- `active`: published live family with a real OpenAPI tag or documented public path family
- `alias`: documentation directory only, mapped to an already-published live family
- `unpublished`: documented gap, intentionally not exposed as a live gateway family

## Related Docs

- [Gateway Capability Index](/api-reference/gateway-capabilities)
- [API Compatibility](/reference/api-compatibility)
- [Full Compatibility Matrix](/api/compatibility-matrix)
