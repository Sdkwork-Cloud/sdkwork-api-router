# 配置

本页定义以下运行方式共享的配置契约：

- 已安装的 server 产品 `sdkwork-api-router-product-server`
- 已安装的 desktop 产品 `sdkwork-router-portal-desktop`
- 直接运行的原始 standalone 二进制，例如 `gateway-service`、`router-product-service`

## 解析顺序

字段的最终优先级从低到高为：

- 内建默认值
- 环境变量兜底
- 配置文件
- CLI

其中有两个输入比较特殊：

- `SDKWORK_CONFIG_DIR`
- `SDKWORK_CONFIG_FILE`

它们属于“配置发现输入”。运行时会优先读取它们，用来定位配置文件集合。完成发现后，只要配置文件中定义了某个字段，该字段就以配置文件为准；环境变量只负责填充配置文件未设置的字段。

`system` 安装默认使用 PostgreSQL。portable server 安装和本地开发流程仍可使用 SQLite。

运行时热加载会继续使用进程启动时捕获的环境变量兜底快照。服务进程启动后再修改父进程环境变量，不会被已运行进程感知。

## 已安装 Server 产品默认目录

Portable 安装：

- 配置根目录：`<product-root>/config/`
- 环境变量文件：`<product-root>/config/router.env`
- 主配置文件：`<product-root>/config/router.yaml`
- 数据目录：`<product-root>/data/`
- 日志目录：`<product-root>/log/`
- 运行目录：`<product-root>/run/`

System 安装：

