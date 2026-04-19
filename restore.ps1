Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$target = Join-Path $PSScriptRoot 'bin\restore.ps1'
if (-not (Test-Path $target -PathType Leaf)) {
    throw "Missing managed restore entrypoint: $target"
}

& $target @args
$hasExitCode = Test-Path Variable:LASTEXITCODE
if ($hasExitCode -and $LASTEXITCODE) {
    exit $LASTEXITCODE
}
