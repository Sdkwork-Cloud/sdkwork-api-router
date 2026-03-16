# sdkwork-api-server

[English Guide](./README.md)

SDKWork API Server 是一个基于 Axum 的 OpenAI 兼容网关、控制平面、公开门户和扩展运行时系统，技术栈包括 Rust、React、pnpm 和 Tauri。

## 当前仓库包含什么

运行面：

- `gateway-service`
  - OpenAI 兼容的 `/v1/*` 网关
- `admin-api-service`
  - 面向 operator 的 `/admin/*` 控制平面
- `portal-api-service`
  - 面向外部用户的 `/portal/*` 自助 API
- `router-web-service`
  - 基于 Pingora 的公开静态站点与 API 代理入口
- `apps/sdkwork-router-admin/`
  - 独立的超级管理后台浏览器工程与 admin 自有 Tauri 桌面宿主
- `apps/sdkwork-router-portal/`
  - 独立的开发者自助门户应用
- `docs/`
  - 使用 VitePress 构建的中英文运维与使用文档站点

当前基础能力：

- 基于 Axum 的 Rust 服务
- SQLite 与 PostgreSQL 存储
- 基于 Pingora 的 admin / portal 对外站点交付
- 独立浏览器 admin / portal 应用
- admin 自有 Tauri 桌面宿主
- builtin、connector、native-dynamic 扩展运行时
- public portal 注册、登录、dashboard、用量、计费态势与 API key 签发
- 基于 `~/.sdkwork/router/` 的本地 JSON/YAML 配置体系

## 支持平台

- Windows
- Linux
- macOS

## 快速开始

Standalone 服务现在支持本地配置目录和内置默认值。

默认配置根目录：

- Linux / macOS：`~/.sdkwork/router/`
- Windows：`%USERPROFILE%\\.sdkwork\\router\\`

配置文件查找顺序：

1. `config.yaml`
2. `config.yml`
3. `config.json`

配置优先级：

1. 内置本地默认值
2. 本地配置文件
3. `SDKWORK_*` 环境变量

即使没有配置文件，系统也会使用本地默认值启动：

- gateway 监听：`127.0.0.1:8080`
- admin 监听：`127.0.0.1:8081`
- portal 监听：`127.0.0.1:8082`
- SQLite 数据库：`~/.sdkwork/router/sdkwork-api-server.db`
- 扩展目录：`~/.sdkwork/router/extensions`
- 本地密钥文件：`~/.sdkwork/router/secrets.json`

示例 `config.yaml`：

```yaml
gateway_bind: "127.0.0.1:8080"
admin_bind: "127.0.0.1:8081"
portal_bind: "127.0.0.1:8082"
database_url: "sqlite://sdkwork-api-server.db"
secret_backend: "local_encrypted_file"
secret_local_file: "secrets.json"
extension_paths:
  - "extensions"
enable_connector_extensions: true
enable_native_dynamic_extensions: false
```

配置文件中的相对路径会按配置文件所在目录解析。

如需覆盖默认位置，可使用：

- `SDKWORK_CONFIG_DIR`
- `SDKWORK_CONFIG_FILE`

## 文档站点

本地安装并预览 `docs`：

```bash
pnpm --dir docs install
pnpm --dir docs dev
```

构建 `docs`：

```bash
pnpm --dir docs build
```

核心英文文档结构：

- Getting Started:
  - [Quickstart](./docs/getting-started/quickstart.md)
  - [Installation](./docs/getting-started/installation.md)
  - [Source Development](./docs/getting-started/source-development.md)
  - [Build and Packaging](./docs/getting-started/build-and-packaging.md)
  - [Release Builds](./docs/getting-started/release-builds.md)
- Architecture:
  - [Software Architecture](./docs/architecture/software-architecture.md)
  - [Functional Modules](./docs/architecture/functional-modules.md)
  - [Runtime Modes Deep Dive](./docs/architecture/runtime-modes.md)
