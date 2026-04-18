---
layout: home

hero:
  name: SDKWork API Server
  text: OpenAI 兼容网关、管理控制平面、公共门户和扩展运行时
  tagline: 一个面向跨平台部署的 Rust 工作区，用于承载 OpenAI 风格 API、在多上游之间做流量路由、运行控制平面，并交付浏览器或桌面产品体验。
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
    details: 暴露完整的 `/v1/*` 能力面，覆盖 chat、responses、embeddings、files、uploads、audio、images、music、assistants、threads、vector stores、evals、videos 等接口。
  - title: 原生控制平面
    details: 通过 `/admin/*` 统一管理 channels、providers、credentials、routing policy、runtime rollout、usage、billing 与 quota。
  - title: 公共自助门户
    details: 通过 `/portal/*` 和独立 portal 应用，为终端用户提供注册、登录、工作区查看、用量与计费趋势以及网关 API key 自助签发。
  - title: 可插拔运行时
    details: 支持 builtin、connector、native-dynamic 三类 provider runtime，并提供热重载、健康快照和 rollout 感知监督。
---

## 文档地图

SDKWork 现在采用更接近成熟 API 平台的文档结构：

- [开始使用](/zh/getting-started/installation)：安装依赖、初始化仓库，并完成第一轮构建验证
- [源码运行](/zh/getting-started/source-development)：理解 browser、preview、portal desktop 等开发态入口
- [脚本生命周期](/zh/getting-started/script-lifecycle)：理解 build、install、start、verify、stop 与 service registration 的完整关系
- [发布构建](/zh/getting-started/release-builds)：查看正式发布物。正式产品只有 `sdkwork-api-router-product-server` 和 `sdkwork-router-portal-desktop`
- [运维](/zh/operations/configuration)：配置、观测与独立部署排障
- [参考](/zh/reference/api-compatibility)：查看兼容标签、仓库结构和构建工具链

## 从这里开始

按当前目标选择入口：

- 首次跑通本地环境：
  - [快速开始](/zh/getting-started/quickstart)
- 首次安装依赖：
  - [安装准备](/zh/getting-started/installation)
- 日常本地开发：
  - [源码运行](/zh/getting-started/source-development)
- 编译与打包：
  - [编译与打包](/zh/getting-started/build-and-packaging)
- 生成正式发布物：
  - [发布构建](/zh/getting-started/release-builds)
- 线上部署：
  - [生产部署](/zh/getting-started/production-deployment)
- 理解系统设计：
  - [软件架构](/zh/architecture/software-architecture)
- 查看接口目录：
  - [网关 API](/zh/api-reference/gateway-api)

## 产品运行面

| 运行面 | 基础路径 | 用途 |
|---|---|---|
| gateway-service | `/v1/*` | OpenAI 兼容数据面 |
| admin-api-service | `/admin/*` | 运维控制平面 |
| portal-api-service | `/portal/*` | 自助认证、工作区与 API key 生命周期 |
| router-web-service | `/admin/*`、`/portal/*`、`/api/*` | Pingora 公共站点交付与 API 代理入口 |
| apps/sdkwork-router-admin | 浏览器或 Tauri | 独立超管体验 |
| apps/sdkwork-router-portal | 浏览器或 Tauri | 独立开发者自助门户与正式桌面壳 |
| docs | `/` | VitePress 文档站 |

## 常用本地端口

托管脚本默认端口：

| 运行面 | 默认绑定 |
|---|---|
| gateway | `127.0.0.1:9980` |
| admin | `127.0.0.1:9981` |
| portal | `127.0.0.1:9982` |
| web host | `127.0.0.1:9983` |
| admin Web 应用 | `127.0.0.1:5173` |
| portal Web 应用 | `127.0.0.1:5174` |

如果直接裸跑服务二进制，在未覆盖端口时仍使用 `8080`、`8081`、`8082`。

## 常用快捷路径

启动托管开发栈：

```bash
./bin/start-dev.sh
```

从源码以前台模式启动完整工作区：

```bash
node scripts/dev/start-workspace.mjs
```

以 portal desktop 和统一 Web Host 方式启动：

```bash
node scripts/dev/start-workspace.mjs --tauri
```

编译独立 release 服务二进制：

```bash
cargo build --release -p admin-api-service -p gateway-service -p portal-api-service -p router-web-service
```

构建正式 server 产品输入：

```bash
./bin/build.sh
```

构建 portal desktop 安装包：

```bash
node scripts/prepare-router-portal-desktop-runtime.mjs
pnpm --dir apps/sdkwork-router-portal tauri:build
```

构建文档站：

```bash
pnpm --dir docs build
```
