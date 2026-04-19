# 脚本生命周期

本页说明各类启动脚本分别做什么、适合什么时候使用、会写入哪些运行时状态，以及完整的启动和停止生命周期。

建议结合以下文档一起阅读：

- [快速开始](/zh/getting-started/quickstart)
- [源码开发](/zh/getting-started/source-development)
- [发布构建](/zh/getting-started/release-builds)

## 两层脚本体系

SDKWork 提供两套脚本层，它们解决的问题不同。

为了保持仓库根目录入口稳定，根目录下的 `start.*`、`build.*`、`install.*`、`stop.*` 都只是薄包装，真正的生命周期实现仍由 `bin/*` 持有。

### `scripts/dev/*`

这是原始源码导向的启动器。

适用场景：

- 正在仓库中进行日常开发
- 需要细粒度控制具体启动哪些面
- 可以接受在当前终端中管理前台进程

特点：

- 直接从源码树运行
- 大多数情况下保持前台运行
- 适合迭代调试和小范围验证
- 不会维护带 PID 文件和日志轮转的托管运行目录

### `bin/*`

这是托管式编排脚本。

适用场景：

- 需要可预测的开发态或发布态生命周期
- 希望用一条命令准备运行目录、追踪主 PID 并打印格式化摘要
- 希望在 Windows、Linux 和 macOS 之间维持一致入口

特点：

- 自动创建并复用运行目录
- 写入 PID 文件、日志和环境文件
- 支持 dry-run、foreground 和 service-manager 托管
- 启动成功后打印统一入口、直连服务地址和 bootstrap 提示

## 脚本目录总览

| 脚本 | 范围 | 主要用途 | 运行时状态 | 如何停止 |
|---|---|---|---|---|
| `bin/build.sh` / `bin/build.ps1` | 发布态 | 构建 release 二进制、前端资源、文档和桌面产物 | `artifacts/release/` 和 Rust target 目录 | 构建结束后退出 |
| `bin/install.sh` / `bin/install.ps1` | 发布态 | 把构建好的 release 安装到产品根目录 | 默认 `artifacts/install/sdkwork-api-router/` | 安装结束后退出 |
| `bin/start.sh` / `bin/start.ps1` | 发布态 | 启动安装后的 `router-product-service` 运行时 | 产品根目录下的 `config/`、`log/`、`run/` 和 `current/release-manifest.json` | `bin/stop.sh` / `bin/stop.ps1` 或 service manager |
| `bin/stop.sh` / `bin/stop.ps1` | 发布态 | 根据 PID 停止托管发布运行时 | 产品根目录 `run/` | 进程树停止后退出 |
| `bin/start-dev.sh` / `bin/start-dev.ps1` | 托管开发态 | 启动托管开发运行时 | `artifacts/runtime/dev/` | `bin/stop-dev.sh` / `bin/stop-dev.ps1` 或前台 `Ctrl+C` |
| `bin/stop-dev.sh` / `bin/stop-dev.ps1` | 托管开发态 | 停止托管开发运行时 | `artifacts/runtime/dev/run/` PID 文件 | 进程树停止后退出 |
| `node scripts/prepare-router-portal-desktop-runtime.mjs` | 桌面打包 | 预制 portal desktop 的 sidecar 运行时载荷 | `bin/portal-rt/router-product/` | 预制完成后退出 |
| `scripts/dev/start-workspace.mjs` / `.ps1` | 原始源码开发态 | 启动后端服务加浏览器或桌面前端 | 仅源码树 | 当前终端 `Ctrl+C` |
| `scripts/dev/start-stack.mjs` / `start-servers.ps1` | 原始源码开发态 | 仅启动后端服务 | 仅源码树 | 当前终端 `Ctrl+C` |
| `scripts/dev/start-admin.mjs` | 原始源码开发态 | 启动 admin 浏览器应用或开发态 Tauri 壳 | 仅源码树 | 当前终端 `Ctrl+C` |
| `scripts/dev/start-portal.mjs` | 原始源码开发态 | 启动 portal 浏览器应用 | 仅源码树 | 当前终端 `Ctrl+C` |
| `scripts/dev/start-web.mjs` | 原始源码开发态 | 构建 admin / portal 静态资源并通过 Pingora 暴露 | 仅源码树 | 当前终端 `Ctrl+C` |