- API Reference:
  - [Overview](./docs/api-reference/overview.md)
  - [Gateway API](./docs/api-reference/gateway-api.md)
  - [Admin API](./docs/api-reference/admin-api.md)
  - [Portal API](./docs/api-reference/portal-api.md)
- Operations:
  - [Configuration](./docs/operations/configuration.md)
  - [Health and Metrics](./docs/operations/health-and-metrics.md)
- Reference:
  - [API Compatibility](./docs/reference/api-compatibility.md)
  - [Repository Layout](./docs/reference/repository-layout.md)
  - [Build and Tooling](./docs/reference/build-and-tooling.md)

核心中文文档结构：

- 开始使用：
  - [快速开始](./docs/zh/getting-started/quickstart.md)
  - [安装准备](./docs/zh/getting-started/installation.md)
  - [源码运行](./docs/zh/getting-started/source-development.md)
  - [编译与打包](./docs/zh/getting-started/build-and-packaging.md)
  - [发布构建](./docs/zh/getting-started/release-builds.md)
- 架构：
  - [软件架构](./docs/zh/architecture/software-architecture.md)
  - [功能模块](./docs/zh/architecture/functional-modules.md)
  - [运行模式详解](./docs/zh/architecture/runtime-modes.md)
- API 参考：
  - [总览](./docs/zh/api-reference/overview.md)
  - [网关 API](./docs/zh/api-reference/gateway-api.md)
  - [管理端 API](./docs/zh/api-reference/admin-api.md)
  - [门户 API](./docs/zh/api-reference/portal-api.md)
- 运维：
  - [配置说明](./docs/zh/operations/configuration.md)
  - [健康检查与 Metrics](./docs/zh/operations/health-and-metrics.md)
- 参考：
  - [API 兼容矩阵](./docs/zh/reference/api-compatibility.md)
  - [仓库结构](./docs/zh/reference/repository-layout.md)
  - [构建与工具链](./docs/zh/reference/build-and-tooling.md)

## 环境要求

必需：

- Rust stable 与 Cargo
- Node.js 20+
- pnpm 10+

可选：

- PostgreSQL 15+
- Tauri CLI：

```bash
cargo install tauri-cli
```

## 源码启动

推荐的完整栈启动方式：

| 工作流 | Windows | Linux / macOS |
|---|---|---|
| 浏览器模式 | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1` | `node scripts/dev/start-workspace.mjs` |
| 桌面模式 | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 -Tauri` | `node scripts/dev/start-workspace.mjs --tauri` |
| dry run | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 -DryRun` | `node scripts/dev/start-workspace.mjs --dry-run` |

打开：

- 浏览器模式 admin：`http://127.0.0.1:5173/admin/`
- 浏览器模式 portal：`http://127.0.0.1:5174/portal/`
- 预览或桌面模式对外 web host：`http://127.0.0.1:3001/portal/`
- 预览或桌面模式 admin 站点：`http://127.0.0.1:3001/admin/`

说明：

- `start-workspace --tauri` 会启动 admin 桌面壳，同时拉起共享的 Pingora web host，对外暴露 admin 与 portal。
- `start-workspace --preview` 会先构建 admin / portal，再通过 Pingora 统一提供静态资源访问。

如果要指定配置根目录启动：

Windows：

```powershell
$env:SDKWORK_CONFIG_DIR="$HOME\\.sdkwork\\router"
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1
```

Linux / macOS：

```bash
export SDKWORK_CONFIG_DIR="$HOME/.sdkwork/router"
node scripts/dev/start-workspace.mjs
```

更底层的源码辅助脚本：

- 仅后端：
  - `scripts/dev/start-servers.ps1`
  - `node scripts/dev/start-stack.mjs`
- 仅 admin：
  - `node scripts/dev/start-admin.mjs`
- 仅 portal：
  - `node scripts/dev/start-portal.mjs`
