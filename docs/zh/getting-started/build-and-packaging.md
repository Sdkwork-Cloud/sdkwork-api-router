# 编译与打包

本页说明如何从仓库检出构建工程产物、准备正式发布输入，并打包 SDKWork 的两个正式产品。

如果你要查看 GitHub Releases 对外发布的正式资产，请使用 [发布构建](/zh/getting-started/release-builds)。如果你要查看 GitHub 托管发布流程本身，请使用 [线上发布](/zh/getting-started/online-release)。如果你要做生产安装和部署，请使用 [生产部署](/zh/getting-started/production-deployment)。

## 正式产品

SDKWork API Router 只发布两个正式用户可见产品：

- `sdkwork-api-router-product-server`
- `sdkwork-router-portal-desktop`

仓库中的其余产物都属于构建输入、开发输出或验证资产，不属于正式发布物。

## 构建目标

| 目标 | 命令 | 输出 |
|---|---|---|
| 产品宿主运行时 | `cargo build --release -p router-product-service` | `target/release/router-product-service` |
| 独立服务二进制 | `cargo build --release -p gateway-service -p admin-api-service -p portal-api-service -p router-web-service` | `target/release/` |
| admin 浏览器应用 | `pnpm --dir apps/sdkwork-router-admin build` | `apps/sdkwork-router-admin/dist/` |
| portal 浏览器应用 | `pnpm --dir apps/sdkwork-router-portal build` | `apps/sdkwork-router-portal/dist/` |
| portal desktop sidecar 载荷 | `node scripts/prepare-router-portal-desktop-runtime.mjs` | `bin/portal-rt/router-product/` |
| 正式 portal desktop 安装包 | `pnpm --dir apps/sdkwork-router-portal tauri:build` | Tauri 平台打包输出 |
| docs 站点 | `pnpm --dir docs build` | `docs/.vitepress/dist/` |

## 构建 Server 产品输入

先编译 server 侧 Rust 二进制：

```bash
cargo build --release -p router-product-service -p gateway-service -p admin-api-service -p portal-api-service -p router-web-service
```

再构建会被 server 产品归档打入的前端静态资源：

```bash
pnpm --dir apps/sdkwork-router-admin install
pnpm --dir apps/sdkwork-router-admin build
pnpm --dir apps/sdkwork-router-portal install
pnpm --dir apps/sdkwork-router-portal build
```

如果你希望使用与正式发布流程一致的仓库托管打包链路，直接运行：

```bash
./bin/build.sh
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1
```

如果要走受治理的本地正式发布路径，请使用 `./bin/build.sh --verify-release` 或 `powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1 -VerifyRelease`。这个模式会始终包含 docs 站点构建，因为官方 release 验证会把 docs site 视为公开产品面的一部分。同时它还会执行本地的 release governance preflight，也就是 `node scripts/release/run-release-governance-checks.mjs --profile preflight`，从而在打包 smoke 之外继续验证 release 治理契约。`--skip-docs cannot be combined with --verify-release`。

该托管构建会准备 `sdkwork-api-router-product-server` 所需输入，并把原生发布资产写入 `artifacts/release/`。

托管构建对正式 server 产品输出的标准文件是：

- `artifacts/release/native/<platform>/<arch>/bundles/sdkwork-api-router-product-server-<platform>-<arch>.tar.gz`
- `artifacts/release/native/<platform>/<arch>/bundles/sdkwork-api-router-product-server-<platform>-<arch>.tar.gz.sha256.txt`
- `artifacts/release/native/<platform>/<arch>/bundles/sdkwork-api-router-product-server-<platform>-<arch>.manifest.json`

外部 server manifest 会描述归档文件、校验文件、内嵌 bundle 契约、受治理的 `releaseVersion` 以及 bundle 根目录安装入口。解压 server 归档后，会得到一个已经包含 `install.sh`、`install.ps1`、`bin/`、`sites/`、`data/`、`deploy/`、`README.txt` 以及内嵌 `release-manifest.json` 的产品根目录。
这份官方 server bundle 还会内嵌 `control/bin/` 和 `control/bin/lib/`；bundle 根目录原生安装工具会从这棵受治理的控制树物化出安装后的 `current/bin/` operator surface，而不是依赖仓库本地辅助脚本。

