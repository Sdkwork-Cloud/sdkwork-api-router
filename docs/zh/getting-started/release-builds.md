# 发布构建

本页说明 SDKWork API Router 的正式发布产物，以及本地和 GitHub Release 如何构建这些产物。

本页只负责正式发布相关的构建与打包产物生成。

如果你要做线上部署、PostgreSQL 初始化、服务注册，或按操作系统标准目录进行 server 安装，请继续阅读[生产部署](/zh/getting-started/production-deployment)。
如果你要查看 GitHub 托管发布流程本身、仓库变量、签名 hook 或发布后校验，请继续阅读[线上发布](/zh/getting-started/online-release)。

## 正式产品

SDKWork API Router 当前只发布两个正式、面向用户的产品：

- `sdkwork-api-router-product-server`
- `sdkwork-router-portal-desktop`

仓库内部仍然会生成一些中间产物，例如 Rust 二进制、前端 `dist/` 目录、Tauri bundle 目录，但这些都只是构建输入，不再作为正式发布产品对外暴露。

## Release 工作流会发布什么

GitHub Release 只发布以下正式产物：

- server 产品归档集合：
  - `sdkwork-api-router-product-server-<platform>-<arch>.tar.gz`
  - `sdkwork-api-router-product-server-<platform>-<arch>.tar.gz.sha256.txt`
  - `sdkwork-api-router-product-server-<platform>-<arch>.manifest.json`
- 当前平台的 portal desktop 安装包集合：
  - `sdkwork-router-portal-desktop-<platform>-<arch>.<ext>`
  - `sdkwork-router-portal-desktop-<platform>-<arch>.<ext>.sha256.txt`
  - `sdkwork-router-portal-desktop-<platform>-<arch>.manifest.json`
- 一个发布级资产索引：
  - `release-catalog.json`

不会再发布：

- 独立的 admin desktop 安装包
- 独立的 web 静态发布归档

治理证据、telemetry 导出、sync audit 和类似材料只保留为 workflow artifact 与 attestation，不作为用户下载资产。

`release-catalog.json` 会在仓库拥有一套完整正式资产时生成到 `artifacts/release/release-catalog.json`。本地 native 打包会为当前输出树生成它，release workflow 也会在 publish 阶段下载完原生正式资产后重新生成它。它把各个正式产品的外部 manifest 聚合成一个统一的机器可读 release 索引，方便自动化和运维使用。catalog 顶层会包含 `generatedAt` 时间戳；每个 variant 还会显式记录 `variantKind`、`primaryFileSizeBytes` 和 `checksumAlgorithm`，方便部署自动化在不重新解析每个 manifest 的情况下完成审计和资产识别。

## 托管本地构建

Linux 或 macOS：

```bash
./bin/build.sh
./bin/build.sh --verify-release
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1 -VerifyRelease
```

托管构建会准备正式产品所需的输入：

- Rust release 服务二进制
- server 产品使用的 admin / portal 前端静态资源
- portal desktop 使用的 `router-product/` sidecar 运行时载荷
- portal desktop 安装包
- `artifacts/release/` 下的原生发布目录
- 当输出目录已经具备完整正式资产集合时，一并生成本地 `artifacts/release/release-catalog.json`

`--verify-release` 是托管的本地正式发布验证模式。它会先完成同一套正式构建与打包步骤，然后直接基于打包后的正式资产执行按平台区分的 smoke 校验，而不是继续依赖原始工作区输出：

- Windows：打包 installed-runtime smoke
- macOS：打包 installed-runtime smoke
- Linux：打包 installed-runtime smoke、Docker Compose 打包 smoke，以及 Helm render 打包 smoke

`--verify-release` 还会始终包含受治理的 docs 站点构建，也就是 `pnpm --dir docs build`。正式的本地 release 验证会把 docs site 视为公开产品面的一部分，因此即使普通工程构建允许跳过文档，这个模式也不会跳过 docs 治理。