- Linux 配置根目录：`/etc/sdkwork-api-router/`
- Linux 数据目录：`/var/lib/sdkwork-api-router/`
- Linux 日志目录：`/var/log/sdkwork-api-router/`
- Linux 运行目录：`/run/sdkwork-api-router/`
- macOS 配置根目录：`/Library/Application Support/sdkwork-api-router/`
- Windows 配置根目录：`C:\ProgramData\sdkwork-api-router\`

安装生成的 `router.env` 只是环境变量兜底层，适合承载：

- 配置目录和配置文件路径
- 数据库 URL 的兜底值
- 绑定地址的兜底值

release 载荷路径，例如当前激活的 router 二进制和 admin / portal 静态资源目录，来自 `current/release-manifest.json`，不应该再写入 `router.env`。

## 已安装 Desktop 产品默认目录

desktop 产品把可变运行时状态存放到 Tauri 提供的操作系统标准每用户目录中：

- 配置目录：
  - 应用 config 目录 + `router-product/`
- 数据目录：
  - 应用 data 目录 + `router-product/`
- 日志目录：
  - 应用 log 目录 + `router-product/`

desktop 专属文件：

- `desktop-runtime.json`
  - 保存桌面壳的访问模式
- `router.yaml`
  - `router-product-service` sidecar 的标准配置文件

desktop 壳会以 `--config-dir <config-root>` 启动 sidecar，并在拉起 sidecar 前清理继承的 `SDKWORK_*` 环境变量，确保配置文件在发现完成后保持权威性。

## 原始 Standalone 的本地默认配置根

如果不通过已安装的 server 产品，而是直接运行原始 standalone 二进制，默认本地配置根目录为：

- Linux / macOS：`~/.sdkwork/router/`
- Windows：`%USERPROFILE%\.sdkwork\router\`

该目录下会查找：

- `router.yaml`
- `router.yml`
- `router.json`
- `config.yaml`
- `config.yml`
- `config.json`
- `sdkwork-api-router.db`
- `secrets.json`
- `extensions/`

## 配置文件发现顺序

当没有显式设置 `SDKWORK_CONFIG_FILE` 时，运行时会按如下顺序查找：

1. `router.yaml`
2. `router.yml`
3. `router.json`
4. `config.yaml`
5. `config.yml`
6. `config.json`

命中的第一个文件就是主配置文件。

可通过以下变量覆盖：

- `SDKWORK_CONFIG_DIR`
- `SDKWORK_CONFIG_FILE`

如果 `SDKWORK_CONFIG_FILE` 是相对路径，则相对于 `SDKWORK_CONFIG_DIR` 或默认配置根目录解析。

`conf.d/*.{yaml,yml,json}` 中的受支持覆盖文件会在主配置之后按字典序叠加。

## 内建默认值

当没有配置文件时，服务仍会使用以下默认值启动：

- `gateway_bind`：`127.0.0.1:8080`
- `admin_bind`：`127.0.0.1:8081`
- `portal_bind`：`127.0.0.1:8082`
- `web_bind`：`127.0.0.1:3001`
- `database_url`：`sqlite://<config-root>/sdkwork-api-router.db`
- `cache_backend`：`memory`
- `cache_url`：未设置
- `extension_paths`：`["<config-root>/extensions"]`
- `secret_local_file`：`<config-root>/secrets.json`
- `enable_connector_extensions`：`true`
- `enable_native_dynamic_extensions`：`false`
- `secret_backend`：`database_encrypted`
- `secret_keyring_service`：`sdkwork-api-router`

这里的回环 `web_bind` 默认值适用于 native server 产品和原始 standalone 二进制在未提供配置文件、环境变量或 CLI 覆盖时的行为。desktop 的局域网共享模式，以及 Docker / Helm 这类面向远程访问的部署资产，会显式改用 `0.0.0.0:3001`。

## 配置文件字段

本地配置文件使用与 `StandaloneConfig` 对齐的顶层结构。

常用字段包括：

- `web_bind`
- `gateway_bind`
- `admin_bind`
- `portal_bind`
- `database_url`
- `cache_backend`
- `cache_url`
- `extension_paths`
- `admin_jwt_signing_secret`
- `portal_jwt_signing_secret`
- `bootstrap_data_dir`
- `bootstrap_profile`
- `secret_backend`
- `credential_master_key`
- `allow_insecure_dev_defaults`
- `metrics_bearer_token`
- `browser_allowed_origins`
- `secret_local_file`
- `secret_keyring_service`

## 路径解析规则

当值来自配置文件时：

- 相对 `secret_local_file` 路径相对于配置文件目录解析
- 相对 `extension_paths` 条目相对于配置文件目录解析
- 相对 SQLite 文件 URL 相对于配置文件目录解析，并规范化为绝对 SQLite URL

环境变量会先提供兜底值，再由配置文件覆盖。只要某个字段在活动配置文件集中被定义，该配置文件值就会生效，除非启动器显式传入 CLI 覆盖。

## 关键环境变量

常用运行时环境变量包括：

- `SDKWORK_CONFIG_DIR`
- `SDKWORK_CONFIG_FILE`
- `SDKWORK_DATABASE_URL`
- `SDKWORK_WEB_BIND`
- `SDKWORK_GATEWAY_BIND`
- `SDKWORK_ADMIN_BIND`
- `SDKWORK_PORTAL_BIND`
- `SDKWORK_CACHE_BACKEND`
- `SDKWORK_CACHE_URL`
- `SDKWORK_ADMIN_JWT_SIGNING_SECRET`
- `SDKWORK_PORTAL_JWT_SIGNING_SECRET`
- `SDKWORK_BOOTSTRAP_DATA_DIR`
- `SDKWORK_BOOTSTRAP_PROFILE`
- `SDKWORK_SECRET_BACKEND`
- `SDKWORK_CREDENTIAL_MASTER_KEY`
- `SDKWORK_ALLOW_INSECURE_DEV_DEFAULTS`
- `SDKWORK_METRICS_BEARER_TOKEN`
- `SDKWORK_BROWSER_ALLOWED_ORIGINS`
- `SDKWORK_SECRET_LOCAL_FILE`
- `SDKWORK_SECRET_KEYRING_SERVICE`
- `SDKWORK_SERVICE_INSTANCE_ID`

## 启动安全校验

运行时允许在纯本机回环绑定场景下保留开发默认值，但一旦任何服务绑定到非回环地址，就会执行更严格的安全校验。

规则：

- 纯回环绑定，例如 `127.0.0.1:8080`，允许保留内建开发默认值
- 如果 `gateway_bind`、`admin_bind` 或 `portal_bind` 中任意一个绑定到非回环地址，则默认拒绝本地开发 JWT、主密钥和 metrics token
- 只有显式设置 `SDKWORK_ALLOW_INSECURE_DEV_DEFAULTS=true` 时，才允许在非回环绑定下继续使用这些开发默认值

推荐的生产姿态：

- 设置强随机的 admin / portal JWT 密钥
- 使用非演示用的 credential master key
- 设置非默认的 metrics bearer token
- 保持 `SDKWORK_BOOTSTRAP_PROFILE=prod`
- 不要开启 `SDKWORK_ALLOW_INSECURE_DEV_DEFAULTS`

## 启动示例

Linux 或 macOS：

```bash
mkdir -p "$HOME/.sdkwork/router"
cat > "$HOME/.sdkwork/router/router.yaml" <<'EOF'
database_url: "sqlite://sdkwork-api-router.db"
secret_backend: "local_encrypted_file"
EOF

./target/release/gateway-service
```

Windows PowerShell：

```powershell
New-Item -ItemType Directory -Force "$HOME\.sdkwork\router" | Out-Null
@"
database_url: "sqlite://sdkwork-api-router.db"
secret_backend: "local_encrypted_file"
"@ | Set-Content -Encoding UTF8 "$HOME\.sdkwork\router\router.yaml"

.\target\release\gateway-service.exe
```
