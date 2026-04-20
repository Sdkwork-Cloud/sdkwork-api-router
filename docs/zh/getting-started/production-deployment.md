# 生产部署

这是 SDKWork API Router 的正式生产部署指南。

当你要发布线上 server、准备原生安装、使用 Docker Compose，或者交付 Helm 部署时，都应以本页为准。

如果你需要查看 GitHub Actions release 流程本身、仓库变量与密钥、desktop 签名钩子，或发布后的 GitHub 校验流程，请转到[在线发布](/zh/getting-started/online-release)。

## 产品契约

- 正式 server 产品是 `sdkwork-api-router-product-server`
- 正式 desktop 产品是 `sdkwork-router-portal-desktop`
- GitHub Releases 对外只发布这两个产品
- `release-catalog.json` 与产品一同发布，但它是 release metadata，不是第三个产品
- `system` install mode 是原生生产环境标准
- `system` installs default to PostgreSQL
- 配置文件是首要事实源
- 环境变量承担 discovery input 和 fallback value 的角色
- 服务托管应交给 `systemd`、`launchd` 或 Windows Service Control Manager

desktop 产品不是线上 server 的部署路径。它是一个按用户安装的 Tauri 壳，负责监管随包分发的 `router-product-service` sidecar，并在固定桌面端口 `3001` 上暴露同一套 Web 与 API 面。

## Server 产品内容

server 产品归档围绕 `router-product-service` 组织，包含：

- release service binaries
- admin 静态资源
- portal 静态资源
- bootstrap data
- Docker 与 Helm 所需的 deploy assets

这份 packaged server bundle 是以下交付方式的唯一标准输入：

- 原生 server 安装
- Docker 镜像构建
- Docker Compose
- Helm

release workflow 在发布前还会对这份 packaged server bundle 执行 `installed-runtime smoke`，因此原生安装路径验证的就是 operator 实际部署的同一份正式产物。
原生安装工具会先从 `release-catalog.json` 选择标准 server archive，再拒绝任何与该 catalog 条目不一致的 archive、checksum 或外部 manifest。

## 选择部署路径

### Docker Compose

适用于需要最快单机交付、并且愿意同时部署 PostgreSQL 的场景。

主要资产：

- `deploy/docker/Dockerfile`
- `deploy/docker/docker-compose.yml`
- `deploy/docker/.env.example`

### Helm

适用于 Kubernetes 环境，并假设 PostgreSQL 由外部独立管理。

主要资产：

- `deploy/helm/sdkwork-api-router/`

### 原生系统安装

适用于需要遵循操作系统标准目录布局并由服务管理器托管启动的场景。

## 构建正式发布输入

Linux 或 macOS：

```bash
./bin/build.sh
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1
```

这会准备与 release workflow 完全一致的 server 构建输入：

- Rust release service binaries
- admin 与 portal 浏览器静态资源
- staged portal desktop `router-product/` payload
- packaged server product archive

如果你要在本地仓库直接验证与正式发布相同的治理契约，请使用 `./bin/build.sh --verify-release` 或 `powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1 -VerifyRelease`。该模式会在同样的构建输入基础上继续要求 docs site 构建、packaged runtime smoke，以及本地 `release governance preflight`。

对原生安装来说，唯一有效输入就是 packaged server bundle。解压归档后，受治理的 bundle-root `install.sh` 与 `install.ps1` 会把 `bin/`、`sites/*/dist/`、`data/`、`deploy/`、`release-manifest.json` 和 `README.txt` 物化到 `releases/<version>/`。
这份 bundle 还携带受治理的 `control/bin/` 控制树，用于生成安装后的 `current/bin/` 入口，使生产运维始终绑定到正式 release artifact。
`release-catalog.json` 是整套正式资产中用于选择与解析该 bundle 的 release 级事实源。

## Release Governance

release workflow 会把治理证据与用户可安装产品严格分离：

- `governance-release` 负责产出 release-window、sync-audit、telemetry、SLO evidence，以及第三方治理资产 `docs/release/third-party-sbom-latest.spdx.json` 与 `docs/release/third-party-notices-latest.json`
- `native-release` 负责构建正式 server 与 portal desktop 产品
- governance artifacts 保持为 workflow artifacts 与 attestations
- 面向用户的可安装产品仍然只包括 server archive 集合与 portal desktop installer 集合
- `release-catalog.json` 在 `artifacts/release/release-catalog.json` 生成、attest，并作为正式资产集的 machine-readable release index 一同发布
- catalog 同时记录 `generatedAt`、每个 variant 的 `variantKind`、`primaryFileSizeBytes` 与 `checksumAlgorithm`，供审计与部署工具使用