`--verify-release` 还会执行本地的 release governance preflight，也就是 `node scripts/release/run-release-governance-checks.mjs --profile preflight`。这样正式的本地 release 验证就不会只证明打包后的运行时 smoke，还会同时校验仓库里的 release 治理契约。

`--skip-docs cannot be combined with --verify-release`。如果你只是做本地工程构建并希望跳过 docs，请使用普通 build 模式，而不要使用正式的本地 release 验证模式。

## 本地正式产物位置

构建成功后，正式产物位于：

- server 产品归档集合：
  - `artifacts/release/native/<platform>/<arch>/bundles/`
  - 目录内包含：
    - `sdkwork-api-router-product-server-<platform>-<arch>.tar.gz`
    - `sdkwork-api-router-product-server-<platform>-<arch>.tar.gz.sha256.txt`
    - `sdkwork-api-router-product-server-<platform>-<arch>.manifest.json`
- portal desktop 安装包目录：
  - `artifacts/release/native/<platform>/<arch>/desktop/portal/`
  - 目录内只包含规范化后的正式发布文件：
    - `sdkwork-router-portal-desktop-<platform>-<arch>.<ext>`
    - `sdkwork-router-portal-desktop-<platform>-<arch>.<ext>.sha256.txt`
    - `sdkwork-router-portal-desktop-<platform>-<arch>.manifest.json`
- portal desktop 的 sidecar 资源载荷：
  - `bin/portal-rt/router-product/`
  - 目录内包含：
    - `router-product/bin/router-product-service`
    - `router-product/sites/admin/dist/`
    - `router-product/sites/portal/dist/`
    - `router-product/data/`
    - `router-product/release-manifest.json`
    - `router-product/README.txt`
- 发布级资产索引：
  - `artifacts/release/release-catalog.json`

## Portal Desktop 打包契约

portal desktop 不是把运行时内嵌到 Tauri 进程里，而是一个原生桌面壳加一个随包分发的、接近 release 形态的运行时载荷。

公开发布契约会刻意收窄，不直接暴露原始 Tauri bundle 目录。release packager 只会挑选一个当前平台的正式安装包，重命名为标准产品文件名，生成 SHA-256 校验文件，并输出记录 sidecar 契约的 manifest。这个 installer manifest 会通过 `embeddedRuntime.routerBinary`、`embeddedRuntime.adminSiteDir`、`embeddedRuntime.portalSiteDir`、`embeddedRuntime.bootstrapDataDir`、`embeddedRuntime.releaseManifestFile` 和 `embeddedRuntime.readmeFile` 明确暴露内嵌运行时路径。

该载荷包含：

- `router-product/bin/router-product-service`
- `router-product/sites/admin/dist/`
- `router-product/sites/portal/dist/`
- `router-product/data/`
- `router-product/release-manifest.json`
- `router-product/README.txt`

desktop 运行时契约：

- 固定的本地桌面入口：`http://127.0.0.1:3001`
- 访问模式切换：
  - 仅本机：`127.0.0.1:3001`
  - 局域网共享：`0.0.0.0:3001`
- 局域网共享 bind 只是 desktop 的显式访问模式覆盖；native server 产品在未被配置文件、环境变量或 CLI 改写时仍默认 `127.0.0.1:3001`
- 可变运行时状态落在操作系统标准的每用户 `config`、`data`、`log` 目录
- 桌面壳把访问模式持久化到 `desktop-runtime.json`
- 桌面壳会生成 sidecar 的标准 `router.yaml`
- 正式发布目录不再暴露原始 Tauri bundle 树，而只暴露标准化的 `sdkwork-router-portal-desktop-*` 产品文件

### Desktop 签名钩子

正式 desktop 发布流在收集规范化安装包之前支持一个显式的签名阶段。

- 当发布必须 fail-closed 时，设置 `SDKWORK_RELEASE_DESKTOP_SIGNING_REQUIRED=true`
- 可使用以下签名钩子变量之一：
  - `SDKWORK_RELEASE_DESKTOP_WINDOWS_SIGN_HOOK`
  - `SDKWORK_RELEASE_DESKTOP_LINUX_SIGN_HOOK`
  - `SDKWORK_RELEASE_DESKTOP_MACOS_SIGN_HOOK`
  - `SDKWORK_RELEASE_DESKTOP_SIGN_HOOK` 作为通用兜底
