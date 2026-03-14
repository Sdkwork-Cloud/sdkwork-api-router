# sdkwork-api-server

[English Guide](./README.md)

SDKWork API Server 是一个基于 Axum 的 OpenAI 兼容网关、控制平面、扩展运行时和公共自助门户系统，技术栈包括 Rust、React、pnpm 和 Tauri。

当前仓库围绕 4 个运行面组织：

- `gateway-service`
  - OpenAI 兼容 `/v1/*` 网关
- `admin-api-service`
  - 面向运营管理的 `/admin/*` 控制平面
- `portal-api-service`
  - 面向外部用户的 `/portal/*` 注册、登录、工作区查看与 API key 签发
- `console/`
  - 可在浏览器访问，也可运行在 Tauri 桌面宿主中的 React 前端

## 当前已实现能力

已经落地的内容：

- OpenAI 兼容网关，当前 `/v1/*` 覆盖范围见 [docs/api/compatibility-matrix.md](./docs/api/compatibility-matrix.md)
- stateful 与 stateless 两种执行路径
- admin API，覆盖 tenants、projects、API keys、channels、proxy providers、credentials、models、routing、usage、billing、extensions
- 公共 portal API：
  - `POST /portal/auth/register`
  - `POST /portal/auth/login`
  - `GET /portal/auth/me`
  - `GET /portal/workspace`
  - `GET /portal/api-keys`
  - `POST /portal/api-keys`
- 统一存储抽象下的 SQLite 与 PostgreSQL 支持
- 加密 secret 存储后端：
  - `database_encrypted`
  - `local_encrypted_file`
  - `os_keyring`
- 扩展运行时支持：
  - `builtin`
  - `connector`
  - `native_dynamic`
- 按 package 拆分的 React 控制台模块：
  - portal SDK
  - portal auth
  - portal dashboard
  - admin workspace
  - channel 管理
  - routing
  - runtime inspection
  - usage 与 billing
- 兼容浏览器与 Tauri 的 hash 路由：
  - `#/portal/register`
  - `#/portal/login`
  - `#/portal/dashboard`
  - `#/admin`

## 支持平台

当前仓库支持以下平台：

- Windows
- Linux
- macOS

Rust 服务本身是跨平台的。React 控制台可在三种平台上的现代浏览器中运行。Tauri 桌面壳是可选能力，并与浏览器控制台共享同一套前端路由。

## 环境要求

必需：

- Rust stable 与 Cargo
- Node.js 20+
- pnpm 10+

可选：

- PostgreSQL 15+，用于 PostgreSQL 部署
- Tauri CLI，桌面开发时使用：

```bash
cargo install tauri-cli
```

## 仓库结构

