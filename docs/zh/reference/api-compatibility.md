# API 兼容矩阵

SDKWork 使用五种执行真值标签来描述网关接口的真实实现方式。

## 执行真值标签

| 标签 | 含义 |
|---|---|
| `native` | 由 SDKWork 直接实现 |
| `relay` | 透明转发到兼容上游 |
| `translated` | 在本地接受请求后，映射到不同的上游原语 |
| `emulated` | 本地返回兼容形状的响应 |
| `unsupported` | 当前运行时下不可用 |

## 高价值 API 家族

当前已覆盖的主要网关家族包括：

- `/v1/models`
- `/v1/chat/completions`
- `/v1/completions`
- `/v1/responses`
- `/v1/embeddings`
- `/v1/files`
- `/v1/uploads`
- `/v1/audio/*`
- `/v1/images/*`
- `/v1/assistants`
- `/v1/threads`
- `/v1/conversations`
- `/v1/vector_stores`
- `/v1/batches`
- `/v1/fine_tuning/jobs`
- `/v1/webhooks`
- `/v1/evals`
- `/v1/videos`

## 如何使用这份信息

- 如果你需要快速判断某一类接口是否能在当前运行时执行，先看完整矩阵
- 如果你需要了解应该调用哪个服务，再看 [API 参考总览](/zh/api-reference/overview)
- 如果你需要明确具体路由家族，再看 [网关 API](/zh/api-reference/gateway-api)

## 进一步阅读

- [API 参考总览](/zh/api-reference/overview)
- [网关 API](/zh/api-reference/gateway-api)
- [完整兼容矩阵](/api/compatibility-matrix)
