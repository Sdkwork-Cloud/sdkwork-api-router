$ErrorActionPreference = 'Stop'
$scriptDir = Split-Path -Parent $PSCommandPath
& node (Join-Path $scriptDir 'router-ops.mjs') install @args
exit $LASTEXITCODE
