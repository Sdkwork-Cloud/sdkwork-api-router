# Gateway Video Families

The video capability index starts with the shared default contract and then lists provider or alias directories.

## Shared Default Video Family

| Family | Live OpenAPI tag | Public path family | Notes |
|---|---|---|---|
| default shared video family | `video.openai` | `/v1/videos*` | OpenAI video routes, including Sora 2 and Sora 2 Pro, stay on the shared official transport |

## Video Directories

| Directory | Live family | Public path family | Status | Notes |
|---|---|---|---|---|
| [openai](./video/openai) | `video.openai` | `/v1/videos*` | active | shared default video contract |
| [sora2](./video/sora2) | alias to `video.openai` | `/v1/videos*` | documented alias | capability-first alias for Sora 2 and Sora 2 Pro |
| [kling](./video/kling) | `video.kling` | `/api/v1/services/aigc/video-generation/video-synthesis` and `/api/v1/tasks/{task_id}` | active | official shared DashScope video transport |
| [aliyun](./video/aliyun) | `video.aliyun` | `/api/v1/services/aigc/video-generation/video-synthesis` and `/api/v1/tasks/{task_id}` | active | official shared DashScope video transport |
| [google-veo](./video/google-veo) | `video.google-veo` | `/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predictLongRunning` and `:fetchPredictOperation` | active | official Vertex AI Veo transport |
| [minimax](./video/minimax) | `video.minimax` | `/v1/video_generation`, `/v1/query/video_generation`, `/v1/files/retrieve` | active | official MiniMax video transport |
| [vidu](./video/vidu) | `video.vidu` | `/ent/v2/*` | active | official Vidu video transport |
| [volcengine](./video/volcengine) | `video.volcengine` | `/api/v1/contents/generations/tasks*` | active | official Volcengine async video transport |
