# Gateway Music Families

The music capability index starts with the shared default contract and then lists provider directories.

## Shared Default Music Family

| Family | Live OpenAPI tag | Public path family | Notes |
|---|---|---|---|
| default shared music family | `music.openai` | `/v1/music*` | list, create, retrieve, delete, content, and lyrics stay on the shared OpenAI-style music contract |

## Shared Default API Inventory

- `GET /v1/music`
- `POST /v1/music`
- `GET /v1/music/{music_id}`
- `DELETE /v1/music/{music_id}`
- `GET /v1/music/{music_id}/content`
- `POST /v1/music/lyrics`

## Music Directories

| Directory | Live family | Public path family | Status | Notes |
|---|---|---|---|---|
| [openai](./music/openai) | `music.openai` | `/v1/music*` | active | shared default music contract |
| [google](./music/google) | `music.google` | `/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predict` | active | official Vertex AI music transport |
| [minimax](./music/minimax) | `music.minimax` | `/v1/music_generation` and `/v1/lyrics_generation` | active | official MiniMax music transport |
| [suno](./music/suno) | `music.suno` | `/api/v1/generate*` and `/api/v1/lyrics*` | active | official Suno music transport |
