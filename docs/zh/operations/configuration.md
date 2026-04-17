# 配置说明

本页定义 standalone 形态下 SDKWork API Router 的运行时配置契约。

## 解析顺序

普通字段的最终优先级从低到高如下：

- 内建默认值 -> 环境变量兜底 -> 配置文件 -> CLI

说明：

- `SDKWORK_CONFIG_DIR` 与 `SDKWORK_CONFIG_FILE` 是配置发现输入，运行时会先读取它们来定位配置文件。
- 一旦某个字段已经在 `router.yaml` 或 `conf.d/*.yaml` 中定义，对应配置文件的值优先。
- 环境变量只为配置文件没有定义的字段提供兜底值。
- `system` 安装模式默认使用 PostgreSQL；`portable` 和本地开发场景仍然可以使用 SQLite。

运行时热重载会继续使用进程启动时捕获的环境变量兜底快照。也就是说，修改 `router.yaml` 或 `conf.d/*.yaml` 可以触发可热更新字段的刷新，但服务启动后再去修改父 shell 中的环境变量，不会被已运行进程感知。

## 默认配置根目录

默认本地配置根目录如下：

- Linux / macOS：`~/.sdkwork/router/`
- Windows：`%USERPROFILE%\\.sdkwork\\router\\`

运行时会在该目录下推导以下默认路径：

- 主 YAML 配置：`router.yaml`
- 次 YAML 配置：`router.yml`
- 次 JSON 配置：`router.json`
- 兼容旧文件名的 YAML：`config.yaml`
- 兼容旧文件名的 YAML fallback：`config.yml`
- 兼容旧文件名的 JSON fallback：`config.json`
- 默认 SQLite 数据库：`sdkwork-api-router.db`
- 本地加密 secrets 文件：`secrets.json`
- 扩展目录：`extensions/`

## 配置文件发现顺序

当没有显式设置 `SDKWORK_CONFIG_FILE` 时，运行时按以下顺序查找：

1. `router.yaml`
2. `router.yml`
3. `router.json`
4. `config.yaml`
5. `config.yml`
6. `config.json`

命中的第一个文件作为主配置文件。

相关发现输入：

- `SDKWORK_CONFIG_DIR`
- `SDKWORK_CONFIG_FILE`

如果 `SDKWORK_CONFIG_FILE` 是相对路径，则会相对于 `SDKWORK_CONFIG_DIR` 或默认配置根目录进行解析。

## 内建默认值

当没有任何配置文件时，服务仍会以以下默认值启动：

- `gateway_bind`：`127.0.0.1:8080`
- `admin_bind`：`127.0.0.1:8081`
- `portal_bind`：`127.0.0.1:8082`
- `database_url`：`sqlite://<config-root>/sdkwork-api-router.db`
- `cache_backend`：`memory`
- `cache_url`：未设置
- `extension_paths`：`["<config-root>/extensions"]`
- `secret_local_file`：`<config-root>/secrets.json`
- `enable_connector_extensions`：`true`
- `enable_native_dynamic_extensions`：`false`
- `extension_hot_reload_interval_secs`：`0`
- `require_signed_connector_extensions`：`false`
- `require_signed_native_dynamic_extensions`：`true`
- `runtime_snapshot_interval_secs`：`0`
- `secret_backend`：`database_encrypted`
- `secret_keyring_service`：`sdkwork-api-router`

## 配置文件字段

本地配置文件使用与 `StandaloneConfig` 对齐的扁平顶层结构。常用字段包括：

- `gateway_bind`
- `admin_bind`
- `portal_bind`
- `database_url`
- `cache_backend`
- `cache_url`
- `extension_paths`
- `enable_connector_extensions`
- `enable_native_dynamic_extensions`
- `extension_hot_reload_interval_secs`
- `extension_trusted_signers`
- `require_signed_connector_extensions`
- `require_signed_native_dynamic_extensions`
- `admin_jwt_signing_secret`
- `portal_jwt_signing_secret`
- `bootstrap_data_dir`
- `bootstrap_profile`
- `runtime_snapshot_interval_secs`
- `secret_backend`
- `credential_master_key`
- `allow_insecure_dev_defaults`
- `metrics_bearer_token`
- `browser_allowed_origins`
- `credential_legacy_master_keys`
- `secret_local_file`
- `secret_keyring_service`

## Cache 后端

当前配置契约支持 `cache_backend` 与可选的 `cache_url`：

- `memory`：适用于单进程、本地开发或测试
- `redis`：适用于多进程、共享缓存场景

目前缓存消费者的启用方式并不完全相同：

- route-decision cache 会在 standalone gateway 中启用
- capability catalog cache 只有在运行时明确注入共享缓存存储时才会启用
- standalone `gateway-service` 与 `admin-api-service` 只有在后端具备跨进程一致性时才会启用 capability catalog cache，目前即 `redis`

## 热重载行为

三个 standalone 服务都会每秒轮询一次当前解析出的配置文件集合。

可无重启热更新的字段：

- `extension_paths`
- `enable_connector_extensions`
- `enable_native_dynamic_extensions`
- `extension_trusted_signers`
- `require_signed_connector_extensions`
- `require_signed_native_dynamic_extensions`
- `extension_hot_reload_interval_secs`
- `runtime_snapshot_interval_secs`
- `database_url`
- `admin_jwt_signing_secret`
- `portal_jwt_signing_secret`
- `gateway_bind`
- `admin_bind`
- `portal_bind`
- `secret_backend`
- `credential_master_key`
- `credential_legacy_master_keys`
- `secret_local_file`
- `secret_keyring_service`

仍然需要重启的字段：

