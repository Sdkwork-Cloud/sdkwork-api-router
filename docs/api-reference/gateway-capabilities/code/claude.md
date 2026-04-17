# code/claude

## Live Mirror Family

- OpenAPI tag: `code.claude`
- Public path family: `/v1/messages*`
- Contract rule: keep Anthropic's official request path

## Primary Routes

- `POST /v1/messages`
- `POST /v1/messages/count_tokens`

## Notes

- This family exists so Claude Code and other Anthropic clients can switch only `base_url`.
- SDKWork still routes these requests through the same governance, quota, and billing path used by the shared gateway runtime.
