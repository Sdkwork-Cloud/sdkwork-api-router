# Build and Packaging

This page is the compilation guide for the repository. Use it when you need to produce artifacts instead of simply running from source.

## Build Targets

| Target | Command | Output |
|---|---|---|
| gateway service | `cargo build --release -p gateway-service` | `target/release/gateway-service` |
| admin service | `cargo build --release -p admin-api-service` | `target/release/admin-api-service` |
| portal service | `cargo build --release -p portal-api-service` | `target/release/portal-api-service` |
| admin console | `pnpm --dir console build` | `console/dist/` |
| portal web app | `pnpm --dir apps/sdkwork-router-portal build` | `apps/sdkwork-router-portal/dist/` |
| docs site | `pnpm --dir docs build` | `docs/.vitepress/dist/` |
| Tauri desktop app | `pnpm --dir console tauri:build` | platform-specific Tauri bundle output |

## Compile the Standalone Services

Build all three production binaries:

```bash
cargo build --release -p admin-api-service -p gateway-service -p portal-api-service
```

For day-to-day development, a non-release build is usually enough:

```bash
cargo build -p admin-api-service -p gateway-service -p portal-api-service
```

## Build the Admin Console

Install dependencies if needed:

```bash
pnpm --dir console install
```

Build:

```bash
pnpm --dir console build
```

Preview locally:

```bash
pnpm --dir console preview
```

## Build the Standalone Portal App

Install dependencies if needed:

```bash
pnpm --dir apps/sdkwork-router-portal install
```

Build:

```bash
pnpm --dir apps/sdkwork-router-portal build
```

Preview locally:

```bash
pnpm --dir apps/sdkwork-router-portal preview
```

## Build the Documentation Site

Install docs dependencies if needed:

```bash
pnpm --dir docs install
```

Build:

```bash
pnpm --dir docs build
```

Preview locally:

```bash
pnpm --dir docs preview
```

## Build the Tauri Desktop App

Development shell:

```bash
pnpm --dir console tauri:dev
```

Production package:

```bash
pnpm --dir console tauri:build
```

Use this path when you want an embedded, desktop-oriented operator experience instead of separately deployed services.

## Recommended Verification Before Packaging

```bash
cargo fmt --all --check
cargo test --workspace -q -j 1
pnpm --dir console build
pnpm --dir apps/sdkwork-router-portal build
pnpm --dir docs build
```

If you are changing TypeScript or docs config as well:

```bash
pnpm --dir console -r typecheck
pnpm --dir apps/sdkwork-router-portal typecheck
pnpm --dir docs typecheck
```

## Related Docs

- source workflows:
  - [Source Development](/getting-started/source-development)
- deployable artifacts:
  - [Release Builds](/getting-started/release-builds)
- workspace structure:
  - [Repository Layout](/reference/repository-layout)