- 仅公开 web host：
  - `node scripts/dev/start-web.mjs`

详细说明见：

- [源码运行](./docs/zh/getting-started/source-development.md)

## Release 构建与启动

构建 release 服务二进制：

```bash
cargo build --release -p admin-api-service -p gateway-service -p portal-api-service -p router-web-service
```

构建 admin 前端资源：

```bash
pnpm --dir apps/sdkwork-router-admin install
pnpm --dir apps/sdkwork-router-admin build
```

构建独立 portal 前端资源：

```bash
pnpm --dir apps/sdkwork-router-portal install
pnpm --dir apps/sdkwork-router-portal build
```

构建 Tauri 桌面包：

```bash
pnpm --dir apps/sdkwork-router-admin tauri:build
```

使用默认本地配置根目录启动 release 二进制：

Windows：

```powershell
New-Item -ItemType Directory -Force "$HOME\\.sdkwork\\router" | Out-Null
.\target\release\admin-api-service.exe
.\target\release\gateway-service.exe
.\target\release\portal-api-service.exe
```

Linux / macOS：

```bash
mkdir -p "$HOME/.sdkwork/router"
./target/release/admin-api-service
./target/release/gateway-service
./target/release/portal-api-service
```

指定显式配置文件启动：

Windows：

```powershell
$env:SDKWORK_CONFIG_FILE="$HOME\\.sdkwork\\router\\config.yaml"
.\target\release\gateway-service.exe
```

Linux / macOS：

```bash
export SDKWORK_CONFIG_FILE="$HOME/.sdkwork/router/config.yaml"
./target/release/gateway-service
```

像 `SDKWORK_DATABASE_URL` 这样的环境变量依然会覆盖配置文件中的值。

完整 release 说明见：

- [Release 构建](./docs/zh/getting-started/release-builds.md)

## 运行与运维

关键端点：

- gateway health: `http://127.0.0.1:8080/health`
- admin health: `http://127.0.0.1:8081/admin/health`
- portal health: `http://127.0.0.1:8082/portal/health`
- gateway metrics: `http://127.0.0.1:8080/metrics`
- admin metrics: `http://127.0.0.1:8081/metrics`
- portal metrics: `http://127.0.0.1:8082/metrics`

关键环境变量：

- `SDKWORK_CONFIG_DIR`
- `SDKWORK_CONFIG_FILE`
- `SDKWORK_DATABASE_URL`
- `SDKWORK_GATEWAY_BIND`
- `SDKWORK_ADMIN_BIND`
- `SDKWORK_PORTAL_BIND`
- `SDKWORK_ADMIN_JWT_SIGNING_SECRET`
- `SDKWORK_PORTAL_JWT_SIGNING_SECRET`
- `SDKWORK_SECRET_BACKEND`
- `SDKWORK_SECRET_LOCAL_FILE`
- `SDKWORK_EXTENSION_PATHS`

详细运维文档：

- [配置说明](./docs/zh/operations/configuration.md)
- [健康检查与 Metrics](./docs/zh/operations/health-and-metrics.md)
- [运行模式](./docs/zh/getting-started/runtime-modes.md)
- [软件架构](./docs/zh/architecture/software-architecture.md)
- [API 参考总览](./docs/zh/api-reference/overview.md)

## 附加技术参考

- [完整兼容矩阵](./docs/api/compatibility-matrix.md)
- [运行模式详解](./docs/zh/architecture/runtime-modes.md)

## 验证

当前验证基线：

```bash
pnpm --dir docs typecheck
pnpm --dir docs build
cargo fmt --all --check
cargo test --workspace -q -j 1
pnpm --dir apps/sdkwork-router-admin typecheck
pnpm --dir apps/sdkwork-router-admin build
pnpm --dir apps/sdkwork-router-portal typecheck
pnpm --dir apps/sdkwork-router-portal build
```
