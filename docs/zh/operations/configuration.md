# 配置说明

本页定义 SDKWork API Server standalone 服务的运行时配置约定。

## 解析顺序

运行时配置的合并顺序为：

1. 内置本地默认值
2. 本地配置文件
3. `SDKWORK_*` 环境变量

这意味着环境变量始终覆盖 `config.yaml`、`config.yml` 或 `config.json` 中的值。

运行中的配置文件重载会继续使用进程启动时捕获的原始环境变量覆盖集。因此，服务启动后编辑 `config.yaml` 可以作用于下文列出的可热更新字段，但在父 shell 里再修改 `SDKWORK_*` 环境变量不会被已运行进程感知。

## 默认本地配置根目录

默认本地配置根目录为：

- Linux / macOS：`~/.sdkwork/router/`
- Windows：`%USERPROFILE%\\.sdkwork\\router\\`

服务会在该目录下推导以下默认路径：

- 主 YAML 配置：`config.yaml`
- 备用 YAML 配置：`config.yml`
- 备用 JSON 配置：`config.json`
- 默认 SQLite 数据库：`sdkwork-api-server.db`
- 本地加密密钥文件：`secrets.json`
- 扩展目录：`extensions/`

## 配置文件查找顺序

当没有设置 `SDKWORK_CONFIG_FILE` 时，运行时按以下顺序搜索：

1. `config.yaml`
2. `config.yml`
3. `config.json`

命中的第一个文件生效。

覆盖配置根目录：

- `SDKWORK_CONFIG_DIR`

直接指定配置文件：

- `SDKWORK_CONFIG_FILE`

如果 `SDKWORK_CONFIG_FILE` 是相对路径，则会相对于 `SDKWORK_CONFIG_DIR` 或默认配置根目录进行解析。

## 内置默认值

即使没有任何配置文件，服务仍会使用以下默认值启动：

- `gateway_bind`：`127.0.0.1:8080`
- `admin_bind`：`127.0.0.1:8081`
- `portal_bind`：`127.0.0.1:8082`
- `database_url`：`sqlite://<config-root>/sdkwork-api-server.db`
- `extension_paths`：`["<config-root>/extensions"]`
- `secret_local_file`：`<config-root>/secrets.json`
- `enable_connector_extensions`：`true`
- `enable_native_dynamic_extensions`：`false`
- `extension_hot_reload_interval_secs`：`0`
- `require_signed_connector_extensions`：`false`
- `require_signed_native_dynamic_extensions`：`true`
- `runtime_snapshot_interval_secs`：`0`
- `secret_backend`：`database_encrypted`
- `secret_keyring_service`：`sdkwork-api-server`

## 文件结构

本地配置文件采用与 `StandaloneConfig` 一致的扁平顶层结构。

支持的字段：

- `gateway_bind`
- `admin_bind`
- `portal_bind`
- `database_url`
- `extension_paths`
- `enable_connector_extensions`
- `enable_native_dynamic_extensions`
- `extension_hot_reload_interval_secs`
- `extension_trusted_signers`
- `require_signed_connector_extensions`
- `require_signed_native_dynamic_extensions`
- `admin_jwt_signing_secret`
- `portal_jwt_signing_secret`
- `runtime_snapshot_interval_secs`
- `secret_backend`
- `credential_master_key`
- `secret_local_file`
- `secret_keyring_service`

## 运行时重载行为

`gateway-service` 与 `admin-api-service` 会以 1 秒轮询一次当前解析出的配置文件集合。

无需重启即可生效：

- `extension_paths`
- `enable_connector_extensions`
- `enable_native_dynamic_extensions`
- `extension_trusted_signers`
- `require_signed_connector_extensions`
- `require_signed_native_dynamic_extensions`
- `extension_hot_reload_interval_secs`
- `runtime_snapshot_interval_secs`

仍然需要重启：

- `gateway_bind`
- `admin_bind`
- `portal_bind`
- `database_url`
- `admin_jwt_signing_secret`
- `portal_jwt_signing_secret`
- `secret_backend`
- `credential_master_key`
- `secret_local_file`
- `secret_keyring_service`

当磁盘上的变更涉及这些需重启字段时，运行中进程会记录“已检测到但需重启后才会应用”的日志。

`portal-api-service` 目前仍只在启动时读取配置。

## YAML 示例

```yaml
gateway_bind: "127.0.0.1:8080"
admin_bind: "127.0.0.1:8081"
portal_bind: "127.0.0.1:8082"
database_url: "sqlite://sdkwork-api-server.db"
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
secret_keyring_service: "sdkwork-api-server"
```

## JSON 示例

```json
{
  "gateway_bind": "127.0.0.1:8080",
  "admin_bind": "127.0.0.1:8081",
  "portal_bind": "127.0.0.1:8082",
  "database_url": "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server",
  "extension_paths": [
    "extensions"
  ],
  "enable_connector_extensions": true,
  "enable_native_dynamic_extensions": false,
  "secret_backend": "database_encrypted",
  "secret_local_file": "secrets.json"
}
```

## 路径归一化规则

当值来自配置文件时：

- 相对 `secret_local_file` 会按配置文件所在目录解析
- 相对 `extension_paths` 条目会按配置文件所在目录解析
- 相对 SQLite 文件 URL 会按配置文件所在目录解析，并被规范化为绝对 SQLite URL

示例：

- 配置文件：`~/.sdkwork/router/config.yaml`
- `database_url: "sqlite://router.db"`
- 运行时解析结果：`sqlite://~/.sdkwork/router/router.db`

环境变量在文件加载之后生效，并按原值覆盖。

## 环境变量

最重要的运行时环境变量包括：

- `SDKWORK_CONFIG_DIR`
- `SDKWORK_CONFIG_FILE`
- `SDKWORK_DATABASE_URL`
- `SDKWORK_GATEWAY_BIND`
- `SDKWORK_ADMIN_BIND`
- `SDKWORK_PORTAL_BIND`
- `SDKWORK_ADMIN_JWT_SIGNING_SECRET`
- `SDKWORK_PORTAL_JWT_SIGNING_SECRET`
- `SDKWORK_SECRET_BACKEND`
- `SDKWORK_CREDENTIAL_MASTER_KEY`
- `SDKWORK_SECRET_LOCAL_FILE`
- `SDKWORK_SECRET_KEYRING_SERVICE`
- `SDKWORK_EXTENSION_PATHS`
- `SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS`
- `SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS`
- `SDKWORK_EXTENSION_HOT_RELOAD_INTERVAL_SECS`
- `SDKWORK_EXTENSION_TRUSTED_SIGNERS`
- `SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS`
- `SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS`
- `SDKWORK_RUNTIME_SNAPSHOT_INTERVAL_SECS`

## 启动示例

Linux / macOS：

```bash
mkdir -p "$HOME/.sdkwork/router"
cat > "$HOME/.sdkwork/router/config.yaml" <<'EOF'
database_url: "sqlite://sdkwork-api-server.db"
secret_backend: "local_encrypted_file"
EOF

./target/release/gateway-service
```

Windows PowerShell：

```powershell
New-Item -ItemType Directory -Force "$HOME\\.sdkwork\\router" | Out-Null
@"
database_url: "sqlite://sdkwork-api-server.db"
secret_backend: "local_encrypted_file"
"@ | Set-Content -Encoding UTF8 "$HOME\\.sdkwork\\router\\config.yaml"

.\target\release\gateway-service.exe
```

显式指定配置文件：

```bash
export SDKWORK_CONFIG_FILE="$HOME/.sdkwork/router/config.json"
./target/release/gateway-service
```
