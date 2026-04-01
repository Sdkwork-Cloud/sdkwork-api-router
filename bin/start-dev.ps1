param(
    [switch]$Foreground,
    [switch]$DryRun,
    [int]$WaitSeconds = 60,
    [switch]$Install,
    [switch]$Browser,
    [switch]$Preview,
    [switch]$Tauri,
    [string]$DatabaseUrl = '',
    [string]$GatewayBind = '',
    [string]$AdminBind = '',
    [string]$PortalBind = '',
    [string]$WebBind = ''
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

. (Join-Path (Split-Path -Parent $PSCommandPath) 'lib\runtime-common.ps1')

$scriptDir = Split-Path -Parent $PSCommandPath
$repoRoot = Get-RouterRepoRoot -ScriptDirectory $scriptDir
$devHome = Get-RouterDefaultDevHome -RepoRoot $repoRoot
$configDirectory = Join-Path $devHome 'config'
$dataDirectory = Join-Path $devHome 'data'
$logDirectory = Join-Path $devHome 'log'
$runDirectory = Join-Path $devHome 'run'
$envFile = Join-Path $configDirectory 'router-dev.env'
$pidFile = Join-Path $runDirectory 'start-workspace.pid'
$stdoutLog = Join-Path $logDirectory 'start-workspace.stdout.log'
$stderrLog = Join-Path $logDirectory 'start-workspace.stderr.log'
$planFile = Join-Path $runDirectory 'start-workspace.plan.txt'

Ensure-RouterDirectory -DirectoryPath $configDirectory
Ensure-RouterDirectory -DirectoryPath $dataDirectory
Ensure-RouterDirectory -DirectoryPath $logDirectory
Ensure-RouterDirectory -DirectoryPath $runDirectory

Import-RouterEnvFile -EnvFile $envFile

if (-not $env:SDKWORK_DATABASE_URL) {
    $env:SDKWORK_DATABASE_URL = "sqlite://$(Convert-ToRouterPortablePath -PathValue $dataDirectory)/sdkwork-api-router-dev.db"
}
if (-not $env:SDKWORK_GATEWAY_BIND) {
    $env:SDKWORK_GATEWAY_BIND = '127.0.0.1:9980'
}
if (-not $env:SDKWORK_ADMIN_BIND) {
    $env:SDKWORK_ADMIN_BIND = '127.0.0.1:9981'
}
if (-not $env:SDKWORK_PORTAL_BIND) {
    $env:SDKWORK_PORTAL_BIND = '127.0.0.1:9982'
}
if (-not $env:SDKWORK_WEB_BIND) {
    $env:SDKWORK_WEB_BIND = '127.0.0.1:9983'
}

if ($DatabaseUrl) { $env:SDKWORK_DATABASE_URL = $DatabaseUrl }
if ($GatewayBind) { $env:SDKWORK_GATEWAY_BIND = $GatewayBind }
if ($AdminBind) { $env:SDKWORK_ADMIN_BIND = $AdminBind }
if ($PortalBind) { $env:SDKWORK_PORTAL_BIND = $PortalBind }
if ($WebBind) { $env:SDKWORK_WEB_BIND = $WebBind }

if ($Browser) {
    $Preview = $false
    $Tauri = $false
} elseif ($Tauri) {
    $Preview = $false
} elseif (-not $Preview) {
    $Preview = $true
}

$workspaceLauncher = Join-Path $repoRoot 'scripts\dev\start-workspace.mjs'
Assert-RouterFileExists -Label 'workspace launcher' -FilePath $workspaceLauncher

$adminNodeModules = Join-Path $repoRoot 'apps\sdkwork-router-admin\node_modules'
$portalNodeModules = Join-Path $repoRoot 'apps\sdkwork-router-portal\node_modules'
if ($Install -or -not (Test-Path $adminNodeModules) -or -not (Test-Path $portalNodeModules)) {
    $Install = $true
}

$startArgs = @(
    'scripts/dev/start-workspace.mjs',
    '--database-url', $env:SDKWORK_DATABASE_URL,
    '--gateway-bind', $env:SDKWORK_GATEWAY_BIND,
    '--admin-bind', $env:SDKWORK_ADMIN_BIND,
    '--portal-bind', $env:SDKWORK_PORTAL_BIND,
    '--web-bind', $env:SDKWORK_WEB_BIND
)

if ($Install) { $startArgs += '--install' }
if ($Preview) { $startArgs += '--preview' }
if ($Tauri) { $startArgs += '--tauri' }

$planArgs = @($startArgs + '--dry-run')

Push-Location $repoRoot
try {
    $planOutput = & node @planArgs
    Set-Content -Path $planFile -Value $planOutput -Encoding utf8

    if ($DryRun) {
        Get-Content $planFile
        return
    }

    if ($Foreground) {
        & node @startArgs
        return
    }

    Assert-RouterNotRunning -PidFile $pidFile
    if (Test-Path $stdoutLog) { Remove-Item $stdoutLog -Force }
    if (Test-Path $stderrLog) { Remove-Item $stderrLog -Force }

    $process = Start-Process `
        -FilePath node `
        -ArgumentList $startArgs `
        -WorkingDirectory $repoRoot `
        -RedirectStandardOutput $stdoutLog `
        -RedirectStandardError $stderrLog `
        -PassThru

    Set-Content -Path $pidFile -Value $process.Id -Encoding utf8

    $gatewayHealthUrl = Resolve-RouterHealthUrl -BindAddress $env:SDKWORK_GATEWAY_BIND -PathSuffix '/health'
    $adminHealthUrl = Resolve-RouterHealthUrl -BindAddress $env:SDKWORK_ADMIN_BIND -PathSuffix '/admin/health'
    $portalHealthUrl = Resolve-RouterHealthUrl -BindAddress $env:SDKWORK_PORTAL_BIND -PathSuffix '/portal/health'

    $backendReady = (Wait-RouterHealthUrl -Url $gatewayHealthUrl -WaitSeconds $WaitSeconds) `
        -and (Wait-RouterHealthUrl -Url $adminHealthUrl -WaitSeconds $WaitSeconds) `
        -and (Wait-RouterHealthUrl -Url $portalHealthUrl -WaitSeconds $WaitSeconds)

    if (-not $backendReady) {
        Stop-RouterProcessTree -Pid $process.Id -WaitSeconds $WaitSeconds -Force | Out-Null
        Remove-Item $pidFile -Force -ErrorAction SilentlyContinue
        Show-RouterLogTail -LogFile $stdoutLog
        Show-RouterLogTail -LogFile $stderrLog
        Throw-RouterError 'development services failed health checks'
    }

    if ($Preview -or $Tauri) {
        $adminSurfaceUrl = Resolve-RouterHealthUrl -BindAddress $env:SDKWORK_WEB_BIND -PathSuffix '/admin/'
        $portalSurfaceUrl = Resolve-RouterHealthUrl -BindAddress $env:SDKWORK_WEB_BIND -PathSuffix '/portal/'
    } else {
        $adminSurfaceUrl = 'http://127.0.0.1:5173/admin/'
        $portalSurfaceUrl = 'http://127.0.0.1:5174/portal/'
    }

    $webReady = (Wait-RouterHealthUrl -Url $adminSurfaceUrl -WaitSeconds $WaitSeconds) `
        -and (Wait-RouterHealthUrl -Url $portalSurfaceUrl -WaitSeconds $WaitSeconds)

    if (-not $webReady) {
        Stop-RouterProcessTree -Pid $process.Id -WaitSeconds $WaitSeconds -Force | Out-Null
        Remove-Item $pidFile -Force -ErrorAction SilentlyContinue
        Show-RouterLogTail -LogFile $stdoutLog
        Show-RouterLogTail -LogFile $stderrLog
        Throw-RouterError 'development web surfaces failed health checks'
    }

    Write-RouterInfo "started development workspace (pid=$($process.Id))"
    $mode = if ($Tauri) {
        'development tauri'
    } elseif ($Preview) {
        'development preview'
    } else {
        'development browser'
    }

    Write-RouterStartupSummary `
        -Mode $mode `
        -WebBind $env:SDKWORK_WEB_BIND `
        -GatewayBind $env:SDKWORK_GATEWAY_BIND `
        -AdminBind $env:SDKWORK_ADMIN_BIND `
        -PortalBind $env:SDKWORK_PORTAL_BIND `
        -UnifiedAccessEnabled ($Preview -or $Tauri) `
        -AdminAppUrl $adminSurfaceUrl `
        -PortalAppUrl $portalSurfaceUrl `
        -StdoutLog $stdoutLog `
        -StderrLog $stderrLog
}
finally {
    Pop-Location
}
