# 源码运行

本页说明 Windows、Linux、macOS 上推荐的源码启动方式。

对大多数贡献者和评估者来说，这是安装完成后的主要入口。

## 默认端口

| 运行面 | 默认绑定地址 | 用途 |
|---|---|---|
| gateway | `127.0.0.1:8080` | OpenAI 兼容 `/v1/*` 流量 |
| admin | `127.0.0.1:8081` | 运维控制平面 |
| portal | `127.0.0.1:8082` | 公共认证、dashboard、用量、计费与 API key 生命周期 |
| console | `127.0.0.1:5173` | landing 页与浏览器 / Tauri admin 开发服务 |
| portal Web 应用 | `127.0.0.1:5174` | 独立浏览器 portal 开发服务 |

## 本地配置根目录

独立服务默认从本地 SDKWork 配置根目录读取配置：

- Linux / macOS：`~/.sdkwork/router/`
- Windows：`%USERPROFILE%\\.sdkwork\\router\\`

即使目录为空，服务也会使用内置默认值启动。

## 最快的端到端启动方式

### Windows

浏览器模式：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1
```

桌面模式：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 -Tauri
```

### Linux 或 macOS

浏览器模式：

```bash
node scripts/dev/start-workspace.mjs
```

桌面模式：

```bash
node scripts/dev/start-workspace.mjs --tauri
```

桌面模式下，admin 控制台会继续在 Tauri 中运行，portal 仍以浏览器应用方式启动。

启动完成后，最常用的本地地址是：

- gateway：`http://127.0.0.1:8080`
- admin：`http://127.0.0.1:8081/admin/health`
- portal：`http://127.0.0.1:8082/portal/health`
- 入口页：`http://127.0.0.1:5173/`
- admin 应用：`http://127.0.0.1:5173/admin/`
- portal 应用：`http://127.0.0.1:5174/`

## 分面启动

仅后端：

```bash
node scripts/dev/start-stack.mjs
```

仅 admin 控制台：

```bash
node scripts/dev/start-console.mjs
```

仅桌面控制台：

```bash
node scripts/dev/start-console.mjs --tauri
```

仅 portal 应用：

```bash
node scripts/dev/start-portal.mjs
```

Windows 下也提供 PowerShell 包装脚本：

- `scripts/dev/start-servers.ps1`
- `scripts/dev/start-console.ps1`
- `scripts/dev/start-workspace.ps1`

## SQLite 开发

SQLite 是默认的本地数据库。

当你不传 `--database-url` 而直接使用辅助脚本启动时，服务会使用本地配置根目录下的默认数据库：

- Linux / macOS：`~/.sdkwork/router/sdkwork-api-server.db`
- Windows：`%USERPROFILE%\\.sdkwork\\router\\sdkwork-api-server.db`

默认启动时会自动创建数据库并执行迁移。

## PostgreSQL 开发

```bash
node scripts/dev/start-workspace.mjs \
  --database-url "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 `
  -DatabaseUrl "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
```

## 原生命令

如果你希望绕过辅助脚本，按运行面分别启动，可以使用下面的命令。

```bash
cargo run -p admin-api-service
```

```bash
cargo run -p gateway-service
```

```bash
cargo run -p portal-api-service
```

如果你希望显式覆盖本地默认数据库，也可以手动设置：

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
```

运行 admin 控制台：

```bash
pnpm --dir console dev
```

运行 Tauri admin 壳：

```bash
pnpm --dir console tauri:dev
```

运行独立 portal 应用：

```bash
pnpm --dir apps/sdkwork-router-portal dev
```

## 推荐校验

启动前后都可以执行以下标准校验：

```bash
cargo fmt --all --check
cargo test --workspace -q -j 1
pnpm --dir console -r typecheck
pnpm --dir console build
pnpm --dir apps/sdkwork-router-portal typecheck
pnpm --dir apps/sdkwork-router-portal build
pnpm --dir docs typecheck
pnpm --dir docs build
```

## 下一步

- 编译产物：
  - [编译与打包](/zh/getting-started/build-and-packaging)
- 面向部署的发布二进制：
  - [发布构建](/zh/getting-started/release-builds)
- 进一步理解系统设计：
  - [软件架构](/zh/architecture/software-architecture)