- 钩子命令支持占位符替换：`{app}`、`{platform}`、`{arch}`、`{target}`、`{file}`、`{evidence}`
- release workflow 会把签名证据写入 `artifacts/release-governance/desktop-release-signing-<platform>-<arch>.json`

仓库侧契约只保证安装包发现、签名钩子执行和证据落盘；具体的平台签名或 notarization 工具链由钩子命令自身负责。

## Server 打包契约

server 产品以标准化归档集合发布，而不是直接暴露原始构建目录。release packager 会产出：

- 一个标准 `sdkwork-api-router-product-server-<platform>-<arch>.tar.gz`
- 一个对应的 SHA-256 校验文件
- 一个外部 `sdkwork-api-router-product-server-<platform>-<arch>.manifest.json`

外部 manifest 记录归档标识以及内嵌 bundle 契约。归档内部的 `release-manifest.json` 继续描述安装和运行时工具使用的解包后产品载荷。

## Release Catalog 契约

`release-catalog.json` 是正式 SKU 集合的发布级索引。它不会替代每个资产自己的 manifest，而是对这些 manifest 做统一聚合；同时它也是两个可安装正式产品之外唯一额外公开发布的元数据资产。

catalog 会记录：

- release tag
- catalog 快照的 `generatedAt` 时间戳
- 正式 product id
- 每个平台、每个架构的 variant，以及对应的 `variantKind`
- 主资产文件名、`primaryFileSizeBytes`、校验文件名、`checksumAlgorithm`、manifest 文件名以及解析后的 SHA-256
- 每个正式资产对应的外部 manifest 内容

## 原生 Server 安装生成

仓库内的原生安装仍通过托管安装脚本生成。

Portable 安装：

```bash
./bin/install.sh
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1
```

面向生产的 system 安装：

```bash
./bin/install.sh --mode system
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 -Mode system
```

自定义 `<product-root>` 安装根目录：

```bash
./bin/install.sh --mode system --home <product-root>
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 -Mode system -Home <product-root>
```

当需要把仓库侧的安装生成直接落到指定的 `<product-root>` 时，请使用 `--home` 或 `-Home`。生成完成后，请按照[生产部署](/zh/getting-started/production-deployment)中的方式，从已安装的 product root 执行校验和运维操作。

`system` 模式是正式 server 安装路径，默认数据库契约为 PostgreSQL。

## Dry Run

```bash
./bin/build.sh --dry-run
./bin/install.sh --dry-run
./bin/install.sh --mode system --dry-run
./bin/install.sh --mode system --home <product-root> --dry-run
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1 -DryRun
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 -DryRun
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 -Mode system -DryRun
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 -Mode system -Home <product-root> -DryRun
```

## 验证

```bash
node --test bin/tests/router-runtime-tooling.test.mjs
node --test scripts/release/tests/release-workflow.test.mjs scripts/release/tests/run-unix-installed-runtime-smoke.test.mjs scripts/release/tests/run-windows-installed-runtime-smoke.test.mjs scripts/release/tests/deployment-assets.test.mjs
node --test scripts/release-governance-workflow.test.mjs
node --test scripts/product-verification-workflow.test.mjs
node --test scripts/rust-verification-workflow.test.mjs
node --test scripts/release-flow-contract.test.mjs scripts/prepare-router-portal-desktop-runtime.test.mjs apps/sdkwork-router-portal/tests/portal-desktop-api-base.test.mjs apps/sdkwork-router-portal/tests/portal-desktop-sidecar-runtime.test.mjs
```

## 下一步

- [生产部署](/zh/getting-started/production-deployment)
- [线上发布](/zh/getting-started/online-release)
- [安装布局](/zh/operations/install-layout)
- [服务管理](/zh/operations/service-management)
