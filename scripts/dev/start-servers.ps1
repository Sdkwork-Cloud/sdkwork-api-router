param(
    [string]$DatabaseUrl = "",
    [string]$AdminBind = "127.0.0.1:9981",
    [string]$GatewayBind = "127.0.0.1:9980",
    [string]$PortalBind = "127.0.0.1:9982",
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"
$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")

function Escape-PsLiteral([string]$Value) {
    return $Value.Replace("'", "''")
}

function Escape-YamlDoubleQuotedScalar([string]$Value) {
    return $Value.Replace('\', '\\').Replace('"', '\"')
}

function New-SourceDevConfigLayout {
    $runLabel = "$(Get-Date -Format 'yyyyMMdd-HHmmssfff')-$PID"
    $runRoot = Join-Path $repoRoot (Join-Path 'artifacts\runtime\source-workspace' $runLabel)
    $configDir = Join-Path $runRoot 'config'
    $configFile = Join-Path $configDir 'router.yaml'
    New-Item -ItemType Directory -Force -Path $configDir | Out-Null

    $lines = @(
        '# Generated source-dev router config.'
        "gateway_bind: ""$(Escape-YamlDoubleQuotedScalar $GatewayBind)"""
        "admin_bind: ""$(Escape-YamlDoubleQuotedScalar $AdminBind)"""
        "portal_bind: ""$(Escape-YamlDoubleQuotedScalar $PortalBind)"""
    )

    if ($DatabaseUrl) {
        $lines += "database_url: ""$(Escape-YamlDoubleQuotedScalar $DatabaseUrl)"""
    }

    Set-Content -LiteralPath $configFile -Value (($lines + '') -join [Environment]::NewLine) -Encoding UTF8
    return @{
        ConfigDir = $configDir
        ConfigFile = $configFile
    }
}

$sourceConfig = New-SourceDevConfigLayout

function Start-ServiceWindow {
    param(
        [string]$Title,
        [string]$PackageName
    )

    $command = @"
`$Host.UI.RawUI.WindowTitle = '$($Title)'
Set-Location -LiteralPath '$([string](Escape-PsLiteral $repoRoot.Path))'
`$env:SDKWORK_CONFIG_DIR = '$([string](Escape-PsLiteral $sourceConfig.ConfigDir))'
`$env:SDKWORK_CONFIG_FILE = '$([string](Escape-PsLiteral $sourceConfig.ConfigFile))'
`$env:SDKWORK_ADMIN_BIND = '$([string](Escape-PsLiteral $AdminBind))'
`$env:SDKWORK_GATEWAY_BIND = '$([string](Escape-PsLiteral $GatewayBind))'
`$env:SDKWORK_PORTAL_BIND = '$([string](Escape-PsLiteral $PortalBind))'
$(if ($DatabaseUrl) { "`$env:SDKWORK_DATABASE_URL = '$([string](Escape-PsLiteral $DatabaseUrl))'" } else { "Remove-Item Env:SDKWORK_DATABASE_URL -ErrorAction SilentlyContinue" })
cargo run -p $PackageName
"@

    if ($DryRun) {
        Write-Host "[start-servers] powershell -NoExit -Command <window '$Title' running cargo run -p $PackageName>"
        return
    }

    Start-Process powershell -ArgumentList @(
        "-NoExit",
        "-Command",
        $command
    ) | Out-Null
}

if ($DatabaseUrl) {
    Write-Host "[start-servers] SDKWORK_DATABASE_URL=$DatabaseUrl"
} else {
    Write-Host "[start-servers] SDKWORK_DATABASE_URL=(local default via config loader)"
}
Write-Host "[start-servers] SDKWORK_ADMIN_BIND=$AdminBind"
Write-Host "[start-servers] SDKWORK_GATEWAY_BIND=$GatewayBind"
Write-Host "[start-servers] SDKWORK_PORTAL_BIND=$PortalBind"
Write-Host "[start-servers] SDKWORK_CONFIG_FILE=$($sourceConfig.ConfigFile)"

Start-ServiceWindow -Title "sdkwork admin-api-service" -PackageName "admin-api-service"
Start-ServiceWindow -Title "sdkwork gateway-service" -PackageName "gateway-service"
Start-ServiceWindow -Title "sdkwork portal-api-service" -PackageName "portal-api-service"
