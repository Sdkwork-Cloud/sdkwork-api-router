# video/sora2

## 能力别名

- 别名状态：仅文档别名
- 真实镜像家族：`video.openai`
- 公开路径家族：`/v1/videos*`

## 为什么需要这一页

- 很多读者会直接按 Sora 2 名称检索。
- 真实网关仍然保留 OpenAI 官方视频传输，因此不会额外发布 `video.sora` 或 `video.sora2` tag。

## 实际应使用的路由家族

- `GET /v1/videos`
- `POST /v1/videos`
- `GET /v1/videos/{video_id}`
- `GET /v1/videos/{video_id}/content`

## 相关文档

- [video/openai](./openai)
- [网关 API](/zh/api-reference/gateway-api)
