# API 兼容矩阵

SDKWork 使用五种执行真值标签来描述网关接口的真实实现方式。

## 执行真值标签

| 标签 | 含义 |
|---|---|
| `native` | 由 SDKWork 直接实现 |
| `relay` | 透明转发到兼容上游 |
| `translated` | 在本地接受请求后，映射到不同的上游原语 |
| `emulated` | 本地返回兼容形状的响应 |
| `unsupported` | 当前运行时下不可用 |

这些标签描述的是运行时真相，不是公开 URL 的分类方式。

## 公开契约规则

- 保留官方 provider 路径，只切换网关 `base_url`
- 如果某个能力已经存在 OpenAI 标准路由，则优先复用该标准作为共享公开契约
- 如果不存在共享标准，则暴露 provider 官方协议路径作为镜像协议面
- 不发明 `/code/*`、`/claude/*`、`/gemini/*` 这类 wrapper 前缀

## 镜像协议家族

- `code.openai`：OpenAI 与 Codex 的 `/v1/*`
- `code.claude`：Claude 的 `/v1/messages` 与 `/v1/messages/count_tokens`
- `code.gemini`：Gemini 的 `/v1beta/models/{model}:*`
- `images.openai`：OpenAI 图片协议 `/v1/images/*`
- `audio.openai`：共享音频协议 `/v1/audio/*`
- `video.openai`：共享视频协议 `/v1/videos*`
- `music.openai`：共享音乐协议 `/v1/music*`

## 高价值 API 家族

当前已覆盖的主要网关家族包括：

- `/v1/models`
- `/v1/chat/completions`
- `/v1/messages`
- `/v1/completions`
- `/v1/responses`
- `/v1beta/models/{model}:generateContent`
- `/v1beta/models/{model}:streamGenerateContent`
- `/v1beta/models/{model}:countTokens`
- `/v1/embeddings`
- `/v1/files`
- `/v1/uploads`
- `/v1/audio/*`
- `/v1/images/*`
- `/v1/music`
- `/v1/assistants`
- `/v1/threads`
- `/v1/conversations`
- `/v1/vector_stores`
- `/v1/batches`
- `/v1/fine_tuning/jobs`
- `/v1/webhooks`
- `/v1/evals`
- `/v1/videos`

`audio` 能力当前以共享 `audio.openai` 镜像家族的形式发布在 `/v1/audio/*` 上，公开契约不会引入 `/audio/openai/*` 这类 wrapper 前缀。

`music` 能力当前以共享 `music.openai` 镜像家族的形式发布在 `/v1/music*` 上，继续采用资源化路由，而不是绑定单一上游厂商的私有传输路径，这样可以与图片、视频一样复用统一的路由、计费和插件适配架构。

图片当前激活的公开镜像家族是 `images.openai`，视频当前激活的公开镜像家族是 `video.openai`。保留镜像家族如 `images.nanobanana`、`images.midjourney`、`images.volcengine`、`images.aliyun`、`images.kling`、`video.sora`、`video.minimax`、`video.vidu`、`video.volcengine`、`video.google-veo`、`video.aliyun`、`video.kling`、`music.suno`、`music.google`、`music.minimax` 当前都只是治理名称，还没有作为公开 OpenAPI tag 或可调用路由发布。

## Agent 客户端兼容面

网关现在提供两组一等镜像协议面，方便现有 agent 客户端直接接入：

- Claude 镜像协议面：`POST /v1/messages` 与 `POST /v1/messages/count_tokens`，适用于 Claude Code 等客户端
- Gemini 镜像协议面：`POST /v1beta/models/{model}:generateContent`、`POST /v1beta/models/{model}:streamGenerateContent?alt=sse`、`POST /v1beta/models/{model}:countTokens`，适用于 Gemini CLI gateway mode 等客户端

这些接口不会绕开 SDKWork 的路由系统，而是转换到现有 OpenAI 兼容执行链路，因此 provider 选择、项目路由偏好、配额控制、计费和 usage 记录都会与 `/v1/*` 网关保持一致。

有状态网关部署除了 `Authorization: Bearer ...` 之外，还支持官方协议原生认证入口：

- Claude 面：`x-api-key`
- Gemini 面：`x-goog-api-key` 或 `?key=...`

## 如何使用这份信息

- 如果你需要快速判断某一类接口是否能在当前运行时执行，先看完整矩阵
- 如果你需要了解公开镜像协议家族、基地址与鉴权方式，再看 [网关 API](/zh/api-reference/gateway-api)
- 如果你需要理解真实执行语义，再看这份兼容文档

## 进一步阅读

- [API 参考总览](/zh/api-reference/overview)
- [网关 API](/zh/api-reference/gateway-api)
- [完整兼容矩阵](/api/compatibility-matrix)
