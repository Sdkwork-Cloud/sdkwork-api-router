# code/openai

## Live Mirror Family

- OpenAPI tag: `code.openai`
- Public path family: `/v1/*`
- Contract rule: switch only `base_url`

## Primary Routes

- `GET /v1/models`
- `POST /v1/chat/completions`
- `POST /v1/completions`
- `POST /v1/responses`
- `POST /v1/embeddings`
- `POST /v1/moderations`

## Notes

- Codex belongs to this shared mirror family instead of a separate wrapper path.
- This is the default shared code contract when OpenAI already defines the standard route.
