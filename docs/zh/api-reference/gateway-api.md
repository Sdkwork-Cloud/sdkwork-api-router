# 网关 API

网关服务以镜像协议方式暴露公开 API。官方客户端路径保持不变，因此现有 SDK 和 CLI 理论上只需要切换 `base_url`。

## 基础地址与鉴权

- 默认本地基地址：`http://127.0.0.1:8080`
- 主要鉴权方式：`Authorization: Bearer skw_live_...`
- 健康检查：`GET /health`
- Metrics：`GET /metrics`
- OpenAPI JSON：`GET /openapi.json`
- API 清单页面：`GET /docs`

最小首个请求：

```bash
curl http://127.0.0.1:8080/v1/models \
  -H "Authorization: Bearer skw_live_your_key_here"
```

在独立服务模式下，网关是一个依赖 admin store 的有状态实现。无状态 gateway runtime 仍以库和运行时形态存在，其覆盖范围通过兼容矩阵文档继续说明。

OpenAPI 直接从当前 `axum` 路由实现生成，因此 JSON 文档与浏览器中的 API 清单会始终跟随真实公开路由面。

如果你希望按能力家族浏览，而不是直接从原始 OpenAPI 清单阅读，请先看 [网关能力索引](/zh/api-reference/gateway-capabilities)。这一层仅用于文档导航，不会改变真实镜像路径或 OpenAPI tag 真值。

## 镜像协议家族

- `code.openai`：OpenAI 与 Codex 的官方 `/v1/*` 镜像协议面
- `code.claude`：Claude 的官方 `/v1/messages*` 镜像协议面
- `code.gemini`：Gemini 的官方 `/v1beta/models/{model}:*` 镜像协议面，包含 Nano Banana 这类可出图 Gemini 模型
- `images.openai`：OpenAI 图片的官方 `/v1/images/*` 镜像协议面
- `images.kling`：可灵图片在共享 DashScope 官方 `/api/v1/services/aigc/image-generation/*` 与 `/api/v1/tasks/{task_id}` 上的镜像协议面
- `images.aliyun`：阿里云图片在共享 DashScope 官方 `/api/v1/services/aigc/image-generation/*` 与 `/api/v1/tasks/{task_id}` 上的镜像协议面
- `images.volcengine`：火山引擎图片在官方 `/api/v3/images/generations` 上的镜像协议面
- `audio.openai`：共享音频的官方 `/v1/audio/*` 镜像协议面
- `video.openai`：共享视频的官方 `/v1/videos*` 镜像协议面，包含 Sora 2 与 Sora 2 Pro
- `video.kling`：可灵视频在共享 DashScope 官方 `/api/v1/services/aigc/video-generation/*` 与 `/api/v1/tasks/{task_id}` 上的镜像协议面
- `video.aliyun`：阿里云视频在共享 DashScope 官方 `/api/v1/services/aigc/video-generation/*` 与 `/api/v1/tasks/{task_id}` 上的镜像协议面
- `video.google-veo`：Google Veo 在 Vertex AI 官方 `/v1/projects/*/locations/*/publishers/google/models/*` 上的镜像协议面，包含通过官方模型路径选择的 Veo 3 类模型
- `video.minimax`：MiniMax 的官方 `/v1/video_generation`、`/v1/query/video_generation` 与 `/v1/files/retrieve` 视频镜像协议面
- `video.vidu`：Vidu 的官方 `/ent/v2/*` 视频镜像协议面
- `video.volcengine`：火山引擎官方 `/api/v1/contents/generations/tasks` 与 `/api/v1/contents/generations/tasks/{id}` 视频镜像协议面
- `music.openai`：共享音乐的官方 `/v1/music*` 镜像协议面
- `music.google`：Google 音乐在 Vertex AI 官方 `/v1/projects/*/locations/*/publishers/google/models/{model}:predict` 上的镜像协议面
- `music.minimax`：MiniMax 的官方 `/v1/music_generation` 与 `/v1/lyrics_generation` 音乐镜像协议面
- `music.suno`：Suno 的官方 `/api/v1/*` 音乐镜像协议面
- 公开契约不会额外发明 `/code/*`、`/claude/*`、`/gemini/*` 这类 wrapper 前缀

## OpenAPI 分组

