param(
    [string]$DatabaseUrl = "sqlite://sdkwork-api-server.db",
    [string]$AdminBind = "127.0.0.1:8081",
    [string]$GatewayBind = "127.0.0.1:8080",
    [string]$PortalBind = "127.0.0.1:8082",
    [switch]$Install,
    [switch]$Preview,
    [switch]$Tauri,
    [switch]$DryRun,
    [switch]$Help
)

$ErrorActionPreference = "Stop"
$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$arguments = @(
    "scripts/dev/start-workspace.mjs",
    "--database-url", $DatabaseUrl,
    "--gateway-bind", $GatewayBind,
    "--admin-bind", $AdminBind,
    "--portal-bind", $PortalBind
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
