# Gateway Image Families

The image capability index starts with the shared default contract and then lists provider or alias directories.

## Shared Default Image Family

| Family | Live OpenAPI tag | Public path family | Notes |
|---|---|---|---|
| default shared image family | `images.openai` | `/v1/images/*` | generations, edits, and variations stay on the official OpenAI image contract |

## Image Directories

| Directory | Live family | Public path family | Status | Notes |
|---|---|---|---|---|
| [openai](./images/openai) | `images.openai` | `/v1/images/*` | active | shared default image contract |
| [kling](./images/kling) | `images.kling` | `/api/v1/services/aigc/image-generation/generation` and `/api/v1/tasks/{task_id}` | active | official shared DashScope image transport |
| [aliyun](./images/aliyun) | `images.aliyun` | `/api/v1/services/aigc/image-generation/generation` and `/api/v1/tasks/{task_id}` | active | official shared DashScope image transport |
| [volcengine](./images/volcengine) | `images.volcengine` | `/api/v3/images/generations` | active | official Volcengine Ark image transport |
| [nanobanana](./images/nanobanana) | alias to `code.gemini` | `/v1beta/models/{model}:generateContent` | documented alias | image-first discovery alias for Gemini-native image generation |
| [midjourney](./images/midjourney) | unpublished | none | unpublished | no mirrorable official API surface under the base-URL-only rule |
