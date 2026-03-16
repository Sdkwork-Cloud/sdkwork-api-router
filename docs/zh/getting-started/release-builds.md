# 发布构建

本页说明如何生成并运行可部署的服务二进制、浏览器静态资源和可选桌面包。

如果你要找的是开发期编译命令，请先看 [编译与打包](/zh/getting-started/build-and-packaging)。本页聚焦发布产物。

## 发布构建目标

独立服务：

- `admin-api-service`
- `gateway-service`
- `portal-api-service`

面向用户的产物：

- admin 控制台静态资源
- portal Web 应用静态资源
- 可选的 Tauri 桌面包

## 构建 Rust 服务

```bash
cargo build --release -p admin-api-service -p gateway-service -p portal-api-service
```

输出二进制位于 `target/release/`。

### 输出路径

Windows 可执行文件名：

- `target/release/admin-api-service.exe`
- `target/release/gateway-service.exe`
- `target/release/portal-api-service.exe`

Linux / macOS 可执行文件名：

- `target/release/admin-api-service`
- `target/release/gateway-service`
- `target/release/portal-api-service`

## 运行发布二进制

独立服务默认会从本地 SDKWork 配置根目录读取配置；如有需要，可通过 `SDKWORK_CONFIG_DIR` 或 `SDKWORK_CONFIG_FILE` 覆盖。

### Windows

```powershell
New-Item -ItemType Directory -Force "$HOME\.sdkwork\router" | Out-Null
$env:SDKWORK_CONFIG_FILE="$HOME\.sdkwork\router\config.yaml"
.\target\release\admin-api-service.exe
```

```powershell
$env:SDKWORK_CONFIG_FILE="$HOME\.sdkwork\router\config.yaml"
.\target\release\gateway-service.exe
```

```powershell
$env:SDKWORK_CONFIG_FILE="$HOME\.sdkwork\router\config.yaml"
.\target\release\portal-api-service.exe
```

### Linux 或 macOS

```bash
mkdir -p "$HOME/.sdkwork/router"
export SDKWORK_CONFIG_FILE="$HOME/.sdkwork/router/config.yaml"
./target/release/admin-api-service
```

```bash
export SDKWORK_CONFIG_FILE="$HOME/.sdkwork/router/config.yaml"
./target/release/gateway-service
```

```bash
export SDKWORK_CONFIG_FILE="$HOME/.sdkwork/router/config.yaml"
./target/release/portal-api-service
```

## 构建 admin 控制台前端

```bash
pnpm --dir console install
pnpm --dir console build
```

输出目录：

- `console/dist/`

本地预览：

```bash
pnpm --dir console preview
```

## 构建 portal Web 应用

```bash
pnpm --dir apps/sdkwork-router-portal install
pnpm --dir apps/sdkwork-router-portal build
```

输出目录：

- `apps/sdkwork-router-portal/dist/`

## 构建 Tauri 桌面包

```bash
pnpm --dir console tauri:build
```

这会在 Tauri 的平台输出目录下生成对应操作系统的桌面安装产物。

## 发布部署建议

推荐的服务端部署形态：

- 三个 Rust 服务作为独立进程运行
- 多用户持久化部署优先使用 PostgreSQL
- 上游密钥优先使用服务端 secret backend 策略
- 如果需要浏览器 admin 控制台，可单独托管 `console/dist/`
- 如果需要浏览器 developer portal，可单独托管 `apps/sdkwork-router-portal/dist/`

推荐的桌面嵌入形态：

- 使用 Tauri 桌面壳
- 本地默认保留 SQLite
- 优先使用 OS keyring 或本地加密文件存储密钥

## 发布校验

```bash
cargo build --release -p admin-api-service -p gateway-service -p portal-api-service
pnpm --dir console build
pnpm --dir apps/sdkwork-router-portal build
pnpm --dir docs build
```
