# 仓库结构

## 顶层结构

```text
.
|-- crates/
|-- services/
|-- apps/
|-- console/
|-- docs/
|-- scripts/
|-- Cargo.toml
|-- README.md
`-- README.zh-CN.md
```

## 运行面

| 路径 | 职责 |
|---|---|
| `services/gateway-service` | 独立 `/v1/*` 网关二进制 |
| `services/admin-api-service` | 独立 `/admin/*` 控制平面二进制 |
| `services/portal-api-service` | 独立 `/portal/*` 自助门户二进制 |
| `console/` | 浏览器与 Tauri admin 控制台及 landing 外壳 |
| `apps/sdkwork-router-portal/` | 独立浏览器 portal 应用 |
| `docs/` | VitePress 文档站 |

## 后端分层

| 分层 | 路径 | 职责 |
|---|---|---|
| interface | `crates/sdkwork-api-interface-*` | HTTP 路由、请求映射、鉴权边界 |
| app | `crates/sdkwork-api-app-*` | 编排、工作流和服务级决策 |
| domain | `crates/sdkwork-api-domain-*` | 领域模型、策略规则、不变量 |
| storage | `crates/sdkwork-api-storage-*` | 持久化契约与具体后端 |
| contracts | `crates/sdkwork-api-contract-*` | API 结构、兼容契约、共享请求响应类型 |
| provider | `crates/sdkwork-api-provider-*` | 上游适配器与 provider 特定执行 |
| runtime | `crates/sdkwork-api-app-runtime`、`crates/sdkwork-api-runtime-host`、`crates/sdkwork-api-extension-*` | 运行时加载、监督、扩展 ABI、嵌入宿主 |
| cross-cutting | `crates/sdkwork-api-config`、`crates/sdkwork-api-observability`、`crates/sdkwork-api-kernel` | 配置、可观测性和运行时胶水层 |

## 独立服务

- `services/gateway-service`
- `services/admin-api-service`
- `services/portal-api-service`

## 前端分层

| 路径 | 职责 |
|---|---|
| `console/src/` | 顶层 admin 外壳与 landing 组合 |
| `console/packages/` | admin 控制平面模块与共享 SDK |
| `console/src-tauri/` | Tauri 宿主和桌面打包集成 |
| `apps/sdkwork-router-portal/src/` | 独立 portal 根外壳与主题 |
| `apps/sdkwork-router-portal/packages/` | portal 基础包与业务模块 |

## 文档与运维资产

- `docs/`
  - VitePress 文档站和深度技术参考
- `docs/plans/`
  - 历史设计与实施记录
- `scripts/dev/`
  - 跨平台启动辅助脚本

## 常见定位规则

- HTTP 路由改动通常从 `crates/sdkwork-api-interface-*` 开始
- 路由、计费、provider、执行编排通常继续落到 `crates/sdkwork-api-app-*`
- 策略规则应归入 `crates/sdkwork-api-domain-*`
- 存储与迁移改动应归入 `crates/sdkwork-api-storage-*`
- 文档和运维说明统一归入 `docs/`
