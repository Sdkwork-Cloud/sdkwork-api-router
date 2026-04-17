# code/openai

## 真实镜像家族

- OpenAPI tag：`code.openai`
- 公开路径家族：`/v1/*`
- 契约规则：只切换 `base_url`

## 主要路由

- `GET /v1/models`
- `POST /v1/chat/completions`
- `POST /v1/completions`
- `POST /v1/responses`
- `POST /v1/embeddings`
- `POST /v1/moderations`

## 说明

- Codex 归在这个共享家族中，不额外发明 wrapper 路径。
