# 安装布局

本页定义正式 server 产品 `sdkwork-api-router-product-server` 的生产安装布局标准。

## 打包模型

每个 server 安装都拆分为三层：

- product root：稳定的顶层安装目录
- current control directory：供运维和服务管理器使用的稳定运行时包装层
- versioned release payload：位于 `releases/<version>/` 下的不可变程序载荷

所有可变状态都不能写入版本化程序载荷。

不可变的 `releases/<version>/` 载荷始终从官方发布的 bundle 物化，来源目录是 `artifacts/release/native/<platform>/<arch>/bundles/`。
安装器会直接根据 `artifacts/release/release-catalog.json` 选择并解析标准 bundle。
任何不在正式 catalog 中的 archive、checksum 或外部 manifest，都会在写入 `releases/<version>/` 之前被拒绝。
打包载荷会完整保留 bundle 中的 `bin/`、`sites/*/dist/`、`data/`、`deploy/`、`release-manifest.json` 和 `README.txt`。
解压后的 bundle 根目录还会暴露受治理的 `install.sh` 与 `install.ps1` 入口，外部 archive manifest 也会记录 `releaseVersion` 以及安装器契约。

## Portable 布局

`portable` 安装用于本地验证、CI smoke test 和显式的非系统安装。

默认 portable product root：

- `artifacts/install/sdkwork-api-router/`

目录结构：

- `current/`
  - `bin/`
  - `service/`
  - `release-manifest.json`
- `releases/<version>/`
  - `bin/`
  - `sites/admin/dist/`
  - `sites/portal/dist/`
  - `data/`
  - `deploy/`
  - `release-manifest.json`
  - `README.txt`
- `config/`
  - `router.yaml`
  - `router.env`
  - `router.env.example`
  - `conf.d/`
- `data/`
- `log/`
- `run/`

说明：

- `current/` 是控制层，只存放包装脚本和 service 资产。
- `current/bin/` 是稳定的 operator surface，包含 `start.sh`、`start.ps1`、`stop.sh`、`stop.ps1`、`validate-config.sh`、`validate-config.ps1`、`backup.sh`、`backup.ps1`、`restore.sh`、`restore.ps1`、`support-bundle.sh` 和 `support-bundle.ps1`。
- 安装后的 `current/bin/` 从官方 server bundle 内嵌的 `control/bin/` 树物化生成，因此 operator 控制脚本与受治理的正式发布产物一起交付。
- `releases/<version>/` 是当前激活的不可变程序载荷，来源于正式 `packaged server bundle`。
- `config/`、`data/`、`log/` 和 `run/` 始终保持可写，并且升级安全。

## System 布局

`system` 安装遵循操作系统标准的可变目录规划，同时把程序载荷固定在独立的产品根目录下。

### Linux

- product root：`/opt/sdkwork-api-router/`
- current control directory：`/opt/sdkwork-api-router/current/`
- versioned release payload：`/opt/sdkwork-api-router/releases/<version>/`
- config home：`/etc/sdkwork-api-router/`
- config file：`/etc/sdkwork-api-router/router.yaml`
- config fragments：`/etc/sdkwork-api-router/conf.d/`
- env file：`/etc/sdkwork-api-router/router.env`
- data home：`/var/lib/sdkwork-api-router/`
- log home：`/var/log/sdkwork-api-router/`
- run home：`/run/sdkwork-api-router/`

### macOS

- product root：`/usr/local/lib/sdkwork-api-router/`
- current control directory：`/usr/local/lib/sdkwork-api-router/current/`
- versioned release payload：`/usr/local/lib/sdkwork-api-router/releases/<version>/`
- config home：`/Library/Application Support/sdkwork-api-router/`
- config file：`/Library/Application Support/sdkwork-api-router/router.yaml`
- config fragments：`/Library/Application Support/sdkwork-api-router/conf.d/`
- env file：`/Library/Application Support/sdkwork-api-router/router.env`
- data home：`/Library/Application Support/sdkwork-api-router/data/`
- log home：`/Library/Logs/sdkwork-api-router/`
- run home：`/Library/Application Support/sdkwork-api-router/run/`

### Windows

- product root：`C:\Program Files\sdkwork-api-router\`
- current control directory：`C:\Program Files\sdkwork-api-router\current\`
- versioned release payload：`C:\Program Files\sdkwork-api-router\releases\<version>\`
- config home：`C:\ProgramData\sdkwork-api-router\`
- config file：`C:\ProgramData\sdkwork-api-router\router.yaml`
- config fragments：`C:\ProgramData\sdkwork-api-router\conf.d\`
- env file：`C:\ProgramData\sdkwork-api-router\router.env`
- data home：`C:\ProgramData\sdkwork-api-router\data\`
- log home：`C:\ProgramData\sdkwork-api-router\log\`
- run home：`C:\ProgramData\sdkwork-api-router\run\`

## 配置发现

主配置文件的发现顺序为：

1. `router.yaml`
2. `router.yml`
3. `router.json`
4. `config.yaml`
5. `config.yml`
6. `config.json`

`conf.d/*.{yaml,yml,json}` 下的受支持覆盖片段会在主文件之后按字典序加载。

## 配置优先级

实际优先级从低到高为：

- 内建默认值
- 环境变量兜底
- 配置文件
- CLI

发现阶段的例外项：

- `SDKWORK_CONFIG_DIR`
- `SDKWORK_CONFIG_FILE`

这两个变量会先被读取，用来定位配置文件集合。完成发现后，配置文件中定义的业务字段优先于环境变量兜底值。

## Release Manifest 契约

安装生成的 `current/release-manifest.json` 是 `current/` 与 `releases/<version>/` 之间的控制桥接文件。

它会记录：

- manifest 架构与生成元数据：`layoutVersion`、`installedAt`
- 安装拓扑与版本选择：`installMode`、`productRoot`、`controlRoot`、`releasesRoot`、`releaseRoot`、`releaseVersion`
- 解析后的目标三元组：`target`
- 已安装服务载荷清单：`installedBinaries`
- 当前激活的程序载荷路径：`routerBinary`、`adminSiteDistDir`、`portalSiteDistDir`
- 当前激活版本
- 当前激活的 release 根目录
- 实际 router 二进制路径
- admin / portal 静态资源目录
- 当前 release 内的引导数据与 `deploy/` 资产根目录：`bootstrapDataRoot`、`deploymentAssetRoot`
- 当前 release 内 `release-manifest.json` 与 `README.txt` 的路径：`releasePayloadManifest`、`releasePayloadReadmeFile`
- config / data / log / run 根目录以及主配置文件路径：`configRoot`、`configFile`、`mutableDataRoot`、`logRoot`、`runRoot`

`current/release-manifest.json` 属于生成状态，正常运维中不应手工修改。

## 数据库默认值

- `portable`
  - 默认使用 portable `data/` 目录下的 SQLite
- `system`
  - 默认使用 PostgreSQL

在 `system` 模式下，PostgreSQL 是正式标准契约。SQLite 只是本地验证便利项，不是生产默认值。
