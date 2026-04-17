# Gateway Code Families

Browse the gateway code and agent-oriented surfaces by capability first.

## Shared Default Family

| Family | Live OpenAPI tag | Public path family | Notes |
|---|---|---|---|
| default shared code family | `code.openai` | `/v1/*` | OpenAI-compatible data plane, including Codex-capable flows |

## Provider Directories

| Directory | Live family | Public path family | Notes |
|---|---|---|---|
| [openai](./code/openai) | `code.openai` | `/v1/*` | shared OpenAI and Codex mirror family |
| [claude](./code/claude) | `code.claude` | `/v1/messages*` | official Claude mirror family |
| [gemini](./code/gemini) | `code.gemini` | `/v1beta/models/{model}:*` | official Gemini mirror family, including image-capable Gemini models |
