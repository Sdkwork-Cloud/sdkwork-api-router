# code/gemini

## 真实镜像家族

- OpenAPI tag：`code.gemini`
- 公开路径家族：`/v1beta/models/{model}:*`
- 契约规则：保留 Google 官方 Gemini 传输

## 主要路由

- `POST /v1beta/models/{model}:generateContent`
- `POST /v1beta/models/{model}:streamGenerateContent?alt=sse`
- `POST /v1beta/models/{model}:countTokens`

## 说明

- Nano Banana 这类可出图 Gemini 模型仍然归在这个真实家族中。
- 为了方便按图片能力检索，文档另外提供了 [images/nanobanana](../images/nanobanana) 别名页。
