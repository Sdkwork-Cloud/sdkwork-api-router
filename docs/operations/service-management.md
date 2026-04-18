# Service Management

This page defines the supported service-manager contract for the installed server product.

It does not apply to `sdkwork-router-portal-desktop`. The desktop product is a per-user Tauri shell that supervises a bundled `router-product-service` sidecar and manages access mode from the app itself instead of registering an OS background service.

Release workflow `installed-runtime smoke` verifies the same packaged server bundle that native install tooling consumes.
In other words, the service-manager contract is validated only against the official `packaged server bundle`, not against raw workspace build outputs.

## Supported Managers

- Linux: `systemd`
- macOS: `launchd`
- Windows: Windows Service Control Manager

`current/service/windows-task/` remains a compatibility asset. The formal Windows production path is `current/service/windows-service/`.

## Control Paths

From the product root:

- current control home: `./current/`
- validation entrypoint: `./current/bin/validate-config.sh`
- foreground start entrypoint: `./current/bin/start.sh`
- stop entrypoint: `./current/bin/stop.sh`

PowerShell equivalents live beside the shell entrypoints under `current/bin/`.

## Pre-Start Validation

Before registering or restarting a production service, run the installed validation entrypoint against the current control home.

Linux or macOS:

```bash
./current/bin/validate-config.sh --home ./current
```

Windows PowerShell:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\current\bin\validate-config.ps1 -Home .\current
```

From a repository checkout, the source-side fallback remains:

```bash
node bin/router-ops.mjs validate-config --mode system --home <product-root>
```

```powershell
node .\bin\router-ops.mjs validate-config --mode system --home <product-root>
```

## Foreground Runtime Contract

Service managers must execute the runtime in foreground mode from the current control home:

- `./current/bin/start.sh --foreground --home <product-root>/current`
- `powershell -NoProfile -ExecutionPolicy Bypass -File .\current\bin\start.ps1 -Foreground -Home <product-root>\current`

The generated service assets already follow this contract.

## Linux: systemd

Generated assets:

- `current/service/systemd/sdkwork-api-router.service`
- `current/service/systemd/install-service.sh`
- `current/service/systemd/uninstall-service.sh`

Typical lifecycle from the product root:

```bash
./current/service/systemd/install-service.sh
systemctl status sdkwork-api-router
./current/service/systemd/uninstall-service.sh
```

## macOS: launchd

Generated assets:

- `current/service/launchd/com.sdkwork.api-router.plist`
- `current/service/launchd/install-service.sh`
- `current/service/launchd/uninstall-service.sh`

Typical lifecycle from the product root:

```bash
./current/service/launchd/install-service.sh
sudo launchctl print system/com.sdkwork.api-router
./current/service/launchd/uninstall-service.sh
```

## Windows Service

Generated assets:

- `current/service/windows-service/run-service.ps1`
- `current/service/windows-service/install-service.ps1`
- `current/service/windows-service/uninstall-service.ps1`

Typical lifecycle from the product root:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\current\service\windows-service\install-service.ps1
Get-Service sdkwork-api-router
powershell -NoProfile -ExecutionPolicy Bypass -File .\current\service\windows-service\uninstall-service.ps1
```

## Operational Notes

- Keep mutable state under the documented `config/`, `data/`, `log/`, and `run/` roots only.
- Do not edit `releases/<version>/` after installation.
- Treat `current/release-manifest.json` as generated control metadata.
- Treat the versioned payload under `releases/<version>/` as content copied from the official packaged server bundle.
- Review `router.yaml` and `router.env` before each upgrade.
- Re-run `validate-config` after every config change.
- Treat `start.* --dry-run` and the product-service `--dry-run` plan output as preflight checks, not as a replacement for service registration.
