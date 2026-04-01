param(
    [Alias('Home')]
    [string]$RuntimeHome = '',
    [switch]$DryRun,
    [int]$WaitSeconds = 30,
    [switch]$GracefulOnly
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
$pidFile = Join-Path $runtimeHome 'var\run\router-product-service.pid'
$stdoutLog = Join-Path $runtimeHome 'var\log\router-product-service.stdout.log'
$stderrLog = Join-Path $runtimeHome 'var\log\router-product-service.stderr.log'

if ($DryRun) {
    Write-RouterInfo "would stop router-product-service using pid file $pidFile"
    return
}

if (-not (Test-Path $pidFile)) {
    Write-RouterInfo "pid file not found, nothing to stop: $pidFile"
    return
}

$pidValue = (Get-Content $pidFile -ErrorAction SilentlyContinue | Select-Object -First 1).Trim()
if ([string]::IsNullOrWhiteSpace($pidValue)) {
    Remove-Item $pidFile -Force -ErrorAction SilentlyContinue
    Write-RouterInfo "removed empty pid file: $pidFile"
    return
}

if (-not (Test-RouterProcessRunning -PidValue $pidValue)) {
    Remove-Item $pidFile -Force -ErrorAction SilentlyContinue
    Write-RouterInfo "process already stopped, removed stale pid file: $pidFile"
    return
}

$stopped = Stop-RouterProcessTree -Pid ([int]$pidValue) -WaitSeconds $WaitSeconds -Force:(-not $GracefulOnly)
if (-not $stopped) {
    Show-RouterLogTail -LogFile $stdoutLog
    Show-RouterLogTail -LogFile $stderrLog
    Throw-RouterError "failed to stop router-product-service pid=$pidValue"
}

Remove-Item $pidFile -Force -ErrorAction SilentlyContinue
Write-RouterInfo "stopped router-product-service pid=$pidValue"