如果你要从仓库 checkout 本地执行治理校验：

Linux 或 macOS：

```bash
export SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_ROOT="$PWD/artifacts/external-release-deps"
node scripts/release/materialize-external-deps.mjs
node scripts/release/verify-release-sync.mjs --format text --live
node scripts/release/run-release-governance-checks.mjs
```

Windows：

```powershell
$env:SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_ROOT = (Join-Path (Get-Location) 'artifacts\external-release-deps')
node scripts/release/materialize-external-deps.mjs
node scripts/release/verify-release-sync.mjs --format text --live
node scripts/release/run-release-governance-checks.mjs
```

## 生成原生 Server 安装

Linux 或 macOS：

```bash
./install.sh --mode system
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1 -Mode system
```

生成后的生产资产包括：

- 标准 `router.yaml`
- `conf.d/` overlay 目录
- `router.env`
- `router.env.example`
- `systemd`、`launchd` 与 Windows Service 所需的 service descriptors

## 初始化生产配置

首次启动前，请先编辑生成好的运行时配置：

- `router.yaml`
  - 标准运行时主配置
- `conf.d/*.{yaml,yml,json}`
  - 可选的分域覆盖配置
- `router.env`
  - 当配置文件未显式给出某字段时，用于 discovery 与 fallback

推荐先完成以下修改：

- 将 PostgreSQL placeholder 替换成真实数据库 URL
- 设置 JWT、credential 与 metrics 所需的 secrets
- 审查 bind 地址与可信网络边界
- 确认 admin 与 portal 静态站点目录

## 服务注册前校验

在已安装的产品根目录中执行：

```bash
./current/bin/validate-config.sh --home <product-root>
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\current\bin\validate-config.ps1 -Home <product-root>
```

如果你仍在仓库 checkout 中操作，可使用托管 fallback：

```bash
node bin/router-ops.mjs validate-config --mode system --home <product-root>
```

```powershell
node .\bin\router-ops.mjs validate-config --mode system --home <product-root>
```

校验内容包括：

- 配置发现与合并顺序
- 配置文件优先于环境变量的业务字段优先级
- 生产安全姿态
- 启动时与 `validate-config` 阶段对 placeholder 数据库 URL 与 secrets 的拒绝
- `system` 模式下默认拒绝 SQLite，除非显式启用开发覆盖

## 备份与恢复

在已安装的产品根目录中执行：

```bash
./current/bin/backup.sh --home <product-root> --output ./backups/2026-04-19 --force
./current/bin/restore.sh --home <product-root> --source ./backups/2026-04-19 --force
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\current\bin\backup.ps1 -Home <product-root> -OutputPath .\backups\2026-04-19 -Force
powershell -NoProfile -ExecutionPolicy Bypass -File .\current\bin\restore.ps1 -Home <product-root> -SourcePath .\backups\2026-04-19 -Force
```

同样支持 dry-run：

```bash
./current/bin/backup.sh --home <product-root> --output ./backups/2026-04-19 --dry-run
./current/bin/restore.sh --home <product-root> --source ./backups/2026-04-19 --force --dry-run
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\current\bin\backup.ps1 -Home <product-root> -OutputPath .\backups\2026-04-19 -DryRun
powershell -NoProfile -ExecutionPolicy Bypass -File .\current\bin\restore.ps1 -Home <product-root> -SourcePath .\backups\2026-04-19 -Force -DryRun
```

运行契约：

- 备份前与恢复前都必须先停止托管运行时
- 当安装后的数据库 URL 使用 PostgreSQL 时，backup bundle 会包含 `control/release-manifest.json`、完整配置快照、可变数据快照，以及 PostgreSQL dump
- `backup-manifest.json` 是 machine-readable 的 backup contract，当前 `formatVersion` 为 `2`，其中 `bundle.controlManifestFile`、`bundle.configSnapshotRoot` 与 `bundle.mutableDataSnapshotRoot` 字段声明导出的 control manifest、配置快照和可变数据快照路径
- 恢复会用该 bundle 替换已安装的配置与可变数据目录，然后基于恢复后的运行时配置回放 PostgreSQL dump
- `log/` 与 `run/` 属于运行态目录，不从 backup bundle 恢复
- PostgreSQL 备份依赖 `pg_dump`；恢复依赖 `pg_restore`，两者都必须在 `PATH` 中

