# 网关 API

网关服务在 `/v1/*` 下暴露 OpenAI 兼容数据平面。

## 基础地址与鉴权

- 默认本地基地址：`http://127.0.0.1:8080/v1`
- 主要鉴权方式：`Authorization: Bearer skw_live_...`
- 健康检查：`GET /health`
- Metrics：`GET /metrics`

最小首个请求：

```bash
curl http://127.0.0.1:8080/v1/models \
  -H "Authorization: Bearer skw_live_your_key_here"
```

在独立服务模式下，网关是一个依赖 admin store 的有状态实现。无状态 gateway runtime 仍以库和运行时形态存在，其覆盖范围通过兼容矩阵文档继续说明。

## 路由家族

| 家族 | 路由 | 说明 |
|---|---|---|
| models | `GET /models`、`GET /models/{model_id}`、`DELETE /models/{model_id}` | 有状态模式下基于 catalog |
| chat completions | `GET /chat/completions`、`POST /chat/completions`、`GET/POST/DELETE /chat/completions/{completion_id}`、`GET /chat/completions/{completion_id}/messages` | 支持兼容 JSON 与流式转发 |
| completions | `POST /completions` | 传统文本补全接口 |
| responses | `POST /responses`、`POST /responses/input_tokens`、`POST /responses/compact`、`GET/DELETE /responses/{response_id}`、`GET /responses/{response_id}/input_items`、`POST /responses/{response_id}/cancel` | OpenAI 风格 responses 工作流 |
| embeddings | `POST /embeddings` | 基于请求模型做 provider 选择 |
| moderations | `POST /moderations` | OpenAI 兼容审核接口 |
| images | `POST /images/generations`、`POST /images/edits`、`POST /images/variations` | 生成与 multipart 编辑流程 |
| audio | `POST /audio/transcriptions`、`POST /audio/translations`、`POST /audio/speech`、`GET /audio/voices`、`POST /audio/voice_consents` | 包含二进制语音输出与语音同意创建 |
| files | `GET/POST /files`、`GET/DELETE /files/{file_id}`、`GET /files/{file_id}/content` | 元数据与二进制内容获取 |
| uploads | `POST /uploads`、`POST /uploads/{upload_id}/parts`、`POST /uploads/{upload_id}/complete`、`POST /uploads/{upload_id}/cancel` | multipart 上传生命周期 |
| containers | `GET/POST /containers`、`GET/DELETE /containers/{container_id}`、`GET/POST /containers/{container_id}/files`、`GET/DELETE /containers/{container_id}/files/{file_id}`、`GET /containers/{container_id}/files/{file_id}/content` | 容器与嵌套文件流程 |
| assistants | `GET/POST /assistants`、`GET/POST/DELETE /assistants/{assistant_id}` | 兼容 assistants 管理 |
| threads | `POST /threads`、`GET/POST/DELETE /threads/{thread_id}`、嵌套 messages 和 runs 路由 | 包含 tool output 提交与 run steps |
| conversations | `GET/POST /conversations`、`GET/POST/DELETE /conversations/{conversation_id}`、嵌套 item 路由 | 面向 response 风格负载的 conversation 流程 |
| vector stores | `GET/POST /vector_stores`、`GET/POST/DELETE /vector_stores/{vector_store_id}`、嵌套 search、files、file batches | 检索与导入流程 |
| batches | `GET/POST /batches`、`GET /batches/{batch_id}`、`POST /batches/{batch_id}/cancel` | 异步批处理工作流 |
| fine tuning | `GET/POST /fine_tuning/jobs`，以及 retrieve、cancel、events、checkpoints、pause、resume、checkpoint permissions | 覆盖较完整的 fine-tuning 家族 |
| webhooks | `GET/POST /webhooks`、`GET/POST/DELETE /webhooks/{webhook_id}` | 兼容 webhook CRUD |
| realtime | `POST /realtime/sessions` | realtime session 创建 |
| evals | `GET/POST /evals`、`GET/POST/DELETE /evals/{eval_id}`、嵌套 runs 和 output item 路由 | 评估工作流 |
| videos | `GET/POST /videos`，以及 retrieve、delete、content、remix、edits、extensions、extend、character 路由 | 同时覆盖 canonical 与嵌套路由 |
| music | `GET/POST /music`、`GET/DELETE /music/{music_id}`、`GET /music/{music_id}/content`、`POST /music/lyrics` | 资源化音乐生成、获取、二进制内容读取与歌词生成 |

## 网关语义

- provider 选择由 models、route keys 和 routing policy 共同决定
- 在有状态模式下，usage 与 billing 绑定到已鉴权项目
- 创建类路由在记录 usage 时可同时保留 route-key 选择与创建后的资源 ID 关联
- chat、completions、responses、embeddings、moderations 这类生成接口即使上游返回资源 ID，计费仍保持以请求模型为主键

## 常用 Header

| Header | 用途 |
|---|---|
| `Authorization` | gateway API key |
| `Content-Type` | JSON、multipart 或兼容上游媒体类型 |
| `x-request-id` | 请求关联 |
| `x-sdkwork-region` | geo-affinity provider 选择的可选提示 |

## 相关文档

- 兼容真值：
  - [API 兼容矩阵](/zh/reference/api-compatibility)
  - [完整兼容矩阵](/api/compatibility-matrix)
- 控制平面：
  - [管理端 API](/zh/api-reference/admin-api)