下表直接使用 `/openapi.json` 中暴露的精确 OpenAPI tag 名称。排序遵循“先共享默认家族，再列 provider-specific 镜像家族”的规则，同时保持官方 provider 路径不变。

| OpenAPI Tag | 路由 | 说明 |
|---|---|---|
| `system.sdkwork` | `GET /health` | OpenAPI 中公开的系统健康检查路由；`GET /metrics` 仍作为运维端点存在，不并入公开 OpenAPI 清单 |
| `code.openai` | `GET /models`、`GET /models/{model_id}`、`DELETE /models/{model_id}`、`GET/POST /chat/completions`、`GET/POST/DELETE /chat/completions/{completion_id}`、`GET /chat/completions/{completion_id}/messages`、`POST /completions`、`POST /responses`、`POST /responses/input_tokens`、`POST /responses/compact`、`GET/DELETE /responses/{response_id}`、`GET /responses/{response_id}/input_items`、`POST /responses/{response_id}/cancel`、`POST /embeddings`、`POST /moderations` | 共享默认的 OpenAI 与 Codex 镜像家族，对应官方 `/v1/*` 契约 |
| `code.claude` | `POST /v1/messages`、`POST /v1/messages/count_tokens` | Claude 官方镜像家族，面向 Claude Code 与其他 Anthropic 客户端 |
| `code.gemini` | `POST /v1beta/models/{model}:generateContent`、`POST /v1beta/models/{model}:streamGenerateContent?alt=sse`、`POST /v1beta/models/{model}:countTokens` | Gemini 官方镜像家族，包含 Nano Banana 这类可出图 Gemini 模型 |
| `images.openai` | `POST /images/generations`、`POST /images/edits`、`POST /images/variations` | 共享默认图片镜像家族，对应官方 OpenAI `/v1/images/*` 契约 |
| `images.kling` | `POST /api/v1/services/aigc/image-generation/generation`、`GET /api/v1/tasks/{task_id}` | 面向可灵兼容客户端的 provider-specific 图片镜像家族，复用共享 DashScope 官方异步图片传输 |
| `images.aliyun` | `POST /api/v1/services/aigc/image-generation/generation`、`GET /api/v1/tasks/{task_id}` | 面向阿里云兼容客户端的 provider-specific 图片镜像家族，复用共享 DashScope 官方异步图片传输 |
| `images.volcengine` | `POST /api/v3/images/generations` | 火山引擎 provider-specific 图片镜像家族，直接复用官方图片生成路径 |
| `audio.openai` | `POST /audio/transcriptions`、`POST /audio/translations`、`POST /audio/speech`、`GET /audio/voices`、`POST /audio/voice_consents` | 共享默认音频镜像家族，对应官方 `/v1/audio/*` 契约 |
| `files` | `GET/POST /files`、`GET/DELETE /files/{file_id}`、`GET /files/{file_id}/content` | 文件元数据与二进制内容获取 |
| `uploads` | `POST /uploads`、`POST /uploads/{upload_id}/parts`、`POST /uploads/{upload_id}/complete`、`POST /uploads/{upload_id}/cancel` | multipart 上传生命周期 |
| `containers` | `GET/POST /containers`、`GET/DELETE /containers/{container_id}`、`GET/POST /containers/{container_id}/files`、`GET/DELETE /containers/{container_id}/files/{file_id}`、`GET /containers/{container_id}/files/{file_id}/content` | 容器与嵌套文件流程 |
| `assistants` | `GET/POST /assistants`、`GET/POST/DELETE /assistants/{assistant_id}` | 兼容 assistants 管理 |
| `threads` | `POST /threads`、`GET/POST/DELETE /threads/{thread_id}`、`GET/POST /threads/{thread_id}/messages`、`GET/POST/DELETE /threads/{thread_id}/messages/{message_id}` | assistant thread 与 message 管理 |
| `runs` | `POST /threads/runs`、`GET/POST /threads/{thread_id}/runs`、`GET/POST /threads/{thread_id}/runs/{run_id}`、`POST /threads/{thread_id}/runs/{run_id}/cancel`、`POST /threads/{thread_id}/runs/{run_id}/submit_tool_outputs`、`GET /threads/{thread_id}/runs/{run_id}/steps`、`GET /threads/{thread_id}/runs/{run_id}/steps/{step_id}` | assistant run 编排与 run-step 面 |
| `conversations` | `GET/POST /conversations`、`GET/POST/DELETE /conversations/{conversation_id}`、`GET/POST /conversations/{conversation_id}/items`、`GET/DELETE /conversations/{conversation_id}/items/{item_id}` | 面向 response 风格负载的 conversation 流程 |
| `vector-stores` | `GET/POST /vector_stores`、`GET/POST/DELETE /vector_stores/{vector_store_id}`、嵌套 search、files、file batches | 检索与导入流程 |
| `batches` | `GET/POST /batches`、`GET /batches/{batch_id}`、`POST /batches/{batch_id}/cancel` | 异步批处理工作流 |
| `fine-tuning` | `GET/POST /fine_tuning/jobs`，以及 retrieve、cancel、events、checkpoints、pause、resume、checkpoint permissions | 覆盖较完整的 fine-tuning 家族 |
| `webhooks` | `GET/POST /webhooks`、`GET/POST/DELETE /webhooks/{webhook_id}` | 兼容 webhook CRUD |
| `realtime` | `POST /realtime/sessions` | realtime session 创建 |
| `evals` | `GET/POST /evals`、`GET/POST/DELETE /evals/{eval_id}`、嵌套 runs 和 output item 路由 | 评估工作流 |
| `video.openai` | `GET/POST /videos`，以及 retrieve、delete、content、remix、edits、extensions、extend、character 路由 | 共享默认视频镜像家族，对应官方 `/v1/videos*` 契约，并覆盖 Sora 2 与 Sora 2 Pro |
| `video.kling` | `POST /api/v1/services/aigc/video-generation/video-synthesis`、`GET /api/v1/tasks/{task_id}` | 面向可灵兼容客户端的 provider-specific 视频镜像家族，复用共享 DashScope 官方异步传输 |
| `video.aliyun` | `POST /api/v1/services/aigc/video-generation/video-synthesis`、`GET /api/v1/tasks/{task_id}` | 面向阿里云兼容客户端的 provider-specific 视频镜像家族，复用共享 DashScope 官方异步传输 |
| `video.google-veo` | `POST /v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predictLongRunning`、`POST /v1/projects/{project}/locations/{location}/publishers/google/models/{model}:fetchPredictOperation` | Google Veo provider-specific 视频镜像家族，并通过 `{model}` 覆盖 Veo 3 类模型 |
| `video.minimax` | `POST /v1/video_generation`、`GET /v1/query/video_generation`、`GET /v1/files/retrieve` | MiniMax provider-specific 视频镜像家族，直接复用官方异步视频路径 |
| `video.vidu` | `POST /ent/v2/text2video`、`POST /ent/v2/img2video`、`POST /ent/v2/reference2video`、`GET /ent/v2/tasks/{id}/creations`、`POST /ent/v2/tasks/{id}/cancel` | Vidu provider-specific 视频镜像家族，直接复用官方异步视频路径 |
| `video.volcengine` | `POST /api/v1/contents/generations/tasks`、`GET /api/v1/contents/generations/tasks/{id}` | 火山引擎 provider-specific 视频镜像家族，直接复用官方异步任务路径 |
| `music.openai` | `GET/POST /music`、`GET/DELETE /music/{music_id}`、`GET /music/{music_id}/content`、`POST /music/lyrics` | 共享默认音乐镜像家族，对应官方 `/v1/music*` 契约 |
| `music.google` | `POST /v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predict` | Google provider-specific 音乐镜像家族，直接复用官方 Vertex AI predict 路径 |
| `music.minimax` | `POST /v1/music_generation`、`POST /v1/lyrics_generation` | MiniMax provider-specific 音乐镜像家族，直接复用官方生成路径 |
| `music.suno` | `POST /api/v1/generate`、`GET /api/v1/generate/record-info`、`POST /api/v1/lyrics`、`GET /api/v1/lyrics/record-info` | Suno provider-specific 音乐镜像家族，直接复用官方传输路径 |
| `market` | `GET /market/products`、`GET /market/offers`、`POST /market/quotes` | 公共 API 商品目录、offer 发现与报价流程 |
| `marketing` | `POST /marketing/coupons/validate`、`POST /marketing/coupons/reserve`、`POST /marketing/coupons/confirm`、`POST /marketing/coupons/rollback` | coupon-first 校验、预留、核销与回滚面 |
| `commercial` | `GET /commercial/account`、`GET /commercial/account/benefit-lots` | 商业账户摘要、benefit-lot 遍历与 coupon/account-arrival 证据查询 |

