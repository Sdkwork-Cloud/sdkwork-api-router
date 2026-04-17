# 网关 Code 家族

按能力角度浏览网关里的代码与代理编程相关协议面。

## 共享默认家族

| 家族 | 真实 OpenAPI tag | 公开路径家族 | 说明 |
|---|---|---|---|
| 默认共享 code 家族 | `code.openai` | `/v1/*` | OpenAI 兼容数据面，同时覆盖 Codex 能力 |

## 目录

| 目录 | 真实家族 | 公开路径家族 | 说明 |
|---|---|---|---|
| [openai](./code/openai) | `code.openai` | `/v1/*` | 共享 OpenAI 与 Codex 镜像家族 |
| [claude](./code/claude) | `code.claude` | `/v1/messages*` | 官方 Claude 镜像家族 |
| [gemini](./code/gemini) | `code.gemini` | `/v1beta/models/{model}:*` | 官方 Gemini 镜像家族，包含可出图 Gemini 模型 |
