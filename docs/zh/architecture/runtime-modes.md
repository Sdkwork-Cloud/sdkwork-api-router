# 运行模式详解

本页从更实现的视角说明 SDKWork 当前支持的运行形态、桌面和服务端边界，以及它们与正式产品标准的关系。

## Server 模式

Server 模式是正式线上部署形态。

特征：

- 服务以独立二进制或一体化 `router-product-service` 方式运行
- gateway、admin、portal API 通过 HTTP 暴露
- PostgreSQL 是正式部署的默认数据库标准
- 上游凭据通常通过服务端 secret backend 管理
- `sdkwork-api-router-product-server` 是正式 server 产品

## Desktop 模式

Desktop 模式是面向单机桌面用户的产品形态。

特征：

- 正式桌面壳位于 `apps/sdkwork-router-portal/src-tauri/`
- 桌面壳负责监督一份随包分发的 `router-product-service` sidecar
- 默认本地入口是 `http://127.0.0.1:3001`
- 可以在设置中心切换为仅本机或共享网络访问
- SQLite 更适合作为本地桌面默认持久化策略
- `sdkwork-router-portal-desktop` 是正式 desktop 产品
- admin Tauri 仅保留为显式开发路径，不属于正式发布物

## 当前实现状态

当前仓库已经包含：

- 独立的 gateway、admin、portal 服务二进制
- 服务端模式的一体化 `router-product-service`
- 独立的 admin 浏览器应用与显式开发用 Tauri 宿主
- 独立的 portal 浏览器应用与正式 desktop 宿主
- 统一的 Pingora Web 交付边界
- 基于 SQLite 与 PostgreSQL 的共享控制平面存储
- OpenAI 兼容网关、portal 自助工作台、admin 控制平面
- provider runtime 的 builtin、connector、native-dynamic 三类扩展形态

## 源码开发入口

常用开发入口：

- `node scripts/dev/start-workspace.mjs`
  - 浏览器模式
- `node scripts/dev/start-workspace.mjs --preview`
  - 统一 Pingora Host 模式
- `node scripts/dev/start-workspace.mjs --tauri`
  - portal desktop 优先模式，同时保留统一浏览器入口
- `node scripts/dev/start-admin.mjs --tauri`
  - 仅在显式开发 admin 自有桌面壳时使用

## 正式产品边界

正式产品标准只包括：

- `sdkwork-api-router-product-server`
- `sdkwork-router-portal-desktop`

以下内容属于工程输入或开发路径，而不是正式发布物：

- 独立的 admin Tauri 安装包
- 单独的 web 资产压缩包
- 历史 `console` 入口

## 相关文档

- 快速理解运行形态：
  - [运行模式](/zh/getting-started/runtime-modes)
- 系统边界：
  - [软件架构](/zh/architecture/software-architecture)
- 配置与运维：
  - [配置说明](/zh/operations/configuration)
  - [健康检查与 Metrics](/zh/operations/health-and-metrics)