## Support Bundle

在已安装的产品根目录中执行：

```bash
./current/bin/support-bundle.sh --home <product-root> --output ./support/2026-04-19 --force
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\current\bin\support-bundle.ps1 -Home <product-root> -OutputPath .\support\2026-04-19 -Force
```

同样支持 dry-run：

```bash
./current/bin/support-bundle.sh --home <product-root> --output ./support/2026-04-19 --dry-run
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\current\bin\support-bundle.ps1 -Home <product-root> -OutputPath .\support\2026-04-19 -DryRun
```

运行契约：

- support bundle 用于 operator diagnostics，不用于 disaster recovery
- support-bundle 可以在不替换可变状态的前提下对已安装运行时安全导出
- bundle 会包含 `control/release-manifest.json`、redacted config snapshots、logs inventory / redacted text captures、runtime-state inventory 与 process-state metadata
- `support-bundle-manifest.json` 是 machine-readable 的 support export contract，当前 `formatVersion` 为 `2`，其中 `paths.controlManifestFile`、`paths.configSnapshotRoot`、`paths.configInventoryFile`、`paths.logsSnapshotRoot`、`paths.logsInventoryFile`、`paths.runtimeSnapshotRoot`、`paths.runtimeInventoryFile` 与 `paths.processStateFile` 字段声明导出产物路径
- 已知含密钥的配置值会在导出前被 redacted；binary credential stores 与 key material 会被省略
- support bundle 用于 operator escalation 与 release-audit capture；灾备迁移仍应通过 backup / restore 完成

## 注册并启动服务

请把前台启动入口交给系统服务管理器托管：

- Linux: `./current/service/systemd/install-service.sh`
- macOS: `./current/service/launchd/install-service.sh`
- Windows: `powershell -NoProfile -ExecutionPolicy Bypass -File .\current\service\windows-service\install-service.ps1`

配套文档：

- [安装布局](/zh/operations/install-layout)
- [服务管理](/zh/operations/service-management)

## Docker Compose 快速部署

```bash
tar -xzf sdkwork-api-router-product-server-linux-x64.tar.gz
cd sdkwork-api-router-product-server-linux-x64
cp deploy/docker/.env.example deploy/docker/.env
docker build -f deploy/docker/Dockerfile -t sdkwork-api-router:local .
docker compose -f deploy/docker/docker-compose.yml --env-file deploy/docker/.env up -d
```

在第一次执行 `docker compose up` 之前，请先替换 `deploy/docker/.env` 中所有 `replace-with-*` 值。容器入口与 `validate-config` 都会对 placeholder 数据库凭证、JWT secrets、credential keys 与 metrics tokens 失败关闭。

## Helm 快速部署

```bash
helm upgrade --install sdkwork-api-router deploy/helm/sdkwork-api-router \
  --set image.repository=ghcr.io/<owner>/sdkwork-api-router \
  --set image.tag=<release-tag> \
  --set secrets.databaseUrl='postgresql://sdkwork:replace-with-db-password@postgresql:5432/sdkwork_api_router' \
  --set secrets.adminJwtSigningSecret='replace-with-admin-jwt-secret' \
  --set secrets.portalJwtSigningSecret='replace-with-portal-jwt-secret' \
  --set secrets.credentialMasterKey='replace-with-credential-master-key' \
  --set secrets.metricsBearerToken='replace-with-metrics-token'
```

正式 GitHub release 还会同步发布多架构 Linux OCI 镜像 `ghcr.io/<owner>/sdkwork-api-router:<release-tag>`。workflow 会先发布 `:<release-tag>-linux-x64` 与 `:<release-tag>-linux-arm64`，再在 GHCR 中合成为公开的多架构 manifest tag。

## 初始化检查清单

- 目标平台 release 输入已成功构建
- PostgreSQL 已创建并可连接
- 已审阅 `router.yaml`
- 已替换 `router.env` 中的 secrets
- 已成功执行 `validate-config`
- 已通过操作系统原生服务管理器完成注册
- 首次启动后已验证 health endpoints