## 端口模型

仓库里有两套重要的默认端口。

### 托管脚本默认端口

托管脚本使用 `998x` 端口段，尽量避开常见本地冲突：

- gateway：`127.0.0.1:9980`
- admin：`127.0.0.1:9981`
- portal：`127.0.0.1:9982`
- 统一 Web Host：`127.0.0.1:9983`

原始 preview 绑定场景下统一 Host 可能使用 `0.0.0.0:9983`。

### 二进制内建默认端口

如果直接运行 standalone 服务二进制，且没有通过脚本层覆盖，则仍使用其内建默认值：

- gateway：`127.0.0.1:8080`
- admin：`127.0.0.1:8081`
- portal：`127.0.0.1:8082`

这意味着：

- 走 `bin/*` 工作流时，优先关注 `998x`
- 直接运行原始二进制时，优先关注 `808x`

## 开发态生命周期

推荐的开发路径是托管式源码开发。

### 1. 启动托管开发运行时

Linux 或 macOS：

```bash
./bin/start-dev.sh
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1
```

默认行为：

- 使用 `artifacts/runtime/dev/data/` 下的可写 SQLite
- 默认进入 preview 模式，使内建统一 Web Host 成为主要浏览器入口
- 等待后端和前端健康检查成功后再报告启动成功
- 打印格式化的启动摘要

### 2. 阅读启动摘要

启动成功后，脚本会打印：

- 统一浏览器入口：
  - `http://127.0.0.1:9983/admin/`
  - `http://127.0.0.1:9983/portal/`
- 统一健康检查入口：
  - `http://127.0.0.1:9983/api/v1/health`
- 后端直连地址：
  - `http://127.0.0.1:9980/health`
  - `http://127.0.0.1:9981/admin/health`
  - `http://127.0.0.1:9982/portal/health`
- 开发身份 bootstrap 提示：
  - 身份来自当前激活的 bootstrap profile
  - 在共享本地环境前先检查 `data/identities/dev.json`

### 3. 可选：切回独立浏览器 dev server

如果你明确需要 Vite 的独立 admin / portal dev server，而不是统一 Pingora Host：

Linux 或 macOS：

```bash
./bin/start-dev.sh --browser
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1 -Browser
```

此时入口为：

- admin：`http://127.0.0.1:5173/admin/`
- portal：`http://127.0.0.1:5174/portal/`
- 后端仍使用 `9980`、`9981`、`9982`

### 4. 停止托管开发运行时

Linux 或 macOS：

```bash
./bin/stop-dev.sh
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\stop-dev.ps1
```

停止脚本会读取托管 PID 文件，停止受管进程树，并在可能时清理陈旧运行时状态。

## 发布态生命周期

推荐的发布流程使用托管发布脚本。

### 1. 构建发布产物

Linux 或 macOS：

```bash
./bin/build.sh
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1
```

这一阶段会编译并打包：

- Rust release 二进制
- admin / portal 静态资源
- 可选的 docs 站点资源，用于文档校验
- portal desktop 的 `router-product/` sidecar 载荷
- 正式 portal desktop bundle
- `artifacts/release/` 下的原生发布包

### 2. 安装产品根目录

Linux 或 macOS：

```bash
./bin/install.sh
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1
```

默认会创建产品根目录：

- `artifacts/install/sdkwork-api-router/`

关键目录包括：

- `current/`
- `releases/<version>/`
- `config/router.env`
- `config/router.yaml`
- `data/`
- `log/`
- `run/`
- `current/service/systemd/`
- `current/service/launchd/`
- `current/service/windows-service/`

### 3. 审阅并调整运行时配置

启动生产运行时之前，请审阅：

- `config/router.env`
- `config/router.yaml`
- `current/release-manifest.json`

将 `router.yaml` 视为标准运行时配置文件，主要承载：

