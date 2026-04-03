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
$stopFile = Join-Path $devHome 'run\start-workspace.stop'
$stateFile = Join-Path $devHome 'run\start-workspace.state.env'
$stdoutLog = Join-Path $devHome 'log\start-workspace.stdout.log'
$stderrLog = Join-Path $devHome 'log\start-workspace.stderr.log'

if ($DryRun) {
    Write-RouterInfo "would stop development workspace using pid file $pidFile and stop file $stopFile"
    return
}

if (-not (Test-Path $pidFile)) {
    Remove-Item $stopFile -Force -ErrorAction SilentlyContinue
    Remove-RouterManagedStateFile -StateFile $stateFile
    Write-RouterInfo "pid file not found, nothing to stop: $pidFile"
    return
}

$pidValue = Get-RouterManagedProcessId -PidFile $pidFile -StateFile $stateFile
if ($pidValue -le 0) {
    Remove-Item $pidFile -Force -ErrorAction SilentlyContinue
    Remove-Item $stopFile -Force -ErrorAction SilentlyContinue
    Remove-RouterManagedStateFile -StateFile $stateFile
    Write-RouterInfo "process already stopped, removed stale pid file: $pidFile"
    return
}

Set-Content -Path $stopFile -Value (Get-Date -Format o) -Encoding utf8
$stopped = Wait-RouterProcessExit -ProcessId ([int]$pidValue) -WaitSeconds $WaitSeconds
if (-not $stopped -and -not $GracefulOnly) {
    Write-RouterInfo "graceful stop timed out for development workspace pid=$pidValue, falling back to process termination"
    $stopped = Stop-RouterProcessTree -ProcessId ([int]$pidValue) -WaitSeconds $WaitSeconds -Force
}

if (-not $stopped) {
    Show-RouterLogTail -LogFile $stdoutLog
    Show-RouterLogTail -LogFile $stderrLog
    Throw-RouterError "failed to stop development workspace pid=$pidValue"
}

Remove-Item $pidFile -Force -ErrorAction SilentlyContinue
Remove-Item $stopFile -Force -ErrorAction SilentlyContinue
Remove-RouterManagedStateFile -StateFile $stateFile
Write-RouterInfo "stopped development workspace pid=$pidValue"
