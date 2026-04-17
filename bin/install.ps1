Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$scriptDir = Split-Path -Parent $PSCommandPath
$translatedArgs = @('install')

for ($index = 0; $index -lt $args.Count; $index += 1) {
    $token = [string]$args[$index]
    switch -Regex ($token) {
        '^(?i)-Force$' {
            $translatedArgs += '--force'
            continue
        }
        '^(?i)-DryRun$' {
            $translatedArgs += '--dry-run'
            continue
        }
        '^(?i)-Mode$' {
            if (($index + 1) -ge $args.Count) {
                throw '--mode requires a value'
            }
            $index += 1
            $translatedArgs += '--mode', ([string]$args[$index])
            continue
        }
        '^(?i)-Home$' {
            if (($index + 1) -ge $args.Count) {
                throw '--home requires a value'
            }
            $index += 1
            $translatedArgs += '--home', ([string]$args[$index])
            continue
        }
        default {
            $translatedArgs += $token
        }
    }
}

& node (Join-Path $scriptDir 'router-ops.mjs') @translatedArgs
exit $LASTEXITCODE
