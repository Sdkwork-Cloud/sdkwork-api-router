# 软件架构

本页说明 SDKWork API Server 作为产品与代码库是如何组装起来的。

## 系统视图

| 运行面 | 主要职责 | 关键路径 |
|---|---|---|
| gateway | OpenAI 兼容数据面与 provider 分发 | `services/gateway-service`、`crates/sdkwork-api-interface-http` |
| admin | 运维控制平面 | `services/admin-api-service`、`crates/sdkwork-api-interface-admin` |
| portal | 终端用户自助边界 | `services/portal-api-service`、`crates/sdkwork-api-interface-portal` |
| web host | Pingora 公网交付与 API 代理 | `services/router-web-service`、`crates/sdkwork-api-runtime-host` |
| product host | 服务端模式与桌面 sidecar 共用的一体化产品宿主 | `services/router-product-service`、`crates/sdkwork-api-product-runtime` |
| admin app | 独立浏览器应用与显式开发用 Tauri 宿主 | `apps/sdkwork-router-admin/` |
| portal app | 独立浏览器应用与正式 desktop 宿主 | `apps/sdkwork-router-portal/` |
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
| app | 用例编排与服务工作流 | `sdkwork-api-app-gateway`、`sdkwork-api-app-routing`、`sdkwork-api-app-billing` |
| domain | 领域规则与核心概念 | `sdkwork-api-domain-routing`、`sdkwork-api-domain-billing`、`sdkwork-api-domain-usage` |
| storage | 持久化契约与具体后端 | `sdkwork-api-storage-core`、`sdkwork-api-storage-sqlite`、`sdkwork-api-storage-postgres` |
| provider | 上游适配器实现 | `sdkwork-api-provider-openai`、`sdkwork-api-provider-openrouter`、`sdkwork-api-provider-ollama` |
| runtime | 扩展加载、宿主监督、嵌入运行时 | `sdkwork-api-runtime-host`、`sdkwork-api-extension-host`、`sdkwork-api-app-runtime` |
| contracts | OpenAI 与网关 API 结构 | `sdkwork-api-contract-openai`、`sdkwork-api-contract-gateway` |

## 运行时架构

SDKWork 同时支持独立服务和产品宿主两类运行形态。

- 独立服务模式下，gateway、admin、portal 以各自二进制对外监听
- `router-web-service` 负责暴露 admin / portal 静态站点，并代理 `/api/admin/*`、`/api/portal/*` 与 `/api/v1/*`
- `sdkwork-api-product-runtime` 负责把独立监听器和共享 Web 宿主组装为产品级运行时
- `router-product-service` 是正式 server 产品的核心入口
- 正式 portal Tauri 宿主负责监督共享产品运行时，因此桌面模式仍能在 localhost 上暴露同一套 admin / portal 浏览器入口
- admin Tauri 宿主仅保留为显式开发壳，不属于正式发布产品
- 产品运行时支持基于 `web`、`gateway`、`admin`、`portal` 的角色切片，可分布到不同节点
- 扩展运行时支持 builtin、connector、native-dynamic 三种形态
- 配置重载、监听重绑、存储切换和 secret rotation 优先通过热重载完成，而不是强制全进程重启

## 配置与密钥边界

配置通过 `sdkwork-api-config` 加载，再注入进程与运行时句柄。

关键边界包括：

- 服务绑定地址
- 数据库后端与连接串
- admin 与 portal JWT 签名密钥
- provider 凭据主密钥
- 扩展发现路径与信任策略

密钥材料刻意与路由逻辑分离：

- 服务从配置中解析 secret-manager 策略
- 凭据通过加密后端持久化
- 历史凭据可读性通过 locator 与 key lineage 元数据保留

## 持久化模型

共享 admin store 是以下数据的系统事实来源：

- tenants 与 projects
- gateway API keys
- channels、providers、credentials 与 models
- routing policies 与 decision logs
- usage records、billing ledger 与 quota policies
- extension 安装与 rollout 状态

当前文档化的后端包括：

- SQLite
- PostgreSQL

## 前端架构

前端产品刻意拆分为两个独立应用：

- `apps/sdkwork-router-admin/` 负责超级管理后台、admin API 客户端与显式开发用 Tauri 宿主
- `apps/sdkwork-router-portal/` 负责用户自助工作台、portal API 客户端与正式 desktop 宿主
- `crates/sdkwork-api-runtime-host` 负责 server 与 desktop 共用的 Pingora 交付边界
- `crates/sdkwork-api-product-runtime` 负责启动本地 API 监听器并发布统一产品 Web 表面

这样既隔离了运维控制平面与用户自助门户，也保留了统一的对外交付契约。

## 运维架构

所有独立服务都暴露：

- health 端点
- Prometheus 风格 metrics
- 结构化请求追踪

一体化产品宿主额外负责：

- 统一的 `/admin/*`、`/portal/*` 与 `/api/*` 公网绑定
- 基于环境变量的角色切片
- 当 `web` 节点独立部署时的显式上游转发

admin 控制平面还负责：

- extension runtime reload 与 rollout
- standalone config rollout
- routing simulation
- health snapshot 查看
- usage 与 billing 可视化

## 相关文档

- 模块职责映射：
  - [功能模块](/zh/architecture/functional-modules)
- 接口总览：
  - [API 参考总览](/zh/api-reference/overview)
- 配置与观测：
  - [配置说明](/zh/operations/configuration)
  - [健康检查与 Metrics](/zh/operations/health-and-metrics)
