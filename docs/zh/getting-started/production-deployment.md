# 生产部署

本页是 SDKWork API Router 的正式生产部署入口。

当你要发布线上 server、生成原生安装、使用 Docker Compose，或通过 Helm 进行部署时，请从这里开始。

## 产品契约

- 正式的 server 产品是 `sdkwork-api-router-product-server`
- 正式的 desktop 产品是 `sdkwork-router-portal-desktop`
- 公开 GitHub Release 只发布这两个正式产品
- `release-catalog.json` 会随发布一起提供，但它只是发布元数据，不是第三个产品
- `system` 安装模式是原生生产安装标准
- `system` 模式默认数据库契约是 PostgreSQL
- 配置文件是运行时配置的主数据源
- 环境变量只承担配置发现和字段兜底
- 服务托管应交给 `systemd`、`launchd` 或 Windows Service Control Manager

desktop 产品不是线上 server 的部署方式。它是一个每用户的 Tauri 桌面壳，负责托管随包分发的 `router-product-service` sidecar，并在固定的桌面端口 `3001` 上暴露同一套 Web 与 API 面。

## Server 产品包含什么

server 产品以 `router-product-service` 为核心，归档包中包含：

- release 服务二进制
- admin 静态资源
- portal 静态资源
- bootstrap 数据
- Docker 与 Helm 所需的部署资产

这个归档包是以下部署方式的统一输入：

- 原生 server 安装
- Docker 镜像构建
- Docker Compose
- Helm

Release 流程还会在发布前对同一份 `packaged server bundle` 执行 `installed-runtime smoke`，确保原生安装链路验证的是与最终交付完全一致的打包产物。
原生安装工具会先通过 `release-catalog.json` 解析正式 server archive，再把 archive、checksum 和外部 manifest 与 catalog 条目逐一比对，之后才执行解包安装。

## 选择部署路径

### Docker Compose

适合单机快速上线，并内置 PostgreSQL。

主要资产：

- `deploy/docker/Dockerfile`
- `deploy/docker/docker-compose.yml`
- `deploy/docker/.env.example`

### Helm

适合 Kubernetes 集群，通常使用外部托管的 PostgreSQL。

主要资产：

- `deploy/helm/sdkwork-api-router/`

### 原生 System 安装

适合需要操作系统标准目录、标准服务托管和长期运维的服务器环境。

## 构建正式发布输入

Linux 或 macOS：

```bash
./bin/build.sh
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1
```

这一步会准备与正式 Release 相同的 server 输入：

- Rust release 服务二进制
- admin / portal 前端静态资源
- portal desktop 的 `router-product/` sidecar 载荷
- server 产品归档包

如果你希望本地仓库执行的就是与正式发布一致的受治理校验路径，请改用 `./bin/build.sh --verify-release` 或 `powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1 -VerifyRelease`。这个模式会复用同一套构建输入，并额外要求 docs 站点构建、打包运行时 smoke，以及本地 `release governance preflight`。

对于原生安装，唯一有效的输入就是 `packaged server bundle`。安装器会把其中的 `bin/`、`sites/*/dist/`、`data/`、`deploy/`、`release-manifest.json` 和 `README.txt` 物化到 `releases/<version>/`。
当一套正式资产已经齐备时，`release-catalog.json` 就是选择这份 bundle 的发布级真相源。

## Release Governance

正式发布流程会把治理证据和用户可下载产品分开处理：

- `governance-release` 负责生成 release-window、sync-audit、telemetry 和 SLO 证据
- `native-release` 负责构建正式的 server 与 portal desktop 产品
- 治理证据保留为 workflow artifact 和 attestation
- 对外可安装产品仍然只包含 server 归档集合和 portal desktop 安装包集合
- `release-catalog.json` 会在 `artifacts/release/release-catalog.json` 生成、纳入 attestation，并作为正式资产集合的机器可读发布索引一起发布
- catalog 还会显式提供 `generatedAt` 以及每个 variant 的 `variantKind`、`primaryFileSizeBytes`、`checksumAlgorithm` 元数据，供审计和部署工具直接使用

在本地仓库 checkout 中执行治理校验时，可使用：

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
./bin/install.sh --mode system
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 -Mode system
```

生成的生产安装资产包括：

- 标准 `router.yaml`
- `conf.d/` 覆盖目录
- `router.env`
- `router.env.example`
- 面向 `systemd`、`launchd` 与 Windows Service 的服务描述文件

## 初始化生产配置

首次启动前，请编辑生成的运行时配置：

- `router.yaml`
  - 运行时的标准主配置
- `conf.d/*.yaml`
  - 可选的领域化覆盖配置
- `router.env`
  - 配置发现信息，以及配置文件未定义字段的兜底值

建议优先修改：

- PostgreSQL 连接串
- JWT、凭据主密钥和 metrics token
- 对外与对内监听地址
- admin / portal 静态资源目录

## 在注册服务前先校验

在安装目录中执行：

```bash
./current/bin/validate-config.sh --home ./current
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\current\bin\validate-config.ps1 -Home .\current
```

在源码仓库 checkout 中，也可以使用：

```bash
node bin/router-ops.mjs validate-config --mode system --home <install-root>
```

```powershell
node .\bin\router-ops.mjs validate-config --mode system --home <install-root>
```

校验内容包括：

- 配置发现与合并顺序
- 业务字段遵循“配置文件覆盖环境变量兜底”的优先级
- 生产安全姿态
- `system` 模式默认拒绝 SQLite，除非显式开启开发覆盖

## 注册并启动服务

使用前台模式交给系统服务管理器托管：

- Linux：
  - `./current/service/systemd/install-service.sh`
- macOS：
  - `./current/service/launchd/install-service.sh`
- Windows：
  - `powershell -NoProfile -ExecutionPolicy Bypass -File .\current\service\windows-service\install-service.ps1`

配套说明：

- [安装布局](/zh/operations/install-layout)
- [服务管理](/zh/operations/service-management)

## Docker Compose 快速部署

```bash
cp deploy/docker/.env.example deploy/docker/.env
docker build -f deploy/docker/Dockerfile -t sdkwork-api-router:local .
docker compose -f deploy/docker/docker-compose.yml --env-file deploy/docker/.env up -d
```

## Helm 快速部署

```bash
helm upgrade --install sdkwork-api-router deploy/helm/sdkwork-api-router \
  --set image.repository=ghcr.io/your-org/sdkwork-api-router \
  --set image.tag=2026.04.18 \
  --set secrets.databaseUrl='postgresql://sdkwork:change-me@postgresql:5432/sdkwork_api_router'
```

## 初始化检查清单

- 目标平台发布输入已成功构建
- PostgreSQL 已创建并可连通
- `router.yaml` 已完成审阅
- `router.env` 中的密钥已替换
- `validate-config` 已成功执行
- 已通过操作系统原生服务管理器完成注册
- 首次启动后已验证健康检查端点
