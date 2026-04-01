Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script:RouterDefaultAdminEmail = 'admin@sdkwork.local'
$script:RouterDefaultAdminPassword = 'ChangeMe123!'
$script:RouterDefaultPortalEmail = 'portal@sdkwork.local'
$script:RouterDefaultPortalPassword = 'ChangeMe123!'

function Write-RouterInfo {
    param([Parameter(Mandatory = $true)][string]$Message)
    Write-Host "[sdkwork-router] $Message"
}

function Throw-RouterError {
    param([Parameter(Mandatory = $true)][string]$Message)
    throw "[sdkwork-router] $Message"
}

function Convert-ToRouterPortablePath {
    param([Parameter(Mandatory = $true)][string]$PathValue)
    return $PathValue.Replace('\', '/')
}

function Get-RouterScriptDirectory {
    param([Parameter(Mandatory = $true)][string]$ScriptPath)
    return Split-Path -Parent (Resolve-Path $ScriptPath)
}

function Get-RouterRepoRoot {
    param([Parameter(Mandatory = $true)][string]$ScriptDirectory)
    return Split-Path -Parent (Resolve-Path $ScriptDirectory)
}

function Get-RouterDefaultInstallHome {
    param([Parameter(Mandatory = $true)][string]$RepoRoot)
    return Join-Path $RepoRoot 'artifacts\install\sdkwork-api-router\current'
}

function Get-RouterDefaultDevHome {
    param([Parameter(Mandatory = $true)][string]$RepoRoot)
    return Join-Path $RepoRoot 'artifacts\runtime\dev'
}

function Test-RouterWindowsPlatform {
    $osName = [string]$env:OS
    if ($osName.Equals('Windows_NT', [System.StringComparison]::OrdinalIgnoreCase)) {
        return $true
    }

    return $PSVersionTable.PSEdition -eq 'Desktop'
}

function Get-RouterBinaryName {
    param([Parameter(Mandatory = $true)][string]$BaseName)

    if (Test-RouterWindowsPlatform) {
        return "$BaseName.exe"
    }

    return $BaseName
}

function Ensure-RouterDirectory {
    param([Parameter(Mandatory = $true)][string]$DirectoryPath)
    New-Item -ItemType Directory -Force -Path $DirectoryPath | Out-Null
}

function Import-RouterEnvFile {
    param([Parameter(Mandatory = $true)][string]$EnvFile)
    if (-not (Test-Path $EnvFile)) {
        return
    }

    foreach ($rawLine in Get-Content $EnvFile) {
        $line = $rawLine.Trim()
        if ([string]::IsNullOrWhiteSpace($line) -or $line.StartsWith('#')) {
            continue
        }

        $separatorIndex = $line.IndexOf('=')
        if ($separatorIndex -lt 1) {
            continue
        }

        $key = $line.Substring(0, $separatorIndex).Trim()
        $value = $line.Substring($separatorIndex + 1).Trim()
        if ($value.Length -ge 2) {
            $quote = $value[0]
            if (($quote -eq '"' -or $quote -eq "'") -and $value[-1] -eq $quote) {
                $value = $value.Substring(1, $value.Length - 2)
                if ($quote -eq '"') {
                    $value = $value.Replace('\"', '"').Replace('\\', '\')
                }
            }
        }
        Set-Item -Path "Env:$key" -Value $value
    }
}

function Test-RouterProcessRunning {
    param([Parameter(Mandatory = $true)][string]$PidValue)
    if ([string]::IsNullOrWhiteSpace($PidValue)) {
        return $false
    }

    $process = Get-Process -Id ([int]$PidValue) -ErrorAction SilentlyContinue
    return $null -ne $process
}

function Clear-RouterStalePidFile {
    param([Parameter(Mandatory = $true)][string]$PidFile)
    if (-not (Test-Path $PidFile)) {
        return $true
    }

    $pidValue = (Get-Content $PidFile -ErrorAction SilentlyContinue | Select-Object -First 1).Trim()
    if ([string]::IsNullOrWhiteSpace($pidValue)) {
        Remove-Item $PidFile -Force -ErrorAction SilentlyContinue
        return $true
    }

    if (Test-RouterProcessRunning -PidValue $pidValue) {
        return $false
    }

    Remove-Item $PidFile -Force -ErrorAction SilentlyContinue
    return $true
}

function Assert-RouterNotRunning {
    param([Parameter(Mandatory = $true)][string]$PidFile)
    if (Clear-RouterStalePidFile -PidFile $PidFile) {
        return
    }

    $pidValue = (Get-Content $PidFile | Select-Object -First 1).Trim()
    Throw-RouterError "process already running with pid $pidValue (pid file: $PidFile)"
}

function Wait-RouterProcessExit {
    param(
        [Parameter(Mandatory = $true)][int]$ProcessId,
        [Parameter(Mandatory = $true)][int]$WaitSeconds
    )

    $deadline = (Get-Date).AddSeconds($WaitSeconds)
    while ((Get-Date) -lt $deadline) {
        if (-not (Get-Process -Id $ProcessId -ErrorAction SilentlyContinue)) {
            return $true
        }
        Start-Sleep -Seconds 1
    }

    return -not (Get-Process -Id $ProcessId -ErrorAction SilentlyContinue)
}

function Get-RouterChildProcessIds {
    param([Parameter(Mandatory = $true)][int]$ParentPid)

    if (Test-RouterWindowsPlatform) {
        $cimCommand = Get-Command Get-CimInstance -ErrorAction SilentlyContinue
        if ($null -eq $cimCommand) {
            return @()
        }

        $childProcesses = Get-CimInstance Win32_Process -Filter "ParentProcessId = $ParentPid" -ErrorAction SilentlyContinue
        return @($childProcesses | ForEach-Object { [int]$_.ProcessId })
    }

    $psCommand = Get-Command ps -ErrorAction SilentlyContinue
    if ($null -eq $psCommand) {
        return @()
    }

    $childIds = @()
    foreach ($line in (& ps -o pid= -o ppid= 2>$null)) {
        $parts = @($line -split '\s+' | Where-Object { $_ })
        if ($parts.Count -lt 2) {
            continue
        }

        $processId = 0
        $reportedParentPid = 0
        if (-not [int]::TryParse($parts[0], [ref]$processId)) {
            continue
        }
        if (-not [int]::TryParse($parts[1], [ref]$reportedParentPid)) {
            continue
        }
        if ($reportedParentPid -eq $ParentPid) {
            $childIds += $processId
        }
    }

    return @($childIds | Select-Object -Unique)
}

function Get-RouterProcessTreeIds {
    param([Parameter(Mandatory = $true)][int]$ParentPid)

    $descendants = @()
    foreach ($childPid in Get-RouterChildProcessIds -ParentPid $ParentPid) {
        $descendants += $childPid
        $descendants += Get-RouterProcessTreeIds -ParentPid $childPid
    }

    return @($descendants | Select-Object -Unique)
}

function Stop-RouterProcessTree {
    param(
        [Parameter(Mandatory = $true)][int]$ProcessId,
        [Parameter(Mandatory = $true)][int]$WaitSeconds,
        [switch]$Force
    )

    if (-not (Get-Process -Id $ProcessId -ErrorAction SilentlyContinue)) {
        return $true
    }

    if (Test-RouterWindowsPlatform) {
        & cmd.exe /c "taskkill /PID $ProcessId /T" | Out-Null
        if (Wait-RouterProcessExit -ProcessId $ProcessId -WaitSeconds $WaitSeconds) {
            return $true
        }

        if (-not $Force) {
            return $false
        }

        & cmd.exe /c "taskkill /PID $ProcessId /T /F" | Out-Null
        return (Wait-RouterProcessExit -ProcessId $ProcessId -WaitSeconds $WaitSeconds)
    }

    $processIds = @(Get-RouterProcessTreeIds -ParentPid $ProcessId)
    $processIds += $ProcessId
    $orderedProcessIds = @($processIds | Select-Object -Unique | Sort-Object -Descending)

    foreach ($processId in $orderedProcessIds) {
        Stop-Process -Id $processId -ErrorAction SilentlyContinue
    }
    if (Wait-RouterProcessExit -ProcessId $ProcessId -WaitSeconds $WaitSeconds) {
        return $true
    }

    if (-not $Force) {
        return $false
    }

    foreach ($processId in $orderedProcessIds) {
        Stop-Process -Id $processId -Force -ErrorAction SilentlyContinue
    }

    return (Wait-RouterProcessExit -ProcessId $ProcessId -WaitSeconds $WaitSeconds)
}

function Start-RouterBackgroundProcess {
    param(
        [Parameter(Mandatory = $true)][string]$FilePath,
        [Parameter()][string[]]$ArgumentList = @(),
        [Parameter(Mandatory = $true)][string]$WorkingDirectory,
        [Parameter(Mandatory = $true)][string]$StdoutLog,
        [Parameter(Mandatory = $true)][string]$StderrLog
    )

    return Start-Process `
        -FilePath $FilePath `
        -ArgumentList $ArgumentList `
        -WorkingDirectory $WorkingDirectory `
        -RedirectStandardOutput $StdoutLog `
        -RedirectStandardError $StderrLog `
        -NoNewWindow `
        -PassThru
}

function Resolve-RouterHealthUrl {
    param(
        [Parameter(Mandatory = $true)][string]$BindAddress,
        [Parameter(Mandatory = $true)][string]$PathSuffix
    )

    $parts = $BindAddress.Split(':')
    if ($parts.Length -lt 2) {
        Throw-RouterError "invalid bind address: $BindAddress"
    }

    $bindHost = ($parts[0..($parts.Length - 2)] -join ':')
    $bindPort = $parts[-1]
    if ([string]::IsNullOrWhiteSpace($bindHost) -or $bindHost -eq '0.0.0.0' -or $bindHost -eq '[::]' -or $bindHost -eq '::') {
        $bindHost = '127.0.0.1'
    }

    return "http://$bindHost`:$bindPort$PathSuffix"
}

function Wait-RouterHealthUrl {
    param(
        [Parameter(Mandatory = $true)][string]$Url,
        [Parameter(Mandatory = $true)][int]$WaitSeconds,
        [int]$ProcessId = 0
    )

    $deadline = (Get-Date).AddSeconds($WaitSeconds)
    while ((Get-Date) -lt $deadline) {
        if ($ProcessId -gt 0 -and -not (Get-Process -Id $ProcessId -ErrorAction SilentlyContinue)) {
            return $false
        }
        try {
            $response = Invoke-WebRequest -UseBasicParsing $Url -TimeoutSec 3
            if ($response.StatusCode -ge 200 -and $response.StatusCode -lt 300) {
                return $true
            }
        } catch {
        }
        Start-Sleep -Seconds 1
    }

    if ($ProcessId -gt 0 -and -not (Get-Process -Id $ProcessId -ErrorAction SilentlyContinue)) {
        return $false
    }

    return $false
}

function Show-RouterLogTail {
    param([Parameter(Mandatory = $true)][string]$LogFile)
    if (Test-Path $LogFile) {
        Get-Content $LogFile -Tail 60 -ErrorAction SilentlyContinue
    }
}

function Assert-RouterFileExists {
    param(
        [Parameter(Mandatory = $true)][string]$Label,
        [Parameter(Mandatory = $true)][string]$FilePath
    )
    if (-not (Test-Path $FilePath -PathType Leaf)) {
        Throw-RouterError "$Label not found: $FilePath"
    }
}

function Assert-RouterDirectoryExists {
    param(
        [Parameter(Mandatory = $true)][string]$Label,
        [Parameter(Mandatory = $true)][string]$DirectoryPath
    )
    if (-not (Test-Path $DirectoryPath -PathType Container)) {
        Throw-RouterError "$Label not found: $DirectoryPath"
    }
}

function Get-RouterStartupSummaryLines {
    param(
        [Parameter(Mandatory = $true)][string]$Mode,
        [Parameter(Mandatory = $true)][string]$WebBind,
        [Parameter(Mandatory = $true)][string]$GatewayBind,
        [Parameter(Mandatory = $true)][string]$AdminBind,
        [Parameter(Mandatory = $true)][string]$PortalBind,
        [bool]$UnifiedAccessEnabled = $true,
        [string]$AdminAppUrl = '',
        [string]$PortalAppUrl = '',
        [Parameter(Mandatory = $true)][string]$StdoutLog,
        [Parameter(Mandatory = $true)][string]$StderrLog
    )

    if (-not $AdminAppUrl) {
        $AdminAppUrl = Resolve-RouterHealthUrl -BindAddress $WebBind -PathSuffix '/admin/'
    }
    if (-not $PortalAppUrl) {
        $PortalAppUrl = Resolve-RouterHealthUrl -BindAddress $WebBind -PathSuffix '/portal/'
    }

    $gatewayUnifiedUrl = Resolve-RouterHealthUrl -BindAddress $WebBind -PathSuffix '/api/v1/health'
    $adminUnifiedUrl = Resolve-RouterHealthUrl -BindAddress $WebBind -PathSuffix '/api/admin/health'
    $portalUnifiedUrl = Resolve-RouterHealthUrl -BindAddress $WebBind -PathSuffix '/api/portal/health'
    $gatewayDirectUrl = Resolve-RouterHealthUrl -BindAddress $GatewayBind -PathSuffix '/health'
    $adminDirectUrl = Resolve-RouterHealthUrl -BindAddress $AdminBind -PathSuffix '/admin/health'
    $portalDirectUrl = Resolve-RouterHealthUrl -BindAddress $PortalBind -PathSuffix '/portal/health'

    $lines = @(
        '------------------------------------------------------------',
        "Mode: $Mode",
        "Bind Summary: web=$WebBind gateway=$GatewayBind admin=$AdminBind portal=$PortalBind"
    )

    if ($UnifiedAccessEnabled) {
        $lines += @(
            'Unified Access',
            "  Admin App: $AdminAppUrl",
            "  Portal App: $PortalAppUrl",
            "  Gateway API Health: $gatewayUnifiedUrl",
            "  Admin API Health: $adminUnifiedUrl",
            "  Portal API Health: $portalUnifiedUrl"
        )
    } else {
        $lines += @(
            'Frontend Access',
            "  Admin App: $AdminAppUrl",
            "  Portal App: $PortalAppUrl"
        )
    }

    $lines += @(
        'Direct Service Access',
        "  Gateway Service: $gatewayDirectUrl",
        "  Admin Service: $adminDirectUrl",
        "  Portal Service: $portalDirectUrl",
        'Initial Credentials',
        "  Admin Console: $($script:RouterDefaultAdminEmail) / $($script:RouterDefaultAdminPassword)",
        "  Portal Console: $($script:RouterDefaultPortalEmail) / $($script:RouterDefaultPortalPassword)",
        '  Gateway API: sign in through the portal and create an API key.',
        'Logs',
        "  STDOUT: $StdoutLog",
        "  STDERR: $StderrLog"
    )

    return $lines
}

function Write-RouterStartupSummary {
    param(
        [Parameter(Mandatory = $true)][string]$Mode,
        [Parameter(Mandatory = $true)][string]$WebBind,
        [Parameter(Mandatory = $true)][string]$GatewayBind,
        [Parameter(Mandatory = $true)][string]$AdminBind,
        [Parameter(Mandatory = $true)][string]$PortalBind,
        [bool]$UnifiedAccessEnabled = $true,
        [string]$AdminAppUrl = '',
        [string]$PortalAppUrl = '',
        [Parameter(Mandatory = $true)][string]$StdoutLog,
        [Parameter(Mandatory = $true)][string]$StderrLog
    )

    foreach ($line in Get-RouterStartupSummaryLines @PSBoundParameters) {
        Write-RouterInfo $line
    }
}
