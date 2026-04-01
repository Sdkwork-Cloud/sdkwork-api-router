param(
    [string]$DatabaseUrl = "",
    [string]$AdminBind = "127.0.0.1:9981",
    [string]$GatewayBind = "127.0.0.1:9980",
    [string]$PortalBind = "127.0.0.1:9982",
    [string]$WebBind = "0.0.0.0:9983",
    [switch]$Install,
    [switch]$Preview,
    [switch]$Tauri,
    [switch]$DryRun,
    [switch]$Help
)

$ErrorActionPreference = "Stop"
$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$arguments = @("scripts/dev/start-workspace.mjs")

if ($DatabaseUrl) {
    $arguments += @("--database-url", $DatabaseUrl)
}

$arguments += @(
    "--gateway-bind", $GatewayBind,
    "--admin-bind", $AdminBind,
    "--portal-bind", $PortalBind,
    "--web-bind", $WebBind
)

if ($Install) {
    $arguments += "--install"
}
if ($Preview) {
    $arguments += "--preview"
}
if ($Tauri) {
    $arguments += "--tauri"
}
if ($DryRun) {
    $arguments += "--dry-run"
}
if ($Help) {
    $arguments += "--help"
}

Push-Location $repoRoot
try {
    & node @arguments
    exit $LASTEXITCODE
} finally {
    Pop-Location
}
