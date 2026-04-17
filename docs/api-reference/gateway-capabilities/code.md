# Gateway Code Families

Browse the gateway code and agent-oriented surfaces by capability first.

## Shared Default Family

| Family | Live OpenAPI tag | Public path family | Notes |
|---|---|---|---|
| default shared code family | `code.openai` | `/v1/*` | OpenAI-compatible data plane, including Codex-capable flows |

## Shared Default API Inventory

- `GET /v1/models`
- `GET /v1/models/{model_id}`
- `POST /v1/chat/completions`
- `POST /v1/completions`
- `POST /v1/responses`
- `POST /v1/embeddings`
- `POST /v1/moderations`

## Provider Directories

| Directory | Live family | Public path family | Notes |
|---|---|---|---|
| [openai](./code/openai) | `code.openai` | `/v1/*` | shared OpenAI and Codex mirror family |
| [claude](./code/claude) | `code.claude` | `/v1/messages*` | official Claude mirror family |
| [gemini](./code/gemini) | `code.gemini` | `/v1beta/models/{model}:*` | official Gemini mirror family, including image-capable Gemini models |
