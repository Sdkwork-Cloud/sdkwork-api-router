# 安装布局

本页定义正式 server 产品 `sdkwork-api-router-product-server` 的安装布局标准。

## 打包模型

每个 server 安装都拆分为三层：

- 产品根目录：稳定的顶层安装目录
- `current` 控制层：供运维和服务管理器使用的稳定运行时入口
- `releases/<version>/`：不可变的版本化程序载荷

所有可变状态都不能写入版本化程序载荷。

安装器只接受官方 bundle 输入：`artifacts/release/native/<platform>/<arch>/bundles/`。
安装器会先读取 `artifacts/release/release-catalog.json`，确认目标 archive、checksum 和外部 manifest 与正式 catalog 条目一致后，才会把载荷物化到 `releases/<version>/`。
版本化载荷会保留包内的 `bin/`、`sites/*/dist/`、`data/`、`deploy/`、`release-manifest.json` 和 `README.txt`。

## Portable 布局

`portable` 用于本地验证、CI smoke test 和显式的非系统安装。

默认产品根目录：

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

- `current/` 是控制层，只放包装脚本和 service 资产。
- `releases/<version>/` 是当前激活的不可变程序载荷。
- `config/`、`data/`、`log/`、`run/` 是可写目录，升级时必须保留。

## System 布局

`system` 安装遵循操作系统标准目录，同时把程序载荷固定在独立的产品根目录下。

### Linux

- 产品根目录：`/opt/sdkwork-api-router/`
- `current` 控制层：`/opt/sdkwork-api-router/current/`
- 版本化载荷：`/opt/sdkwork-api-router/releases/<version>/`
- 配置目录：`/etc/sdkwork-api-router/`
- 主配置文件：`/etc/sdkwork-api-router/router.yaml`
- 配置片段目录：`/etc/sdkwork-api-router/conf.d/`
- 环境变量文件：`/etc/sdkwork-api-router/router.env`
- 数据目录：`/var/lib/sdkwork-api-router/`
- 日志目录：`/var/log/sdkwork-api-router/`
- 运行时目录：`/run/sdkwork-api-router/`

### macOS

- 产品根目录：`/usr/local/lib/sdkwork-api-router/`
- `current` 控制层：`/usr/local/lib/sdkwork-api-router/current/`
- 版本化载荷：`/usr/local/lib/sdkwork-api-router/releases/<version>/`
- 配置目录：`/Library/Application Support/sdkwork-api-router/`
- 主配置文件：`/Library/Application Support/sdkwork-api-router/router.yaml`
- 配置片段目录：`/Library/Application Support/sdkwork-api-router/conf.d/`
- 环境变量文件：`/Library/Application Support/sdkwork-api-router/router.env`
- 数据目录：`/Library/Application Support/sdkwork-api-router/data/`
- 日志目录：`/Library/Logs/sdkwork-api-router/`
- 运行时目录：`/Library/Application Support/sdkwork-api-router/run/`

### Windows

- 产品根目录：`C:\Program Files\sdkwork-api-router\`
- `current` 控制层：`C:\Program Files\sdkwork-api-router\current\`
- 版本化载荷：`C:\Program Files\sdkwork-api-router\releases\<version>\`
- 配置目录：`C:\ProgramData\sdkwork-api-router\`
- 主配置文件：`C:\ProgramData\sdkwork-api-router\router.yaml`
- 配置片段目录：`C:\ProgramData\sdkwork-api-router\conf.d\`
- 环境变量文件：`C:\ProgramData\sdkwork-api-router\router.env`
- 数据目录：`C:\ProgramData\sdkwork-api-router\data\`
- 日志目录：`C:\ProgramData\sdkwork-api-router\log\`
- 运行时目录：`C:\ProgramData\sdkwork-api-router\run\`

## 配置发现顺序

主配置文件的发现顺序为：

1. `router.yaml`
2. `router.yml`
3. `router.json`
4. `config.yaml`
5. `config.yml`
6. `config.json`

`conf.d/*.{yaml,yml,json}` 下的受支持片段会在主文件之后按字典序加载。

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

## Release Manifest 约定

安装生成的 `current/release-manifest.json` 是 `current/` 与 `releases/<version>/` 之间的控制桥接文件。

其中记录：

- manifest 架构与生成元数据：`layoutVersion`、`installedAt`
- 安装拓扑与版本选择：`installMode`、`productRoot`、`controlRoot`、`releasesRoot`、`releaseRoot`、`releaseVersion`
- 解析后的目标描述：`target`
- 已安装服务载荷清单：`installedBinaries`
- 当前激活版本
- 当前激活的 release 根目录
- 实际 router 二进制路径
- admin / portal 静态资源目录
- 当前 release 内的引导数据与 `deploy/` 部署资产根目录：`bootstrapDataRoot`、`deploymentAssetRoot`
- 当前 release 内 `release-manifest.json` 与 `README.txt` 的路径：`releasePayloadManifest`、`releasePayloadReadmeFile`
- config / data / log / run 根目录以及主配置文件路径：`configRoot`、`configFile`、`mutableDataRoot`、`logRoot`、`runRoot`

`current/release-manifest.json` 属于生成文件，正常运维中不应手工修改。

## 数据库默认值

- `portable`
  - 默认使用 portable `data/` 目录下的 SQLite
- `system`
  - 默认使用 PostgreSQL

在 `system` 模式下，PostgreSQL 是正式标准契约。SQLite 只用于本地验证，不是生产默认值。
