# 网关 Video 家族

这一页先列共享默认视频契约，再列 provider 与别名目录。

## 共享默认视频家族

| 家族 | 真实 OpenAPI tag | 公开路径家族 | 说明 |
|---|---|---|---|
| 默认共享视频家族 | `video.openai` | `/v1/videos*` | OpenAI 视频路由继续复用官方协议，并覆盖 Sora 2 与 Sora 2 Pro |

## 共享默认 API 清单

- `GET /v1/videos`
- `POST /v1/videos`
- `GET /v1/videos/{video_id}`
- `GET /v1/videos/{video_id}/content`
- `POST /v1/videos/{video_id}/remix`
- `POST /v1/videos/{video_id}/edits`

## Video 目录

| 目录 | 真实家族 | 公开路径家族 | 状态 | 说明 |
|---|---|---|---|---|
| [openai](./video/openai) | `video.openai` | `/v1/videos*` | active | 默认共享视频契约 |
| [sora2](./video/sora2) | 别名到 `video.openai` | `/v1/videos*` | 文档别名 | 面向能力视角的 Sora 2 与 Sora 2 Pro 索引页 |
| [kling](./video/kling) | `video.kling` | `/api/v1/services/aigc/video-generation/video-synthesis` 与 `/api/v1/tasks/{task_id}` | active | 共享 DashScope 官方视频传输 |
| [aliyun](./video/aliyun) | `video.aliyun` | `/api/v1/services/aigc/video-generation/video-synthesis` 与 `/api/v1/tasks/{task_id}` | active | 共享 DashScope 官方视频传输 |
| [google-veo](./video/google-veo) | `video.google-veo` | `/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predictLongRunning` 与 `:fetchPredictOperation` | active | Google Vertex AI Veo 官方传输 |
| [minimax](./video/minimax) | `video.minimax` | `/v1/video_generation`、`/v1/query/video_generation`、`/v1/files/retrieve` | active | MiniMax 官方视频传输 |
| [vidu](./video/vidu) | `video.vidu` | `/ent/v2/*` | active | Vidu 官方视频传输 |
| [volcengine](./video/volcengine) | `video.volcengine` | `/api/v1/contents/generations/tasks*` | active | 火山引擎官方异步视频传输 |
