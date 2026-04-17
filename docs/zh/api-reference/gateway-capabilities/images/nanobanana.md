# images/nanobanana

## 能力别名

- 别名状态：仅文档别名
- 真实镜像家族：`code.gemini`
- 公开路径家族：`/v1beta/models/{model}:generateContent`

## 为什么需要这一页

- 很多读者会先按图片能力查找 Nano Banana。
- 真实网关仍然必须保留 Google 官方 Gemini 路径，确保客户端只切换 `base_url` 即可使用。
- 所以这里提供的是能力别名页，而不是额外发明一个新的 `images.nanobanana` OpenAPI tag。

## 实际应使用的路由

- `POST /v1beta/models/{model}:generateContent`
- `POST /v1beta/models/{model}:streamGenerateContent?alt=sse`

## 相关文档

- [code/gemini](../code/gemini)
- [网关 API](/zh/api-reference/gateway-api)
