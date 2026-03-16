# 软件架构

本页说明 SDKWork API Server 作为产品与代码库是如何组装起来的。

## 系统视图

| 运行面 | 主要职责 | 关键路径 |
|---|---|---|
| gateway | OpenAI 兼容数据平面与 provider 分发 | `services/gateway-service`、`crates/sdkwork-api-interface-http` |
| admin | 运维控制平面 | `services/admin-api-service`、`crates/sdkwork-api-interface-admin` |
| portal | 终端用户自助边界 | `services/portal-api-service`、`crates/sdkwork-api-interface-portal` |
| console | 浏览器与 Tauri UI 外壳 | `console/` |
| docs | 文档产品 | `docs/` |

## 请求流

一个典型的有状态网关请求会经历以下路径：

1. 客户端调用 `gateway-service` 的 `/v1/*` 路由
2. `sdkwork-api-interface-http` 完成网关 API key 鉴权并映射 HTTP 路由
3. 应用层 crate 解析模型、provider、credential、routing policy、quota 与 runtime state
4. provider runtime 通过 builtin、connector 或 native-dynamic 路径执行
5. usage 与 billing 记录通过 admin-store 契约持久化
6. OpenAI 兼容响应返回给调用方

admin 与 portal 服务整体分层相同，但它们终止在原生控制平面工作流，而不是 provider dispatch。

## 工作区分层

| 分层 | 职责 | 示例 |
|---|---|---|
| interface | HTTP 路由、鉴权边界、响应整形 | `sdkwork-api-interface-http`、`sdkwork-api-interface-admin`、`sdkwork-api-interface-portal` |
| app | 用例编排与服务工作流 | `sdkwork-api-app-gateway`、`sdkwork-api-app-routing`、`sdkwork-api-app-billing`、`sdkwork-api-app-extension` |
| domain | 策略规则与核心领域概念 | `sdkwork-api-domain-routing`、`sdkwork-api-domain-billing`、`sdkwork-api-domain-usage` |
| storage | 持久化契约与具体后端 | `sdkwork-api-storage-core`、`sdkwork-api-storage-sqlite`、`sdkwork-api-storage-postgres` |
| provider | 上游适配器实现 | `sdkwork-api-provider-openai`、`sdkwork-api-provider-openrouter`、`sdkwork-api-provider-ollama` |
| runtime | 扩展加载、嵌入宿主、监督运行 | `sdkwork-api-runtime-host`、`sdkwork-api-extension-host`、`sdkwork-api-app-runtime` |
| contracts | OpenAI 与网关 API 协议形状 | `sdkwork-api-contract-openai`、`sdkwork-api-contract-gateway` |

## 运行时架构

SDKWork 同时支持独立服务与嵌入式两类运行形态。

- 三个独立服务分别监听 gateway、admin、portal HTTP 接口
- 浏览器模式下，console 直接调用这些监听地址
- 同一套 console 也可以由 Tauri 承载，用于桌面运行
- 扩展运行时支持 builtin、connector、native-dynamic 三种形态
- 运行时配置、监听器重绑定、存储切换与 secret-manager 轮换优先走热重载，而不是整进程重启

更深入的运行时说明见 [运行模式详解](/zh/architecture/runtime-modes)。

## 配置与密钥边界

配置通过 `sdkwork-api-config` 加载，再注入进进程与运行时句柄。

关键边界包括：

- 服务绑定地址
- 数据库后端与连接串
- admin 与 portal 的 JWT 签名密钥
- provider 凭据主密钥
- 扩展发现路径与信任策略

密钥材料刻意与路由逻辑分离：

- 服务从配置中解析 secret-manager 策略
- 凭据通过加密后端持久化
- 通过 locator 与 key lineage 元数据，历史凭据在密钥轮换后依然可读

## 持久化模型

共享 admin store 是以下数据的系统事实来源：

- tenants 与 projects
- gateway API keys
- channels、providers、credentials 与 models
- routing policy 与 decision logs
- usage records、billing ledger 与 quota policy
- extension installation 与 rollout 状态

仓库中当前文档化的后端包括：

- SQLite
- PostgreSQL

## 前端架构

console 被拆分为轻量外壳和可复用业务包：

- `console/src/` 负责应用组合
- `console/packages/` 承担复用的 API、auth、routing、usage 与 workspace 模块
- `console/src-tauri/` 负责桌面宿主集成

这让浏览器版和桌面版可以共享业务实现，同时把原生差异约束在 Tauri 边界内。

## 运维架构

三个独立服务都暴露：

- health 端点
- Prometheus 风格 metrics
- 结构化请求追踪

其中 admin 控制平面额外负责：

- extension runtime reload 与 rollout
- standalone config rollout
- routing simulation
- health snapshot 查看
- usage 与 billing 可视化

## 相关文档

- 模块职责映射：
  - [功能模块](/zh/architecture/functional-modules)
- 接口清单：
  - [API 参考总览](/zh/api-reference/overview)
- 配置与可观测性：
  - [配置说明](/zh/operations/configuration)
  - [健康检查与 Metrics](/zh/operations/health-and-metrics)
