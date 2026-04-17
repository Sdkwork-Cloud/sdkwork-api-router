# 网关能力索引

这一组文档在现有网关 OpenAPI 清单之上，补了一层“按能力浏览”的索引视图。

它只负责发现与导航，不改变真实公开契约：

- 客户端仍然只需要切换网关 `base_url`
- 官方 provider 路径保持不变
- 真实路由与 OpenAPI tag 仍以当前网关实现为准

当你更关心 `code / images / video / music` 这类能力家族，而不是直接看 OpenAPI tag 清单时，先从这里进入。

## 能力家族

| 家族 | 先看共享默认家族 | 再看 provider 或别名目录 |
|---|---|---|
| [Code](/zh/api-reference/gateway-capabilities/code) | `code.openai`，路径 `/v1/*` | `openai`、`claude`、`gemini` |
| [Images](/zh/api-reference/gateway-capabilities/images) | `images.openai`，路径 `/v1/images/*` | `openai`、`kling`、`aliyun`、`volcengine`、`nanobanana`、`midjourney` |
| [Video](/zh/api-reference/gateway-capabilities/video) | `video.openai`，路径 `/v1/videos*` | `openai`、`sora2`、`kling`、`aliyun`、`google-veo`、`minimax`、`vidu`、`volcengine` |
| [Music](/zh/api-reference/gateway-capabilities/music) | `music.openai`，路径 `/v1/music*` | `openai`、`google`、`minimax`、`suno` |

## 如何理解这一层

- 共享默认家族优先：如果 OpenAI 已有标准公开路径，就继续沿用这一套共享契约。
- provider 目录其次：如果没有共享 OpenAI 标准，就直接镜像对应 provider 的官方路径。
- 别名目录补充：有些名字对检索很重要，但真实网关不会为它额外发明一个新的 OpenAPI tag。

例子：

- `images/nanobanana` 是文档别名页，但真实协议家族是 `code.gemini`，路径 `/v1beta/models/{model}:generateContent`
- `video/sora2` 是文档别名页，但真实协议家族是 `video.openai`，路径 `/v1/videos*`
- `images/midjourney` 是未发布能力说明页，因为当前没有满足“只切 base_url 即可镜像”的官方 Midjourney API 面

## 相关文档

- [网关 API](/zh/api-reference/gateway-api)
- [API 兼容矩阵](/zh/reference/api-compatibility)
- [完整兼容矩阵](/api/compatibility-matrix)
