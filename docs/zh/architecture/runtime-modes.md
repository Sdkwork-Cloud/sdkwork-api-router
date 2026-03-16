# 运行模式详解

本页从更深的实现视角说明 SDKWork 当前支持的运行形态、扩展运行时状态，以及配置和路由行为。

## 独立服务模式

独立服务模式是当前最完整的部署形态。

特点：

- `gateway-service`、`admin-api-service`、`portal-api-service` 作为独立二进制运行
- gateway、admin、portal API 都通过 HTTP 暴露
- 共享 admin store 既支持 SQLite，也支持 PostgreSQL
- 上游凭据通常通过服务端 secret backend 管理
- 更适合浏览器接入、多用户使用和服务化部署

## 嵌入模式

嵌入模式是面向桌面的运行形态。

特点：

- 运行时可通过 `sdkwork-api-runtime-host` 在进程内承载
- `console/src-tauri/` 主要承载 admin 控制台
- portal 现已作为位于 `apps/sdkwork-router-portal/` 的独立浏览器应用
- 默认信任边界是本机 loopback
- SQLite 是更适合本地桌面的首选持久化策略
- 当系统支持时，OS keyring 是更合适的密钥后端

## 当前实现状态

当前仓库已具备以下基础：

- 独立的 gateway、admin、portal 三套服务二进制
- `console/` 下的 landing 与 admin 控制台
- `apps/sdkwork-router-portal/` 下的独立 portal 应用，包含 dashboard、usage、credits、billing、API key 和 account 模块
- 公开门户注册、登录、工作区查看、dashboard、usage、billing 与 API key 签发
- 基于 SQLite 与 PostgreSQL 的共享控制平面存储
- 经过加密的 provider credential 持久化与运行时 secret 解析
- 有状态 OpenAI 兼容网关，可在 provider 配置存在时执行 chat、responses、embeddings 等请求
- 基于环境和本地配置文件的运行时配置加载
- admin JWT 与 portal JWT 的独立签发边界
- 通过持久化 gateway API key 推导租户与项目上下文
- Prometheus 风格 `/metrics` 与 `x-request-id` 传播
- 路由策略、健康状态、决策日志、用量与计费记录的持久化

## 扩展运行时状态

扩展架构是分层设计的。

| 运行时 | 当前状态 | 说明 |
|---|---|---|
| `builtin` | 已启用 | 第一方 provider 扩展通过 `sdkwork-api-extension-host` 在进程内注册 |
| `native_dynamic` | 已启用 | 可信包可通过动态库加载，以受限 ABI 执行 JSON 和部分流式能力 |
| `connector` | 已启用 | 宿主可拉起受监督的外部进程，探测健康状态并复用已健康的端点 |

## 配置层次

当前扩展运行时把三个概念分开管理：

| 层次 | 职责 |
|---|---|
| `ExtensionManifest` | 包身份、兼容性和能力声明 |
| `ExtensionInstallation` | 已安装运行时选择、信任状态、启用状态和包级配置 |
| `ExtensionInstance` | 环境相关配置，例如 `base_url`、`credential_ref` 和实例级参数 |

这意味着一个扩展包可以在不同运行模式下支撑多个具体实例。

## 配置与热重载边界

当前运行时优先通过热重载而不是重启进程来应用变更。典型边界包括：

- 本地配置文件轮询重载
- extension tree 变更后的自动热重载
- admin 触发的 extension runtime reload
- admin 触发的 runtime-config rollout
- 数据库连接、JWT 签名密钥、secret backend 和监听地址的热切换

## 路由与执行特征

当前路由行为保持保守且可解释：

- 策略优先级按 `priority DESC` 再按 `policy_id ASC` 排序
- 模型匹配同时支持精确值和带 `*` 的 glob 风格匹配
- provider 选择会综合可用性、运行时健康状态以及实例级 `cost`、`latency_ms`、`weight` 提示
- 当没有实时运行时状态时，可回退到最新持久化健康快照
- `/v1/chat/completions`、`/v1/completions`、`/v1/responses`、`/v1/embeddings` 支持项目级 quota-aware admission
- `geo_affinity` 路由会优先选择与 `x-sdkwork-region` 匹配的 provider 实例，并在无匹配时安全降级

## 管理端运行时控制

admin 控制平面当前可负责：

- 查看扩展包与运行时状态
- 触发整机、单扩展、单 connector 实例的 runtime reload
- 通过持久化 rollout 记录协调多节点扩展 rollout
- 通过共享存储协调 gateway、admin、portal 节点的配置 rollout
- 查看 routing simulation、health snapshot 和 decision log

## 相关文档

- 快速理解运行形态：
  - [运行模式](/zh/getting-started/runtime-modes)
- 系统边界：
  - [软件架构](/zh/architecture/software-architecture)
- 配置与运维：
  - [配置说明](/zh/operations/configuration)
  - [健康检查与 Metrics](/zh/operations/health-and-metrics)
