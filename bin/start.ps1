param(
    [Alias('Home')]
    [string]$RuntimeHome = '',
    [switch]$Foreground,
    [switch]$DryRun,
    [int]$WaitSeconds = 60,
    [string]$Bind = '',
    [string]$ConfigDir = '',
    [string]$ConfigFile = '',
    [string]$DatabaseUrl = '',
    [string]$Roles = '',
    [string]$NodeIdPrefix = '',
    [string]$GatewayBind = '',
    [string]$AdminBind = '',
    [string]$PortalBind = '',
    [string]$GatewayUpstream = '',
    [string]$AdminUpstream = '',
    [string]$PortalUpstream = '',
    [string]$AdminSiteDir = '',
    [string]$PortalSiteDir = ''
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

. (Join-Path (Split-Path -Parent $PSCommandPath) 'lib\runtime-common.ps1')

$scriptDir = Split-Path -Parent $PSCommandPath
$repoRoot = Get-RouterRepoRoot -ScriptDirectory $scriptDir
$defaultHome = Get-RouterDefaultInstallHome -RepoRoot $repoRoot
$binaryName = Get-RouterBinaryName -BaseName 'router-product-service'

if ([string]::IsNullOrWhiteSpace($RuntimeHome)) {
    $siblingBinary = Join-Path $scriptDir $binaryName
    if (Test-Path $siblingBinary) {
        $RuntimeHome = Split-Path -Parent $scriptDir
    } else {
        $RuntimeHome = $defaultHome
    }
}

$runtimeHome = if (Test-Path $RuntimeHome) { (Resolve-Path $RuntimeHome).Path } else { $RuntimeHome }
$binDir = Join-Path $runtimeHome 'bin'
$binaryPath = Join-Path $binDir $binaryName
$configDirectory = Join-Path $runtimeHome 'config'
$varDirectory = Join-Path $runtimeHome 'var'
$dataDirectory = Join-Path $varDirectory 'data'
$logDirectory = Join-Path $varDirectory 'log'
$runDirectory = Join-Path $varDirectory 'run'
$envFile = Join-Path $configDirectory 'router.env'
$pidFile = Join-Path $runDirectory 'router-product-service.pid'
$stdoutLog = Join-Path $logDirectory 'router-product-service.stdout.log'
$stderrLog = Join-Path $logDirectory 'router-product-service.stderr.log'
$planFile = Join-Path $runDirectory 'router-product-service.plan.json'
$defaultAdminSiteDir = Join-Path $runtimeHome 'sites\admin\dist'
$defaultPortalSiteDir = Join-Path $runtimeHome 'sites\portal\dist'

Ensure-RouterDirectory -DirectoryPath $configDirectory
Ensure-RouterDirectory -DirectoryPath $dataDirectory
Ensure-RouterDirectory -DirectoryPath $logDirectory
Ensure-RouterDirectory -DirectoryPath $runDirectory

Import-RouterEnvFile -EnvFile $envFile

if (-not $env:SDKWORK_ROUTER_BINARY) {
    $env:SDKWORK_ROUTER_BINARY = $binaryPath
}
if (-not $env:SDKWORK_CONFIG_DIR) {
    $env:SDKWORK_CONFIG_DIR = Convert-ToRouterPortablePath -PathValue $configDirectory
}
if (-not $env:SDKWORK_DATABASE_URL) {
    $env:SDKWORK_DATABASE_URL = "sqlite://$(Convert-ToRouterPortablePath -PathValue $dataDirectory)/sdkwork-api-router.db"
}
if (-not $env:SDKWORK_WEB_BIND) {
    $env:SDKWORK_WEB_BIND = '0.0.0.0:9983'
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
if (-not $env:SDKWORK_ADMIN_SITE_DIR) {
    $env:SDKWORK_ADMIN_SITE_DIR = Convert-ToRouterPortablePath -PathValue $defaultAdminSiteDir
}
if (-not $env:SDKWORK_PORTAL_SITE_DIR) {
    $env:SDKWORK_PORTAL_SITE_DIR = Convert-ToRouterPortablePath -PathValue $defaultPortalSiteDir
}

if ($Bind) { $env:SDKWORK_WEB_BIND = $Bind }
if ($ConfigDir) { $env:SDKWORK_CONFIG_DIR = $ConfigDir }
if ($ConfigFile) { $env:SDKWORK_CONFIG_FILE = $ConfigFile }
if ($DatabaseUrl) { $env:SDKWORK_DATABASE_URL = $DatabaseUrl }
if ($Roles) { $env:SDKWORK_ROUTER_ROLES = $Roles }
if ($NodeIdPrefix) { $env:SDKWORK_ROUTER_NODE_ID_PREFIX = $NodeIdPrefix }
if ($GatewayBind) { $env:SDKWORK_GATEWAY_BIND = $GatewayBind }
if ($AdminBind) { $env:SDKWORK_ADMIN_BIND = $AdminBind }
if ($PortalBind) { $env:SDKWORK_PORTAL_BIND = $PortalBind }
if ($GatewayUpstream) { $env:SDKWORK_GATEWAY_PROXY_TARGET = $GatewayUpstream }
if ($AdminUpstream) { $env:SDKWORK_ADMIN_PROXY_TARGET = $AdminUpstream }
if ($PortalUpstream) { $env:SDKWORK_PORTAL_PROXY_TARGET = $PortalUpstream }
if ($AdminSiteDir) { $env:SDKWORK_ADMIN_SITE_DIR = $AdminSiteDir }
if ($PortalSiteDir) { $env:SDKWORK_PORTAL_SITE_DIR = $PortalSiteDir }

Assert-RouterFileExists -Label 'router-product-service binary' -FilePath $env:SDKWORK_ROUTER_BINARY
Assert-RouterDirectoryExists -Label 'admin site directory' -DirectoryPath $env:SDKWORK_ADMIN_SITE_DIR
Assert-RouterDirectoryExists -Label 'portal site directory' -DirectoryPath $env:SDKWORK_PORTAL_SITE_DIR

Push-Location $runtimeHome
try {
    $planOutput = & $env:SDKWORK_ROUTER_BINARY --dry-run --plan-format json
    Set-Content -Path $planFile -Value $planOutput -Encoding utf8

    if ($DryRun) {
        Get-Content $planFile
        return
    }

    if ($Foreground) {
        & $env:SDKWORK_ROUTER_BINARY
        return
    }

    Assert-RouterNotRunning -PidFile $pidFile
    if (Test-Path $stdoutLog) { Remove-Item $stdoutLog -Force }
    if (Test-Path $stderrLog) { Remove-Item $stderrLog -Force }

    $process = Start-RouterBackgroundProcess `
        -FilePath $env:SDKWORK_ROUTER_BINARY `
        -WorkingDirectory $runtimeHome `
        -StdoutLog $stdoutLog `
        -StderrLog $stderrLog

    Set-Content -Path $pidFile -Value $process.Id -Encoding utf8

    $gatewayHealthUrl = Resolve-RouterHealthUrl -BindAddress $env:SDKWORK_WEB_BIND -PathSuffix '/api/v1/health'
    $adminHealthUrl = Resolve-RouterHealthUrl -BindAddress $env:SDKWORK_WEB_BIND -PathSuffix '/api/admin/health'
    $portalHealthUrl = Resolve-RouterHealthUrl -BindAddress $env:SDKWORK_WEB_BIND -PathSuffix '/api/portal/health'

    $ready = (Wait-RouterHealthUrl -Url $gatewayHealthUrl -WaitSeconds $WaitSeconds -ProcessId $process.Id) `
        -and (Wait-RouterHealthUrl -Url $adminHealthUrl -WaitSeconds $WaitSeconds -ProcessId $process.Id) `
        -and (Wait-RouterHealthUrl -Url $portalHealthUrl -WaitSeconds $WaitSeconds -ProcessId $process.Id)

    if (-not $ready) {
        $runtimeExited = -not (Get-Process -Id $process.Id -ErrorAction SilentlyContinue)
        Stop-RouterProcessTree -ProcessId $process.Id -WaitSeconds $WaitSeconds -Force | Out-Null
        Remove-Item $pidFile -Force -ErrorAction SilentlyContinue
        Show-RouterLogTail -LogFile $stdoutLog
        Show-RouterLogTail -LogFile $stderrLog
        if ($runtimeExited) {
            Throw-RouterError 'production runtime exited before health checks completed; see startup log above'
        }
        Throw-RouterError "router-product-service failed health checks on $($env:SDKWORK_WEB_BIND)"
    }

    Write-RouterInfo "started router-product-service (pid=$($process.Id))"
    Write-RouterStartupSummary `
        -Mode 'production release' `
        -WebBind $env:SDKWORK_WEB_BIND `
        -GatewayBind $env:SDKWORK_GATEWAY_BIND `
        -AdminBind $env:SDKWORK_ADMIN_BIND `
        -PortalBind $env:SDKWORK_PORTAL_BIND `
        -UnifiedAccessEnabled $true `
        -StdoutLog $stdoutLog `
        -StderrLog $stderrLog
}
finally {
    Pop-Location
}