## OpenAPI Tag To Capability Docs

当你希望从真实 OpenAPI tag 跳转到按能力组织的文档目录时，直接使用这一节。

| OpenAPI tag | 能力文档 | 状态 | 说明 |
|---|---|---|---|
| `audio.openai` | [audio](/zh/api-reference/gateway-capabilities/audio) | live | 共享 `/v1/audio/*` 音频家族 |
| `code.openai` | [code/openai](/zh/api-reference/gateway-capabilities/code/openai) | live | 共享 OpenAI 与 Codex 家族 |
| `code.claude` | [code/claude](/zh/api-reference/gateway-capabilities/code/claude) | live | Claude 官方镜像家族 |
| `code.gemini` | [code/gemini](/zh/api-reference/gateway-capabilities/code/gemini) | live | Gemini 官方镜像家族 |
| `images.openai` | [images/openai](/zh/api-reference/gateway-capabilities/images/openai) | live | 共享默认图片家族 |
| `images.kling` | [images/kling](/zh/api-reference/gateway-capabilities/images/kling) | live | 共享 DashScope 图片传输 |
| `images.aliyun` | [images/aliyun](/zh/api-reference/gateway-capabilities/images/aliyun) | live | 共享 DashScope 图片传输 |
| `images.volcengine` | [images/volcengine](/zh/api-reference/gateway-capabilities/images/volcengine) | live | 火山引擎 Ark 官方图片传输 |
| `video.openai` | [video/openai](/zh/api-reference/gateway-capabilities/video/openai) | live | 共享默认视频家族 |
| `video.kling` | [video/kling](/zh/api-reference/gateway-capabilities/video/kling) | live | 共享 DashScope 视频传输 |
| `video.aliyun` | [video/aliyun](/zh/api-reference/gateway-capabilities/video/aliyun) | live | 共享 DashScope 视频传输 |
| `video.google-veo` | [video/google-veo](/zh/api-reference/gateway-capabilities/video/google-veo) | live | Vertex AI 官方 Veo 传输 |
| `video.minimax` | [video/minimax](/zh/api-reference/gateway-capabilities/video/minimax) | live | MiniMax 官方视频传输 |
| `video.vidu` | [video/vidu](/zh/api-reference/gateway-capabilities/video/vidu) | live | Vidu 官方视频传输 |
| `video.volcengine` | [video/volcengine](/zh/api-reference/gateway-capabilities/video/volcengine) | live | 火山引擎官方视频传输 |
| `music.openai` | [music/openai](/zh/api-reference/gateway-capabilities/music/openai) | live | 共享默认音乐家族 |
| `music.google` | [music/google](/zh/api-reference/gateway-capabilities/music/google) | live | Google 官方音乐传输 |
| `music.minimax` | [music/minimax](/zh/api-reference/gateway-capabilities/music/minimax) | live | MiniMax 官方音乐传输 |
| `music.suno` | [music/suno](/zh/api-reference/gateway-capabilities/music/suno) | live | Suno 官方音乐传输 |

