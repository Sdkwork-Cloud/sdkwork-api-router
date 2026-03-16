---
layout: home

hero:
  name: SDKWork API Server
  text: OpenAI 兼容网关、管理控制平面、公开门户与扩展运行时
  tagline: 一个面向跨平台部署的 Rust 工作区，用于承载 OpenAI 风格 API、在多上游之间做流量路由、运行控制平面，并交付浏览器或桌面运营端体验。
  actions:
    - theme: brand
      text: 快速开始
      link: /zh/getting-started/quickstart
    - theme: alt
      text: API 参考
      link: /zh/api-reference/overview
    - theme: alt
      text: 软件架构
      link: /zh/architecture/software-architecture

features:
  - title: OpenAI 兼容网关
    details: 暴露覆盖广泛的 `/v1/*` 接口面，包含 chat、responses、embeddings、files、uploads、audio、images、assistants、threads、vector stores、evals、videos 等能力。
  - title: 原生管理控制平面
    details: 通过 `/admin/*` 统一管理 channels、providers、credentials、routing policy、运行时 rollout、usage、billing 与 quota。
  - title: 公开自助门户
    details: 通过 `/portal/*` 与独立 portal 应用，为终端用户提供注册、登录、工作区查看、用量与计费态势洞察，以及网关 API key 自助签发。
  - title: 可插拔运行时
    details: 支持 builtin、connector、native-dynamic 三类 provider runtime，并围绕热重载、健康快照与 rollout 感知监督运行。
---

## 文档地图

SDKWork 现在采用更接近成熟 API 平台的文档结构：

- [开始使用](/zh/getting-started/installation)：安装依赖、从源码启动、编译二进制，以及构建浏览器或 Tauri 产物
- [架构](/zh/architecture/software-architecture)：理解独立服务、工作区分层、扩展运行时和模块边界
- [API 参考](/zh/api-reference/overview)：按网关、管理端、门户三个接口面查看基地址、鉴权边界和路由家族
- [运维](/zh/operations/configuration)：查看配置、可观测性与常见排障入口
- [参考](/zh/reference/api-compatibility)：查看兼容真值标签、仓库结构和构建工具链

## 从这里开始

按当前目标选择入口：

- 首次跑通本地栈：
  - [快速开始](/zh/getting-started/quickstart)
- 首次搭建环境：
  - [安装准备](/zh/getting-started/installation)
- 本地联调与开发：
  - [源码运行](/zh/getting-started/source-development)
- 编译与打包：
  - [编译与打包](/zh/getting-started/build-and-packaging)
- 发布产物与部署：
  - [发布构建](/zh/getting-started/release-builds)
- 理解系统设计：
  - [软件架构](/zh/architecture/software-architecture)
- 查看接口清单：
  - [网关 API](/zh/api-reference/gateway-api)

## 产品运行面

| 运行面 | 基础路径 | 作用 |
|---|---|---|
| gateway-service | `/v1/*` | OpenAI 兼容数据平面 |
| admin-api-service | `/admin/*` | 运维与治理控制平面 |
| portal-api-service | `/portal/*` | 终端用户自助认证、工作区与 API key 生命周期 |
| docs | `/` | VitePress 文档站 |
| console | 浏览器或 Tauri | operator 控制台与 landing 外壳 |
| apps/sdkwork-router-portal | 浏览器 | 独立开发者自助门户 |

## 本地默认端口

| 运行面 | 默认绑定 |
|---|---|
| gateway | `127.0.0.1:8080` |
| admin | `127.0.0.1:8081` |
| portal | `127.0.0.1:8082` |
| console 开发服务 | `127.0.0.1:5173` |
| portal Web 应用 | `127.0.0.1:5174` |

## 快速路径

从源码启动完整栈：

```bash
node scripts/dev/start-workspace.mjs
```

从源码启动完整栈并拉起桌面壳：

```bash
node scripts/dev/start-workspace.mjs --tauri
```

编译三套独立服务的 release 二进制：

```bash
cargo build --release -p admin-api-service -p gateway-service -p portal-api-service
```

构建 operator 控制台：

```bash
pnpm --dir console build
```

构建独立 portal 应用：

```bash
pnpm --dir apps/sdkwork-router-portal build
```

构建文档站点：

```bash
pnpm --dir docs build
```
