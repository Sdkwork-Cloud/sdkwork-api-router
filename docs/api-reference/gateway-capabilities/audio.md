# Gateway Audio Families

The audio capability index currently has one shared live family.

## Shared Default Audio Family

| Family | Live OpenAPI tag | Public path family | Status | Notes |
|---|---|---|---|---|
| default shared audio family | `audio.openai` | `/v1/audio/*` | active | transcription, translation, speech, voices, and voice consent stay on the shared audio contract |

## Shared Default API Inventory

- `POST /v1/audio/transcriptions`
- `POST /v1/audio/translations`
- `POST /v1/audio/speech`
- `GET /v1/audio/voices`
- `POST /v1/audio/voice_consents`

## Notes

- Audio currently does not publish provider-specific mirror directories.
- The shared contract remains `audio.openai` because the current live router keeps one official OpenAI-style audio family.

## Related Docs

- [Capability Matrix](./matrix)
- [Gateway Capability Index](/api-reference/gateway-capabilities)