- `cache_backend`
- `cache_url`
- 仅在父 shell 中修改、但服务启动后才发生变化的环境变量
- 二进制升级及其他进程外发布动作

如果配置文件同时包含可热更新字段和需重启字段，运行时会立即应用可热更新部分，并将需重启的部分标记为 pending。

## YAML 示例

```yaml
gateway_bind: "127.0.0.1:8080"
admin_bind: "127.0.0.1:8081"
portal_bind: "127.0.0.1:8082"
database_url: "sqlite://sdkwork-api-router.db"
cache_backend: "memory"
extension_paths:
  - "extensions"
  - "extensions/partner"
enable_connector_extensions: true
enable_native_dynamic_extensions: false
extension_hot_reload_interval_secs: 5
extension_trusted_signers:
  sdkwork: "ZXhwaWNpdC1wdWJsaWMta2V5"
  partner: "c2Vjb25kLXB1YmxpYy1rZXk="
require_signed_connector_extensions: false
require_signed_native_dynamic_extensions: true
admin_jwt_signing_secret: "change-me-admin"
portal_jwt_signing_secret: "change-me-portal"
runtime_snapshot_interval_secs: 30
secret_backend: "local_encrypted_file"
credential_master_key: "change-me-master-key"
secret_local_file: "secrets.json"
secret_keyring_service: "sdkwork-api-router"
```

## JSON 示例

```json
{
  "gateway_bind": "127.0.0.1:8080",
  "admin_bind": "127.0.0.1:8081",
  "portal_bind": "127.0.0.1:8082",
  "database_url": "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_router",
  "cache_backend": "memory",
  "extension_paths": [
    "extensions"
  ],
  "enable_connector_extensions": true,
  "enable_native_dynamic_extensions": false,
  "secret_backend": "database_encrypted",
  "secret_local_file": "secrets.json"
}
```

## 路径解析规则

当值来自配置文件时：

- 相对 `secret_local_file` 会相对于配置文件所在目录解析
- 相对 `extension_paths` 条目会相对于配置文件所在目录解析
- 相对 SQLite 文件 URL 会相对于配置文件所在目录解析，并被规范化为绝对 SQLite URL

示例：

- 配置文件：`~/.sdkwork/router/router.yaml`
- `database_url: "sqlite://router.db"`
- 运行时解析结果：`sqlite://~/.sdkwork/router/router.db`

## 常用环境变量

- `SDKWORK_CONFIG_DIR`
- `SDKWORK_CONFIG_FILE`
- `SDKWORK_DATABASE_URL`
- `SDKWORK_CACHE_BACKEND`
- `SDKWORK_CACHE_URL`
- `SDKWORK_GATEWAY_BIND`
- `SDKWORK_ADMIN_BIND`
- `SDKWORK_PORTAL_BIND`
- `SDKWORK_ADMIN_JWT_SIGNING_SECRET`
- `SDKWORK_PORTAL_JWT_SIGNING_SECRET`
- `SDKWORK_BOOTSTRAP_DATA_DIR`
- `SDKWORK_BOOTSTRAP_PROFILE`
- `SDKWORK_SECRET_BACKEND`
- `SDKWORK_CREDENTIAL_MASTER_KEY`
- `SDKWORK_ALLOW_INSECURE_DEV_DEFAULTS`
- `SDKWORK_METRICS_BEARER_TOKEN`
- `SDKWORK_BROWSER_ALLOWED_ORIGINS`
- `SDKWORK_CREDENTIAL_LEGACY_MASTER_KEYS`
- `SDKWORK_SECRET_LOCAL_FILE`
- `SDKWORK_SECRET_KEYRING_SERVICE`
- `SDKWORK_SERVICE_INSTANCE_ID`
- `SDKWORK_EXTENSION_PATHS`
- `SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS`
- `SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS`
- `SDKWORK_EXTENSION_HOT_RELOAD_INTERVAL_SECS`
- `SDKWORK_EXTENSION_TRUSTED_SIGNERS`
- `SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS`
- `SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS`
- `SDKWORK_RUNTIME_SNAPSHOT_INTERVAL_SECS`

## 启动安全校验

standalone 运行时允许 loopback-only 的本地开发默认值，但当任一服务绑定到非 loopback 地址时，除非显式设置 `SDKWORK_ALLOW_INSECURE_DEV_DEFAULTS=true`，否则会拒绝继续启动。

推荐的生产姿态：

- 为 admin 和 portal JWT 配置强且唯一的签名密钥
- 配置非演示用途的 `credential_master_key`
- 配置单独的 metrics bearer token
- 默认保持 `SDKWORK_BOOTSTRAP_PROFILE=prod`

## 集群协调

standalone 服务会向 admin store 上报共享协调身份：

- 如果设置了 `SDKWORK_SERVICE_INSTANCE_ID`，它会作为持久节点 ID
- 否则运行时会基于服务类型、进程 ID 与启动时间合成一个节点 ID
- `POST /admin/extensions/runtime-rollouts` 会面向活跃的 gateway 与 admin 节点
- `POST /admin/runtime-config/rollouts` 会面向活跃的 gateway、admin、portal 节点

## 启动示例

Linux / macOS：

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
New-Item -ItemType Directory -Force "$HOME\\.sdkwork\\router" | Out-Null
@"
database_url: "sqlite://sdkwork-api-router.db"
secret_backend: "local_encrypted_file"
"@ | Set-Content -Encoding UTF8 "$HOME\\.sdkwork\\router\\router.yaml"

.\target\release\gateway-service.exe
```

显式指定配置文件：

```bash
export SDKWORK_CONFIG_FILE="$HOME/.sdkwork/router/router.json"
./target/release/gateway-service
```
