Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$scriptDir = Split-Path -Parent $PSCommandPath
$translatedArgs = @('build')

for ($index = 0; $index -lt $args.Count; $index += 1) {
    $token = [string]$args[$index]
    switch -Regex ($token) {
        '^(?i)-Install$' {
            $translatedArgs += '--install'
            continue
        }
        '^(?i)-SkipDocs$' {
            $translatedArgs += '--skip-docs'
            continue
        }
        '^(?i)-DryRun$' {
            $translatedArgs += '--dry-run'
            continue
        }
        '^(?i)-VerifyRelease$' {
            $translatedArgs += '--verify-release'
            continue
        }
        default {
            $translatedArgs += $token
        }
    }
}

& node (Join-Path $scriptDir 'router-ops.mjs') @translatedArgs
exit $LASTEXITCODE
