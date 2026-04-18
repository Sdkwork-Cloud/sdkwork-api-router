# 服务管理

本页定义已安装 server 产品的 service-manager 标准。

它不适用于 `sdkwork-router-portal-desktop`。desktop 产品是一个每用户 Tauri 桌面壳，负责托管随包分发的 `router-product-service` sidecar，并通过应用内设置管理访问模式，而不是向操作系统注册后台服务。

Release 流程里的 `installed-runtime smoke` 会安装并校验同一份 `packaged server bundle`。
也就是说，service-manager 契约只针对正式打包产物验证，而不是针对源码工作区输出验证。

## 支持的服务管理器

- Linux：`systemd`
- macOS：`launchd`
- Windows：Windows Service Control Manager

`current/service/windows-task/` 继续保留为兼容性资产，但正式的 Windows 生产路径是 `current/service/windows-service/`。

## 控制路径

从产品根目录看：

- `current/`：稳定控制目录
- `./current/bin/validate-config.sh`：配置校验入口
- `./current/bin/start.sh`：前台启动入口
- `./current/bin/stop.sh`：停止入口

PowerShell 对应入口位于 `current/bin/` 下的同名位置。

## 启动前校验

在注册或重启生产服务之前，先针对当前控制目录执行配置校验。

Linux 或 macOS：

```bash
./current/bin/validate-config.sh --home ./current
```

Windows PowerShell：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\current\bin\validate-config.ps1 -Home .\current
```

在源码仓库中，也可以使用：

```bash
node bin/router-ops.mjs validate-config --mode system --home <product-root>
```

```powershell
node .\bin\router-ops.mjs validate-config --mode system --home <product-root>
```

## 前台运行契约

service manager 必须以“前台模式”从 `current` 控制目录启动运行时：

- `./current/bin/start.sh --foreground --home <product-root>/current`
- `powershell -NoProfile -ExecutionPolicy Bypass -File .\current\bin\start.ps1 -Foreground -Home <product-root>\current`

生成出来的 service 资产已经遵循该契约。

## Linux: systemd

生成资产：

- `current/service/systemd/sdkwork-api-router.service`
- `current/service/systemd/install-service.sh`
- `current/service/systemd/uninstall-service.sh`

典型流程：

```bash
./current/service/systemd/install-service.sh
systemctl status sdkwork-api-router
./current/service/systemd/uninstall-service.sh
```

## macOS: launchd

生成资产：

- `current/service/launchd/com.sdkwork.api-router.plist`
- `current/service/launchd/install-service.sh`
- `current/service/launchd/uninstall-service.sh`

典型流程：

```bash
./current/service/launchd/install-service.sh
sudo launchctl print system/com.sdkwork.api-router
./current/service/launchd/uninstall-service.sh
```

## Windows Service

生成资产：

- `current/service/windows-service/run-service.ps1`
- `current/service/windows-service/install-service.ps1`
- `current/service/windows-service/uninstall-service.ps1`

典型流程：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\current\service\windows-service\install-service.ps1
Get-Service sdkwork-api-router
powershell -NoProfile -ExecutionPolicy Bypass -File .\current\service\windows-service\uninstall-service.ps1
```

## 运维注意事项

- 只允许把可变状态写到 `config/`、`data/`、`log/` 和 `run/`。
- 不要修改 `releases/<version>/` 中的内容。
- 把 `current/release-manifest.json` 视为生成的控制元数据。
- 把 `releases/<version>/` 视为从官方 `packaged server bundle` 解包得到的只读版本化载荷。
- 每次升级前都要检查 `router.yaml` 和 `router.env`。
- 每次配置变更后都要重新执行 `validate-config`。
- `start.* --dry-run` 与产品服务的 `--dry-run` 结果只用于预检，不能替代正式服务注册。