```text
.
|-- crates/                      # domain、app、interface、provider、runtime、storage 等 Rust crate
|-- services/
|   |-- admin-api-service/       # 独立 admin HTTP 服务
|   |-- gateway-service/         # 独立 OpenAI 兼容网关 HTTP 服务
|   `-- portal-api-service/      # 独立 public portal HTTP 服务
|-- console/                     # React + pnpm workspace + 可选 Tauri 桌面壳
|-- scripts/
|   `-- dev/                     # 跨平台启动辅助脚本
|-- docs/                        # 架构说明、计划文档、兼容性说明
`-- README.md                    # 英文操作文档
```

## 默认端口

| 运行面 | 默认绑定地址 | 用途 |
|---|---|---|
| gateway | `127.0.0.1:8080` | OpenAI 兼容 `/v1/*` 请求 |
| admin | `127.0.0.1:8081` | 运营控制平面 |
| portal | `127.0.0.1:8082` | 公共认证、工作区与 API key 生命周期 |
| console | `127.0.0.1:5173` | 浏览器与 Tauri 共用前端开发服务 |

## 推荐启动命令

优先使用统一的 workspace 启动器。更底层的脚本保留给分面启动或调试。

| 工作流 | Windows | Linux / macOS |
|---|---|---|
| 浏览器模式启动完整栈 | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1` | `node scripts/dev/start-workspace.mjs` |
| 桌面模式启动完整栈并保留浏览器访问 | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 -Tauri` | `node scripts/dev/start-workspace.mjs --tauri` |
| 完整栈 dry run | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 -DryRun` | `node scripts/dev/start-workspace.mjs --dry-run` |
| 仅启动后端服务 | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-servers.ps1` | `node scripts/dev/start-stack.mjs` |
| 仅启动浏览器控制台 | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-console.ps1` | `node scripts/dev/start-console.mjs` |
| 仅启动 Tauri | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-console.ps1 -Tauri` | `node scripts/dev/start-console.mjs --tauri` |
| 预览生产构建 | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-console.ps1 -Preview` | `node scripts/dev/start-console.mjs --preview` |

说明：

- `start-workspace` 会一起启动后端服务与控制台
- `start-workspace --tauri` 运行时，浏览器仍然可以访问 `http://127.0.0.1:5173`
- `start-servers.ps1` 在 Windows 上仍会为后端分服务打开独立 PowerShell 窗口
- Node 启动器是 Windows、Linux、macOS 都可直接使用的跨平台入口

## 使用 SQLite 快速启动

这是当前最快的端到端本地运行方式。

### Windows

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1
```

### Linux 或 macOS

```bash
node scripts/dev/start-workspace.mjs
```

打开：

- `http://127.0.0.1:5173/#/portal/register`
- `http://127.0.0.1:5173/#/portal/login`
- `http://127.0.0.1:5173/#/portal/dashboard`
- `http://127.0.0.1:5173/#/admin`

如果你希望后端和前端分开在不同终端或不同窗口中运行，可以改用 `start-stack`、`start-servers` 和 `start-console` 这些更底层的脚本。

## 桌面模式同时支持浏览器访问

如果你希望 Tauri 桌面窗口和浏览器界面同时可用，请使用统一启动器的桌面模式。

### Windows

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 -Tauri
```

### Linux 或 macOS

```bash
node scripts/dev/start-workspace.mjs --tauri
```

原因：

- `tauri dev` 使用 Vite dev server 作为前端来源
- 同一个 Vite 地址仍然可以被浏览器直接访问
- portal 注册、登录、dashboard 与 admin 路由在浏览器和 Tauri 中完全一致

## Public Portal 自助流程

当后端服务和 console 启动后：

1. 打开 `http://127.0.0.1:5173/#/portal/register`
2. 注册一个 portal 用户
3. 注册成功后进入 `#/portal/dashboard`
4. 为 `live`、`test` 或 `staging` 创建 gateway API key
5. 立即复制返回的明文 key
6. 使用该 key 调用网关

示例：

```bash
curl http://127.0.0.1:8080/v1/models \
  -H "Authorization: Bearer skw_live_your_key_here"
```

注意：portal 的列表接口不会再次返回明文 key。明文只会在创建时返回一次。

## 使用 PostgreSQL 启动

如果要使用 PostgreSQL，只需要让完整栈指向同一个数据库 URL。

### Windows

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 `
  -DatabaseUrl "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
```

### Linux 或 macOS

```bash
node scripts/dev/start-workspace.mjs \
  --database-url "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
```

SQLite 和 PostgreSQL 的迁移都会在启动时自动执行。

## 分面启动脚本

如果你需要比统一启动器更细粒度的控制，可以使用下面这些脚本。

### 仅启动后端服务

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-servers.ps1
```

Linux 或 macOS：

```bash
node scripts/dev/start-stack.mjs
```

### 仅启动控制台

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-console.ps1
```

Linux 或 macOS：

```bash
node scripts/dev/start-console.mjs
```

### 原生命令回退方式

如果你不想使用辅助脚本，也可以直接执行原生命令。

Windows PowerShell：

```powershell
$env:SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p admin-api-service
```

```powershell
$env:SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p gateway-service
```

```powershell
$env:SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p portal-api-service
```

```powershell
pnpm --dir console install
pnpm --dir console dev
```

```powershell
pnpm --dir console tauri:dev
```

Linux 或 macOS：

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p admin-api-service
```

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p gateway-service
```

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p portal-api-service
```

```bash
pnpm --dir console install
pnpm --dir console dev
```

```bash
pnpm --dir console tauri:dev
```

## 浏览器控制台工作区

安装前端依赖：

```bash
pnpm --dir console install
```

启动浏览器开发服务器：

```bash
pnpm --dir console dev
```

检查所有 package 的类型：

```bash
pnpm --dir console -r typecheck
```

构建生产资源：

```bash
pnpm --dir console build
```

预览生产构建：

```bash
pnpm --dir console preview
```

开发服务器默认代理：

- `/admin` -> `http://127.0.0.1:8081`
- `/portal` -> `http://127.0.0.1:8082`
- `/v1` -> `http://127.0.0.1:8080`

如需覆盖代理目标，可使用：

- `SDKWORK_ADMIN_PROXY_TARGET`
- `SDKWORK_PORTAL_PROXY_TARGET`
- `SDKWORK_GATEWAY_PROXY_TARGET`

## 健康检查与 Metrics

健康检查地址：

- gateway: `http://127.0.0.1:8080/health`
- admin: `http://127.0.0.1:8081/admin/health`
- portal: `http://127.0.0.1:8082/portal/health`

Metrics 地址：

- gateway: `http://127.0.0.1:8080/metrics`
- admin: `http://127.0.0.1:8081/metrics`
- portal: `http://127.0.0.1:8082/metrics`

示例：

```bash
curl http://127.0.0.1:8082/portal/health
curl http://127.0.0.1:8082/metrics
```

## 运行时配置

重要环境变量：

- `SDKWORK_GATEWAY_BIND`
- `SDKWORK_ADMIN_BIND`
- `SDKWORK_PORTAL_BIND`
- `SDKWORK_DATABASE_URL`
- `SDKWORK_ADMIN_JWT_SIGNING_SECRET`
- `SDKWORK_PORTAL_JWT_SIGNING_SECRET`
- `SDKWORK_SECRET_BACKEND`
- `SDKWORK_CREDENTIAL_MASTER_KEY`
- `SDKWORK_SECRET_LOCAL_FILE`
- `SDKWORK_SECRET_KEYRING_SERVICE`
- `SDKWORK_RUNTIME_SNAPSHOT_INTERVAL_SECS`
- `SDKWORK_EXTENSION_PATHS`
- `SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS`
- `SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS`
- `SDKWORK_EXTENSION_TRUSTED_SIGNERS`
- `SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS`
- `SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS`

支持的 secret backend：

- `database_encrypted`
- `local_encrypted_file`
- `os_keyring`

## 扩展与包命名约定

扩展架构区分三个名字：

- runtime ID
  - `sdkwork.provider.openrouter`
  - `sdkwork.channel.openai`
- distribution package name
  - `sdkwork-provider-openrouter`
  - `sdkwork-channel-openai`
- Rust crate name
  - `sdkwork-api-ext-provider-openrouter`
  - `sdkwork-api-ext-channel-openai`

这样可以明确区分 channel 与 proxy provider，同时支持配置驱动加载、运行时发现以及未来外部分发。

## 架构摘要

后端分层：

- interface / controller crates 位于 `crates/sdkwork-api-interface-*`
- app / service crates 位于 `crates/sdkwork-api-app-*`
- repository / storage crates 位于 `crates/sdkwork-api-storage-*`

前端分层：

- 根壳层组合位于 `console/src/`
- 可复用业务包位于 `console/packages/`
- public portal 拆分为：
  - `sdkwork-api-portal-sdk`
  - `sdkwork-api-portal-auth`
  - `sdkwork-api-portal-user`

## 当前有意未实现部分

当前系统已经可以端到端使用，但以下能力仍然属于后续路线图：

- portal 多用户工作区与邀请
- 密码重置与邮件投递
- OAuth / SSO
- 独立的 MySQL 或 libsql 部署流程
- 已发现外部扩展的热重载

## 参考文档

更多运行和架构细节见：

- [docs/api/compatibility-matrix.md](./docs/api/compatibility-matrix.md)
- [docs/architecture/runtime-modes.md](./docs/architecture/runtime-modes.md)
- [docs/plans/2026-03-14-public-portal-cross-platform-design.md](./docs/plans/2026-03-14-public-portal-cross-platform-design.md)
- [docs/plans/2026-03-14-unified-workspace-launch-design.md](./docs/plans/2026-03-14-unified-workspace-launch-design.md)

## 验证命令

当前建议的最新验证基线：

```bash
node --check scripts/dev/workspace-launch-lib.mjs
node --check scripts/dev/start-workspace.mjs
node --test scripts/dev/tests/start-workspace.test.mjs
node scripts/dev/start-workspace.mjs --dry-run
cargo fmt --all --check
cargo test --workspace -q -j 1
pnpm --dir console -r typecheck
pnpm --dir console build
```