- 绑定地址
- 数据库位置
- 代理目标

`router.env` 只用于配置发现与兜底值补全，不应再承担主配置职责。

`current/release-manifest.json` 是生成出来的元数据，它把 `current/` 指向当前激活的 `releases/<version>/` 不可变载荷。

### 4. 启动发布运行时

Linux 或 macOS：

```bash
./bin/start.sh --home artifacts/install/sdkwork-api-router/current
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start.ps1 -Home .\artifacts\install\sdkwork-api-router\current
```

发布态启动脚本会：

- 启动 `router-product-service`
- 从 `current/release-manifest.json` 解析激活的二进制和静态资源目录
- 在 portable 模式下默认使用产品根目录内的 SQLite
- 等待统一健康检查通过
- 输出与开发态一致风格的启动摘要

### 5. 停止发布运行时

Linux 或 macOS：

```bash
./bin/stop.sh --home artifacts/install/sdkwork-api-router/current
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\stop.ps1 -Home .\artifacts\install\sdkwork-api-router\current
```

### 6. 可选：注册系统服务

从产品根目录执行：

- Linux / systemd：
  - `./current/service/systemd/install-service.sh`
  - `./current/service/systemd/uninstall-service.sh`
- macOS / launchd：
  - `./current/service/launchd/install-service.sh`
  - `./current/service/launchd/uninstall-service.sh`
- Windows / Service Control Manager：
  - `powershell -NoProfile -ExecutionPolicy Bypass -File .\current\service\windows-service\install-service.ps1`
  - `powershell -NoProfile -ExecutionPolicy Bypass -File .\current\service\windows-service\uninstall-service.ps1`

service manager 场景下应使用前台模式：

- `bin/start.sh --foreground --home <product-root>/current`
- `bin/start.ps1 -Foreground -Home <product-root>\current`

## Portal Desktop 生命周期

portal desktop 产品有自己独立的打包和运行时生命周期，不使用 `bin/start.sh`，也不通过操作系统后台服务注册来驱动用户桌面应用。

构建输入：

- `pnpm --dir apps/sdkwork-router-admin build`
- `pnpm --dir apps/sdkwork-router-portal build`
- `cargo build --release -p router-product-service`
- `node scripts/prepare-router-portal-desktop-runtime.mjs`

运行时契约：

- Tauri 桌面壳启动随包分发的 `router-product-service` sidecar
- 固定本地桌面入口：`http://127.0.0.1:3001`
- 访问模式决定公开 bind：
  - 仅本机：`127.0.0.1:3001`
  - 局域网共享：`0.0.0.0:3001`
- 这里的局域网共享 bind 仅属于 desktop 访问模式；native server 安装在未额外配置时仍默认 `127.0.0.1:3001`
- 可变状态落在操作系统标准的每用户 config、data、log 目录
- `desktop-runtime.json` 保存桌面壳访问模式
- `router.yaml` 是 sidecar 的标准配置文件

## Dry-Run 生命周期

所有托管脚本都支持 dry-run，可在改动机器之前先预检：

- `./bin/build.sh --dry-run`
- `./bin/install.sh --dry-run`
- `./bin/start-dev.sh --dry-run`
- `./bin/start.sh --dry-run`

Windows：

- `powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1 -DryRun`
- `powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 -DryRun`
- `powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1 -DryRun`
- `powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start.ps1 -DryRun`

## 运维注意事项

- `bin/start-dev.*` 只用于源码树内的托管开发流程，不依赖 `bin/build.*` 或 `bin/install.*`
- `bin/start.*` 只用于安装后的发布运行时，不负责构建和安装
- `bin/stop-dev.*` 与 `bin/stop.*` 只管理各自运行目录中的 PID 和进程树
- gateway 没有默认用户名密码，主要面向 portal 签发的 API key
- 如果前端迭代需要 Vite 热更新和独立端口，请使用 `--browser` 或直接使用 `scripts/dev/*`
- 需要统一对外入口时，优先使用 preview、发布态或系统服务托管模式
