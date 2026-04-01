param(
    [switch]$DryRun,
    [int]$WaitSeconds = 30,
    [switch]$GracefulOnly
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

. (Join-Path (Split-Path -Parent $PSCommandPath) 'lib\runtime-common.ps1')

$scriptDir = Split-Path -Parent $PSCommandPath
$repoRoot = Get-RouterRepoRoot -ScriptDirectory $scriptDir
$devHome = Get-RouterDefaultDevHome -RepoRoot $repoRoot
$pidFile = Join-Path $devHome 'run\start-workspace.pid'
$stdoutLog = Join-Path $devHome 'log\start-workspace.stdout.log'
$stderrLog = Join-Path $devHome 'log\start-workspace.stderr.log'

if ($DryRun) {
    Write-RouterInfo "would stop development workspace using pid file $pidFile"
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
    Throw-RouterError "failed to stop development workspace pid=$pidValue"
}

Remove-Item $pidFile -Force -ErrorAction SilentlyContinue
Write-RouterInfo "stopped development workspace pid=$pidValue"
