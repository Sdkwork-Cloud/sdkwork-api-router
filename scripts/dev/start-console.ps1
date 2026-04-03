param(
    [switch]$Install,
    [switch]$Preview,
    [switch]$Tauri,
    [switch]$DryRun,
    [switch]$Help
)

$ErrorActionPreference = "Stop"
$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$arguments = @("scripts/dev/start-console.mjs")

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