能力文档目录还包含若干仅用于导航的 docs-only 条目，它们不会单独发布实时 OpenAPI tag：

| 仅文档能力目录 | 映射到 | 状态 | 说明 |
|---|---|---|---|
| [images/nanobanana](/zh/api-reference/gateway-capabilities/images/nanobanana) | `code.gemini` | alias | 以图片视角浏览 Gemini 原生出图能力 |
| [images/midjourney](/zh/api-reference/gateway-capabilities/images/midjourney) | none | unpublished | 在只切换 `base_url` 的镜像约束下仍属于未发布缺口 |
| [video/sora2](/zh/api-reference/gateway-capabilities/video/sora2) | `video.openai` | alias | Sora 2 与 Sora 2 Pro 继续复用共享 OpenAI 视频传输 |

当前网关已发布四个活跃图片镜像 tag，分别落在三组公开路径家族上：共享 `images.openai` 使用 `/v1/images*`，provider-specific 的 `images.kling` 与 `images.aliyun` 复用官方 DashScope `/api/v1/services/aigc/image-generation/generation` 与 `/api/v1/tasks/{task_id}` 路径，provider-specific 的 `images.volcengine` 则复用火山引擎 Ark 官方 `/api/v3/images/generations` 路径。Nano Banana 仍运行在 Google 官方 Gemini `/v1beta/models/{model}:generateContent` 协议面，因此归属 `code.gemini`，而不是单独的 `images.nanobanana` 家族。`images.midjourney` 仍为未发布状态，因为 Midjourney 当前没有可通过仅切换 `base_url` 进行镜像的官方 API 面。

