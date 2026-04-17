# 网关 Music 家族

这一页先列共享默认音乐契约，再列 provider 目录。

## 共享默认音乐家族

| 家族 | 真实 OpenAPI tag | 公开路径家族 | 说明 |
|---|---|---|---|
| 默认共享音乐家族 | `music.openai` | `/v1/music*` | list、create、retrieve、delete、content、lyrics 继续沿用共享音乐契约 |

## Music 目录

| 目录 | 真实家族 | 公开路径家族 | 状态 | 说明 |
|---|---|---|---|---|
| [openai](./music/openai) | `music.openai` | `/v1/music*` | active | 默认共享音乐契约 |
| [google](./music/google) | `music.google` | `/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predict` | active | Google Vertex AI 官方音乐传输 |
| [minimax](./music/minimax) | `music.minimax` | `/v1/music_generation` 与 `/v1/lyrics_generation` | active | MiniMax 官方音乐传输 |
| [suno](./music/suno) | `music.suno` | `/api/v1/generate*` 与 `/api/v1/lyrics*` | active | Suno 官方音乐传输 |
