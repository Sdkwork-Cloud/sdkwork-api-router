# 发布构建

本页说明如何生成并运行可部署的服务二进制、浏览器静态资源和可选桌面包。

如果你要找的是开发期编译命令，请先看 [编译与打包](/zh/getting-started/build-and-packaging)。本页聚焦发布产物。

## 发布构建目标

独立服务：

- `admin-api-service`
- `gateway-service`
- `portal-api-service`
- `router-web-service`
- `router-product-service`

面向用户的产物：

- admin Web 应用静态资源
- portal Web 应用静态资源
- 可选的 console Tauri 桌面包
- 可选的 admin Tauri 桌面包
- 可选的 portal Tauri 桌面包
- 内含 `router-product-service`、admin 站点和 portal 站点的按平台产品服务端归档包

## 构建 Rust 服务

构建全部发布服务二进制：

```bash
cargo build --release -p admin-api-service -p gateway-service -p portal-api-service
cargo build --release -p router-web-service -p router-product-service
```

输出二进制位于 `target/release/`。

### 输出路径

Windows 可执行文件名：

- `target/release/admin-api-service.exe`
- `target/release/gateway-service.exe`
- `target/release/portal-api-service.exe`
- `target/release/router-web-service.exe`
- `target/release/router-product-service.exe`

Linux / macOS 可执行文件名：

- `target/release/admin-api-service`
- `target/release/gateway-service`
- `target/release/portal-api-service`
- `target/release/router-web-service`
- `target/release/router-product-service`

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

## 构建 Admin Web 应用

```bash
pnpm --dir apps/sdkwork-router-admin install
pnpm --dir apps/sdkwork-router-admin build
```

输出目录：

- `apps/sdkwork-router-admin/dist/`

本地预览：

```bash
pnpm --dir apps/sdkwork-router-admin preview
```

## 构建 portal Web 应用

```bash
pnpm --dir apps/sdkwork-router-portal install
pnpm --dir apps/sdkwork-router-portal build
```

输出目录：

- `apps/sdkwork-router-portal/dist/`

## 构建 Tauri 桌面应用

```bash
pnpm --dir console tauri:build
pnpm --dir apps/sdkwork-router-admin tauri:build
pnpm --dir apps/sdkwork-router-portal tauri:build
```

这会在 Tauri 的平台输出目录下生成对应操作系统的桌面安装产物。

## 发布部署建议

推荐的服务端部署形态：

- 独立运行所有 Rust 服务，或按需组合运行
- 需要统一公网入口时，使用 `router-web-service` 承载 admin 和 portal 静态站点并代理 `/api/*`
- 需要单一产品级服务端入口时，使用 `router-product-service` 统一承载 `/admin/*`、`/portal/*` 和 `/api/*`
- 多用户持久化部署优先使用 PostgreSQL
- 上游密钥优先使用服务端 secret backend 策略
- 构建 `apps/sdkwork-router-admin/dist/` 与 `apps/sdkwork-router-portal/dist/`
- 通过 `router-web-service` 或 `router-product-service` 将这些静态资源暴露到 `/admin/` 与 `/portal/`

## 自动化 GitHub 发布

`.github/workflows/release.yml` 支持标签触发和手动触发的 GitHub Release。

当前自动化矩阵包括：

- Windows x64 和 arm64
- Linux x64 和 arm64
- macOS x64 和 arm64
- admin、portal、console 桌面安装包
- 独立服务二进制
- `router-product-service`
- 按平台打包的产品服务端归档包，内含 admin 与 portal 静态站点

推荐的桌面嵌入形态：

- 使用 Tauri 桌面壳
- 本地默认保留 SQLite
- 优先使用 OS keyring 或本地加密文件存储密钥

## 发布校验

```bash
cargo build --release -p admin-api-service -p gateway-service -p portal-api-service -p router-web-service -p router-product-service
pnpm --dir apps/sdkwork-router-admin build
pnpm --dir apps/sdkwork-router-portal build
pnpm --dir docs build
```
