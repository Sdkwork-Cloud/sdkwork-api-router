# 服务管理

本页定义已安装 server 产品的 service-manager 标准契约。

它不适用于 `sdkwork-router-portal-desktop`。desktop 产品是按用户运行的 Tauri 桌面壳，负责监管随包分发的 `router-product-service` sidecar，并由应用自身管理访问模式，而不是注册为操作系统后台服务。

Release 工作流中的 `installed-runtime smoke` 会验证同一份 `packaged server bundle`，也就是说，service-manager 契约只针对正式发布包验证，而不是针对原始工作区输出验证。
首次安装应当在解压正式 archive 之后，通过 bundle 根目录的 `install.sh` 或 `install.ps1` 完成。

## 支持的服务管理器

- Linux：`systemd`
- macOS：`launchd`
- Windows：Windows Service Control Manager

`current/service/windows-task/` 仍然保留为 compatibility asset，但正式的 Windows 生产路径是 `current/service/windows-service/`。

## 控制路径

从 product root 观察：

- current control directory：`./current/`
- 配置校验入口：`./current/bin/validate-config.sh`
- 前台启动入口：`./current/bin/start.sh`
- 停止入口：`./current/bin/stop.sh`
- support export 入口：`./current/bin/support-bundle.sh`

PowerShell 等价入口位于 `current/bin/` 下的同名路径，包括 `.\current\bin\support-bundle.ps1`。
这些已安装入口都从正式 `packaged server bundle` 内嵌的 `control/bin/` 子树物化生成；仓库中的 `bin/*` 工具只作为源码检出时的兜底入口。

## 启动前校验

在注册或重启生产服务之前，先对 product root 执行已安装的校验入口。

Linux 或 macOS：

```bash
./current/bin/validate-config.sh --home <product-root>
```

Windows PowerShell：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\current\bin\validate-config.ps1 -Home <product-root>
```

如果是在源码仓库中操作，仍可以使用 source-side fallback：

```bash
node bin/router-ops.mjs validate-config --mode system --home <product-root>
```

```powershell
node .\bin\router-ops.mjs validate-config --mode system --home <product-root>
```

## 前台运行契约

service manager 必须以前台模式启动进程，并把已安装的 product root 作为目标 home：

- `./current/bin/start.sh --foreground --home <product-root>`
- `powershell -NoProfile -ExecutionPolicy Bypass -File .\current\bin\start.ps1 -Foreground -Home <product-root>`

生成的 service 资产已经遵循这一契约。

## Linux: systemd

生成资产：

- `current/service/systemd/sdkwork-api-router.service`
- `current/service/systemd/install-service.sh`
- `current/service/systemd/uninstall-service.sh`

从 product root 的典型生命周期：

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

从 product root 的典型生命周期：

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

从 product root 的典型生命周期：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\current\service\windows-service\install-service.ps1
Get-Service sdkwork-api-router
powershell -NoProfile -ExecutionPolicy Bypass -File .\current\service\windows-service\uninstall-service.ps1
```

## 运维注意事项

- 只允许把可变状态写入文档规定的 `config/`、`data/`、`log/` 和 `run/` 目录。
- 安装完成后不要修改 `releases/<version>/`。
- 把 `current/release-manifest.json` 视为生成的控制元数据。
- 把 `releases/<version>/` 视为从官方 `packaged server bundle` 解包得到的版本化只读载荷。
- 每次升级前都检查 `router.yaml` 和 `router.env`。
- 每次配置变更后都重新执行 `validate-config`。
- 需要导出 release-safe operator diagnostics bundle 时，优先执行 `support-bundle.*`。
- `start.* --dry-run` 和产品服务自身的 `--dry-run` 结果都只是预检输出，不能替代正式的服务注册。
