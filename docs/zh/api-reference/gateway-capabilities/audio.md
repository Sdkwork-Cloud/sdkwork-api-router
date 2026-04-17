# 网关 Audio 家族

当前音频能力只有一个共享真实家族。

## 共享默认 Audio 家族

| 家族 | 真实 OpenAPI tag | 公开路径家族 | 状态 | 说明 |
|---|---|---|---|---|
| 默认共享 audio 家族 | `audio.openai` | `/v1/audio/*` | active | transcription、translation、speech、voices、voice consent 都继续使用共享音频契约 |

## 共享默认 API 清单

- `POST /v1/audio/transcriptions`
- `POST /v1/audio/translations`
- `POST /v1/audio/speech`
- `GET /v1/audio/voices`
- `POST /v1/audio/voice_consents`

## 说明

- 当前音频能力还没有 provider-specific 的独立镜像目录。
- 真实共享契约仍然是 `audio.openai`。

## 相关文档

- [能力矩阵](./matrix)
- [网关能力索引](/zh/api-reference/gateway-capabilities)
