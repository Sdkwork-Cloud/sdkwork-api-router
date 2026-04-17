# 网关能力矩阵

这一页把 capability 文档压缩成一张总表。

它仍然只是文档层。真实路由真值依旧以当前网关实现、`/openapi.json` 和兼容性文档为准。

| 能力 | 文档目录 | 真实家族或 tag | 公开路径家族 | 状态 | 说明 |
|---|---|---|---|---|---|
| audio 默认 | `audio` | `audio.openai` | `/v1/audio/*` | active | 共享默认音频契约 |
| code openai | `code/openai` | `code.openai` | `/v1/*` | active | 共享 OpenAI 与 Codex 家族 |
| code claude | `code/claude` | `code.claude` | `/v1/messages*` | active | 官方 Claude 镜像家族 |
| code gemini | `code/gemini` | `code.gemini` | `/v1beta/models/{model}:*` | active | 官方 Gemini 镜像家族 |
| images openai | `images/openai` | `images.openai` | `/v1/images/*` | active | 共享默认图片契约 |
| images kling | `images/kling` | `images.kling` | `/api/v1/services/aigc/image-generation/generation` 与 `/api/v1/tasks/{task_id}` | active | 共享 DashScope 图片传输 |
| images aliyun | `images/aliyun` | `images.aliyun` | `/api/v1/services/aigc/image-generation/generation` 与 `/api/v1/tasks/{task_id}` | active | 共享 DashScope 图片传输 |
| images volcengine | `images/volcengine` | `images.volcengine` | `/api/v3/images/generations` | active | 火山引擎官方图片传输 |
| images.nanobanana | `images/nanobanana` | 别名到 `code.gemini` | `/v1beta/models/{model}:generateContent` | alias | 面向图片视角的别名页，不是单独 OpenAPI tag |
| images.midjourney | `images/midjourney` | 无 | 无 | unpublished | 当前没有满足 base-url-only 规则的官方 Midjourney API |
| video openai | `video/openai` | `video.openai` | `/v1/videos*` | active | 共享默认视频契约 |
| video.sora2 | `video/sora2` | 别名到 `video.openai` | `/v1/videos*` | alias | Sora 2 与 Sora 2 Pro 仍归在共享 OpenAI 视频传输 |
| video kling | `video/kling` | `video.kling` | `/api/v1/services/aigc/video-generation/video-synthesis` 与 `/api/v1/tasks/{task_id}` | active | 共享 DashScope 视频传输 |
| video aliyun | `video/aliyun` | `video.aliyun` | `/api/v1/services/aigc/video-generation/video-synthesis` 与 `/api/v1/tasks/{task_id}` | active | 共享 DashScope 视频传输 |
| video google-veo | `video/google-veo` | `video.google-veo` | `/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predictLongRunning` 与 `:fetchPredictOperation` | active | Google Vertex AI Veo 官方传输 |
| video minimax | `video/minimax` | `video.minimax` | `/v1/video_generation`、`/v1/query/video_generation`、`/v1/files/retrieve` | active | MiniMax 官方视频传输 |
| video vidu | `video/vidu` | `video.vidu` | `/ent/v2/*` | active | Vidu 官方传输 |
| video volcengine | `video/volcengine` | `video.volcengine` | `/api/v1/contents/generations/tasks*` | active | 火山引擎官方视频传输 |
| music openai | `music/openai` | `music.openai` | `/v1/music*` | active | 共享默认音乐契约 |
| music google | `music/google` | `music.google` | `/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predict` | active | Google 官方音乐传输 |
| music minimax | `music/minimax` | `music.minimax` | `/v1/music_generation` 与 `/v1/lyrics_generation` | active | MiniMax 官方音乐传输 |
| music suno | `music/suno` | `music.suno` | `/api/v1/generate*` 与 `/api/v1/lyrics*` | active | Suno 官方音乐传输 |

## 如何理解状态列

- `active`：已经发布的真实家族，具备实际公开路径或 OpenAPI tag
- `alias`：仅文档目录，映射到已发布的真实家族
- `unpublished`：已记录的能力缺口，但刻意不作为真实网关家族公开

## 相关文档

- [网关能力索引](/zh/api-reference/gateway-capabilities)
- [API 兼容矩阵](/zh/reference/api-compatibility)
- [完整兼容矩阵](/api/compatibility-matrix)
