# 网关 Images 家族

这一页先列共享默认图片契约，再列 provider 与别名目录。

## 共享默认图片家族

| 家族 | 真实 OpenAPI tag | 公开路径家族 | 说明 |
|---|---|---|---|
| 默认共享图片家族 | `images.openai` | `/v1/images/*` | generations、edits、variations 继续沿用 OpenAI 官方图片契约 |

## 共享默认 API 清单

- `POST /v1/images/generations`
- `POST /v1/images/edits`
- `POST /v1/images/variations`

## Images 目录

| 目录 | 真实家族 | 公开路径家族 | 状态 | 说明 |
|---|---|---|---|---|
| [openai](./images/openai) | `images.openai` | `/v1/images/*` | active | 默认共享图片契约 |
| [kling](./images/kling) | `images.kling` | `/api/v1/services/aigc/image-generation/generation` 与 `/api/v1/tasks/{task_id}` | active | 共享 DashScope 官方图片传输 |
| [aliyun](./images/aliyun) | `images.aliyun` | `/api/v1/services/aigc/image-generation/generation` 与 `/api/v1/tasks/{task_id}` | active | 共享 DashScope 官方图片传输 |
| [volcengine](./images/volcengine) | `images.volcengine` | `/api/v3/images/generations` | active | 火山引擎官方图片传输 |
| [nanobanana](./images/nanobanana) | 别名到 `code.gemini` | `/v1beta/models/{model}:generateContent` | 文档别名 | 面向图片视角的 Gemini 能力索引页 |
| [midjourney](./images/midjourney) | 未发布 | 无 | unpublished | 当前没有满足镜像规则的官方 Midjourney API 面 |
