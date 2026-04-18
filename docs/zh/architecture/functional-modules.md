# 功能模块

本页把用户可感知能力映射到实际工作区模块，便于快速定位实现落点。

## 运行面模块

| 模块 | 作用 | 关键路径 |
|---|---|---|
| gateway | 暴露 OpenAI 兼容 `/v1/*` 接口面 | `services/gateway-service`、`crates/sdkwork-api-interface-http` |
| admin control plane | 管理 tenants、projects、credentials、routing、billing 与 runtime operations | `services/admin-api-service`、`crates/sdkwork-api-interface-admin` |
| public portal | 支持注册、登录、工作区查看与 API key 自助签发 | `services/portal-api-service`、`crates/sdkwork-api-interface-portal` |
| admin app | 独立超管浏览器应用与显式开发用 Tauri 宿主 | `apps/sdkwork-router-admin/` |
| portal app | 独立自助浏览器应用与正式 desktop 宿主 | `apps/sdkwork-router-portal/` |
| docs | operator 与贡献者文档 | `docs/` |

## 核心后端模块

| 能力 | 职责 | 关键 crate |
|---|---|---|
| 网关编排 | 把 API 路由映射到 provider 执行与本地回退 | `sdkwork-api-app-gateway` |
| 路由 | 策略模拟、决策日志、健康感知分发 | `sdkwork-api-app-routing`、`sdkwork-api-domain-routing` |
| 计费与用量 | usage records、billing summary、quota enforcement | `sdkwork-api-app-usage`、`sdkwork-api-app-billing`、`sdkwork-api-domain-usage`、`sdkwork-api-domain-billing` |
| 身份体系 | admin JWT、portal JWT、gateway API key | `sdkwork-api-app-identity`、`sdkwork-api-domain-identity` |
| 租户与目录 | tenants、projects、models、channels、providers | `sdkwork-api-app-tenant`、`sdkwork-api-app-catalog`、`sdkwork-api-domain-tenant`、`sdkwork-api-domain-catalog` |
| 凭据管理 | 加密 provider credential 存储与 secret 解析 | `sdkwork-api-app-credential`、`sdkwork-api-secret-*`、`sdkwork-api-domain-credential` |
| 运行时与扩展加载 | 扩展发现、ABI 边界、runtime supervision 与宿主装配 | `sdkwork-api-app-runtime`、`sdkwork-api-extension-host`、`sdkwork-api-runtime-host`、`sdkwork-api-extension-*` |

## Provider 与协议模块

| 模块 | 职责 |
|---|---|
| `sdkwork-api-provider-openai` | OpenAI 兼容上游转发 |
| `sdkwork-api-provider-openrouter` | OpenRouter 兼容转发 |
| `sdkwork-api-provider-ollama` | 本地 Ollama 兼容转发 |
| `sdkwork-api-provider-core` | 共享 provider 请求与流式抽象 |
| `sdkwork-api-contract-openai` | OpenAI 兼容请求与响应结构 |
| `sdkwork-api-contract-gateway` | SDKWork 自有契约类型 |

## 存储模块

| 模块 | 职责 |
|---|---|
| `sdkwork-api-storage-core` | 共享 admin-store 契约 |
| `sdkwork-api-storage-sqlite` | 基于 SQLite 的 admin store 与迁移 |
| `sdkwork-api-storage-postgres` | 基于 PostgreSQL 的 admin store |
| `sdkwork-api-storage-libsql` | libSQL 后端工作 |
| `sdkwork-api-storage-mysql` | MySQL 后端工作 |

## 开发与运维工具

| 模块 | 职责 |
|---|---|
| `scripts/dev/start-workspace.*` | 一条命令启动本地工作区 |
| `scripts/dev/start-stack.mjs` | 仅启动后端 |
| `scripts/dev/start-admin.mjs` | 仅启动 admin |
| `scripts/dev/start-portal.mjs` | 仅启动 portal |
| `docs/` | 文档站与历史方案记录 |
| `apps/sdkwork-router-portal/src-tauri/` | 正式 desktop 打包与 sidecar 宿主集成 |
| `apps/sdkwork-router-admin/src-tauri/` | 显式开发用 admin 桌面宿主 |

## 模块如何协作

处理变更时可以使用以下经验法则：

- HTTP 行为起点通常在 interface crate
- 工作流与编排逻辑应放在 app crate
- 策略与规则逻辑应归入 domain crate
- 持久化和迁移逻辑应归入 storage crate
- 上游执行与协议适配应放在 provider 或 extension runtime crate

这样可以避免把产品行为埋进传输层或存储层，保持工作区可扩展。

## 相关文档

- 系统级设计：
  - [软件架构](/zh/architecture/software-architecture)
- 工作区结构：
  - [仓库结构](/zh/reference/repository-layout)
