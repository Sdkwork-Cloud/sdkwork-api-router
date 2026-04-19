param(
    [Alias('Home')]
    [string]$RuntimeHome = '',
    [Alias('SourcePath')]
    [string]$RestoreSource = '',
    [switch]$DryRun,
    [switch]$Force,
    [ValidateSet('text', 'json')]
    [string]$PlanFormat = 'json'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

. (Join-Path (Split-Path -Parent $PSCommandPath) 'lib\runtime-common.ps1')

$scriptDir = Split-Path -Parent $PSCommandPath
$repoRoot = Get-RouterRepoRoot -ScriptDirectory $scriptDir
$defaultHome = Get-RouterDefaultInstallHome -RepoRoot $repoRoot
$binaryName = Get-RouterBinaryName -BaseName 'router-product-service'

if ([string]::IsNullOrWhiteSpace($RestoreSource)) {
    Throw-RouterError '--source requires a value'
}

if ([string]::IsNullOrWhiteSpace($RuntimeHome)) {
    $manifestHome = Split-Path -Parent $scriptDir
    $siblingBinary = Join-Path $scriptDir $binaryName
    if (Test-Path (Join-Path $manifestHome 'release-manifest.json')) {
        $RuntimeHome = $manifestHome
    } elseif (Test-Path $siblingBinary) {
        $RuntimeHome = Split-Path -Parent $scriptDir
    } else {
        $RuntimeHome = $defaultHome
    }
}

$runtimeHome = Resolve-RouterAbsolutePath -BasePath (Get-Location).Path -CandidatePath $RuntimeHome
$releaseManifest = Get-RouterReleaseManifest -RuntimeHome $runtimeHome
$manifestReleaseRoot = Get-RouterReleaseManifestString -Manifest $releaseManifest -PropertyName 'releaseRoot'
$manifestRouterBinary = Get-RouterReleaseManifestString -Manifest $releaseManifest -PropertyName 'routerBinary'
$defaultBinaryPath = if (-not [string]::IsNullOrWhiteSpace($manifestReleaseRoot)) {
    Join-Path $manifestReleaseRoot (Join-Path 'bin' $binaryName)
} else {
    Join-Path $runtimeHome (Join-Path 'bin' $binaryName)
}
$binaryPath = if (-not [string]::IsNullOrWhiteSpace($manifestRouterBinary)) {
    $manifestRouterBinary
} else {
    $defaultBinaryPath
}
$binaryPath = Resolve-RouterHostPath -PathValue $binaryPath -DefaultValue $defaultBinaryPath

if (-not (Test-Path $binaryPath -PathType Leaf)) {
    Throw-RouterError "router-product-service binary not found: $binaryPath"
}

$arguments = @('--runtime-home', $runtimeHome, '--restore-source', $RestoreSource)
if ($Force) {
    $arguments += '--force'
}
if ($DryRun) {
    $arguments += @('--dry-run', '--plan-format', $PlanFormat)
}

& $binaryPath @arguments
$hasExitCode = Test-Path Variable:LASTEXITCODE
if ($hasExitCode -and $LASTEXITCODE) {
    exit $LASTEXITCODE
}