当托管构建的输出树已经具备完整的正式资产集合时，还会在 `artifacts/release/release-catalog.json` 生成发布级元数据索引。这个 catalog 会把两个正式产品的外部 manifest 聚合成一个统一的机器可读 release 索引，并记录 `generatedAt` 以及每个 variant 的 `variantKind`、`primaryFileSizeBytes` 和 `checksumAlgorithm` 字段；它属于发布元数据，而不是第三个可安装产品。

## 构建正式 Portal Desktop 产品

正式 desktop 产品采用 portal-first 标准。它以 `apps/sdkwork-router-portal` 作为原生外壳，并内置一份 release 风格的 `router-product-service` sidecar 载荷。

先预制运行时载荷：

```bash
node scripts/prepare-router-portal-desktop-runtime.mjs
```

开发态运行桌面壳：

```bash
pnpm --dir apps/sdkwork-router-portal tauri:dev
```

如果你希望直接从仓库根目录进入正式产品开发流，可以使用：

```bash
pnpm tauri:dev
pnpm server:dev
```

`pnpm tauri:dev` 会通过统一根入口启动 portal desktop 产品路径。
`pnpm server:dev` 会通过同一套根入口启动完整的 server 开发工作区。
这个根级 `server:dev` 仅用于开发，会同时启动 backend API、admin Vite server、portal Vite server，以及统一的 Pingora web host。

如果你需要独立的一体化 `router-product-service` CLI 或面向部署的 server 运行时参数，请使用 `pnpm --dir apps/sdkwork-router-portal server:start`。

构建正式 desktop 安装包：

```bash
pnpm --dir apps/sdkwork-router-portal tauri:build
```

托管构建会把正式 desktop 产品标准化为以下文件：

- `artifacts/release/native/<platform>/<arch>/desktop/portal/sdkwork-router-portal-desktop-<platform>-<arch>.<ext>`
- `artifacts/release/native/<platform>/<arch>/desktop/portal/sdkwork-router-portal-desktop-<platform>-<arch>.<ext>.sha256.txt`
- `artifacts/release/native/<platform>/<arch>/desktop/portal/sdkwork-router-portal-desktop-<platform>-<arch>.manifest.json`

原始 Tauri bundle 目录仍然只是平台构建过程中的中间产物，不属于对外的正式打包、文档或发布契约。

admin Tauri 仍然可以作为显式开发路径使用，但它不属于正式发布产品集合。

## 构建独立浏览器应用

admin 浏览器应用：

```bash
pnpm --dir apps/sdkwork-router-admin build
pnpm --dir apps/sdkwork-router-admin preview
```

portal 浏览器应用：

```bash
pnpm --dir apps/sdkwork-router-portal build
pnpm --dir apps/sdkwork-router-portal preview
```

这些路径适合源码开发和本地验证，但不会作为独立 GitHub 发布物对外发布。

## 构建文档站

```bash
pnpm --dir docs install
pnpm --dir docs build
pnpm --dir docs preview
```

docs 站点对临时工程构建可以按需处理，但一旦进入 `--verify-release` 的托管本地正式发布验证流程，就必须重新纳入构建与治理；同一流程还会执行 release governance preflight，确保本地正式验证和发布治理标准保持一致。

## 打包前推荐验证

```bash
cargo fmt --all --check
cargo test --workspace -q -j 1
pnpm --dir apps/sdkwork-router-admin build
pnpm --dir apps/sdkwork-router-portal build
node scripts/prepare-router-portal-desktop-runtime.mjs
pnpm --dir docs build
```

如果同时修改了 TypeScript 或 docs 配置：

```bash
pnpm --dir apps/sdkwork-router-admin typecheck
pnpm --dir apps/sdkwork-router-portal typecheck
pnpm --dir docs typecheck
```

## 相关文档

- 源码开发流程：
  - [源码运行](/zh/getting-started/source-development)
- 正式发布资产：
  - [发布构建](/zh/getting-started/release-builds)
- GitHub 托管发布：
  - [线上发布](/zh/getting-started/online-release)
- 生产安装和部署：
  - [生产部署](/zh/getting-started/production-deployment)
- 仓库结构：
  - [仓库结构](/zh/reference/repository-layout)
