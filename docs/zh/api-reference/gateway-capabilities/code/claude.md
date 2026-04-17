# code/claude

## 真实镜像家族

- OpenAPI tag：`code.claude`
- 公开路径家族：`/v1/messages*`
- 契约规则：保留 Anthropic 官方请求路径

## 主要路由

- `POST /v1/messages`
- `POST /v1/messages/count_tokens`

## 说明

- Claude Code 与其他 Anthropic 客户端可以只切换 `base_url` 使用。