当前音频镜像契约仍保持为共享 `/v1/audio/*` 路由，并以 `audio.openai` 作为公开 tag。公开契约继续停留在这组共享音频路径上，不引入 `/audio/openai/*` 这类 wrapper 路径。

当前网关已发布七个活跃视频镜像家族：共享 `video.openai` 使用 `/v1/videos*`，覆盖 OpenAI 视频客户端与 Sora 2、Sora 2 Pro；provider-specific 的 `video.kling` 与 `video.aliyun` 复用官方 DashScope `/api/v1/services/aigc/video-generation/video-synthesis` 与 `/api/v1/tasks/{task_id}` 路径；provider-specific 的 `video.google-veo` 复用 Google Vertex AI 官方 `/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predictLongRunning` 与 `:fetchPredictOperation` 路径，并通过 `{model}` 覆盖 Veo 3 类模型；provider-specific 的 `video.minimax` 复用 MiniMax 官方 `/v1/video_generation`、`/v1/query/video_generation` 与 `/v1/files/retrieve` 路径；provider-specific 的 `video.vidu` 复用 Vidu 官方 `/ent/v2/text2video`、`/ent/v2/img2video`、`/ent/v2/reference2video`、`/ent/v2/tasks/{id}/creations` 与 `/ent/v2/tasks/{id}/cancel` 路径；provider-specific 的 `video.volcengine` 复用火山引擎官方 `/api/v1/contents/generations/tasks` 与 `/api/v1/contents/generations/tasks/{id}` 路径。由于 OpenAI 已定义官方 Sora 传输，网关不会再额外发布 `video.sora` 家族。

当前网关已发布四个活跃音乐镜像家族：共享 `music.openai` 使用 `/v1/music*`，provider-specific 的 `music.google` 复用 Google Vertex AI 官方 `/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predict` 路径，provider-specific 的 `music.minimax` 复用 MiniMax 官方 `/v1/music_generation` 与 `/v1/lyrics_generation` 路径，provider-specific 的 `music.suno` 复用 Suno 官方 `/api/v1/generate*` 与 `/api/v1/lyrics*` 路径。

## 网关语义

- 公开契约规则：保留官方客户端路径，只切换网关 `base_url`
- provider 选择由 models、route keys 和 routing policy 共同决定
- 在有状态模式下，usage 与 billing 绑定到已鉴权项目
- 创建类路由在记录 usage 时可同时保留 route-key 选择与创建后的资源 ID 关联
- chat、completions、responses、embeddings、moderations 这类生成接口即使上游返回资源 ID，计费仍保持以请求模型为主键
- images、videos、music 这类生成型媒体接口在保留下游资源 ID 的同时，计费仍以请求模型为主键
- commercial benefit-lot 遍历支持 `after_lot_id` 与 `limit`，并返回 `page.after_lot_id`、`page.next_after_lot_id`、`page.has_more` 与 `page.returned_count`
- coupon 到账户到账证据继续通过 `GET /commercial/account/benefit-lots` 上的 `scope_order_id` 显式暴露

## 常用 Header

| Header | 用途 |
|---|---|
| `Authorization` | gateway API key |
| `Content-Type` | JSON、multipart 或兼容上游媒体类型 |
| `x-request-id` | 请求关联 |
| `x-sdkwork-region` | geo-affinity provider 选择的可选提示 |

## 相关文档

- [网关能力索引](/zh/api-reference/gateway-capabilities)
- [能力矩阵](/zh/api-reference/gateway-capabilities/matrix)
- 公开契约与执行真值：
  - [API 兼容矩阵](/zh/reference/api-compatibility)
  - [完整兼容矩阵](/api/compatibility-matrix)
- 控制平面：
  - [管理端 API](/zh/api-reference/admin-api)
