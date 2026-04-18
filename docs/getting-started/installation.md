# Installation

This page covers prerequisites, repository bootstrap, and first-run verification for Windows, Linux, and macOS.

## What You Are Installing

SDKWork API Router is a multi-surface product workspace, not a single binary. The repository contains:

- standalone gateway, admin, and portal Rust services
- the integrated `router-product-service` host used by the official server product
- the development-only admin browser app under `apps/sdkwork-router-admin/`
- the standalone portal app plus official desktop shell under `apps/sdkwork-router-portal/`
- a VitePress documentation site
- a Rust extension and provider runtime workspace

The admin browser app is part of the source workspace and the server-delivered admin surface. It is not an official release product.

The fastest way to become productive is:

1. install the required toolchain
2. clone the repository
3. install frontend and docs dependencies
4. continue to [Quickstart](/getting-started/quickstart)

## Required Tooling

Install these on every platform:

- Rust stable with Cargo
- Node.js 20 or newer
- pnpm 10 or newer

Recommended optional tooling:

- PostgreSQL 15 or newer for PostgreSQL-backed server deployments
- Tauri CLI for desktop development or packaging

Install the Tauri CLI:

```bash
cargo install tauri-cli
```

## Platform Notes

### Windows

Recommended environment:

- Rust via `rustup`
- Node.js 20+
- PowerShell 7 or Windows PowerShell
- WebView2 runtime if you intend to use Tauri

### Linux

Recommended environment:

- Rust via `rustup`
- Node.js 20+
- pnpm enabled via Corepack or installed separately
- desktop WebView dependencies if you intend to use Tauri

### macOS

Recommended environment:

- Rust via `rustup`
- Node.js 20+
- pnpm enabled via Corepack or installed separately
- Xcode Command Line Tools for native desktop compilation paths

## Clone and Install

Clone the repository:

```bash
git clone https://github.com/Sdkwork-Cloud/sdkwork-api-router.git
cd sdkwork-api-router
```

If pnpm is not already available, enable Corepack first:

```bash
corepack enable
```

Install admin app dependencies:

```bash
pnpm --dir apps/sdkwork-router-admin install
```

Install portal app dependencies:

```bash
pnpm --dir apps/sdkwork-router-portal install
```

Install docs dependencies:

```bash
pnpm --dir docs install
```

## Verify Tooling

Rust workspace tooling:

```bash
rustc --version
cargo --version
```

Node and pnpm:

```bash
node --version
pnpm --version
```

Optional PostgreSQL:

```bash
psql --version
```

## Recommended First Verification

You do not need to run the full workspace immediately. A quick confidence pass is enough:

```bash
cargo fmt --all --check
pnpm --dir docs build
```

If you plan to work on the browser or desktop shells as well:

```bash
pnpm --dir apps/sdkwork-router-admin build
pnpm --dir apps/sdkwork-router-portal build
```

If you plan to work on the official desktop bundle:

```bash
node scripts/prepare-router-portal-desktop-runtime.mjs
```

## Next Steps

- run a verified first request flow:
  - [Quickstart](/getting-started/quickstart)
- run the full stack locally:
  - [Source Development](/getting-started/source-development)
- compile and package the official products:
  - [Build and Packaging](/getting-started/build-and-packaging)
  - [Release Builds](/getting-started/release-builds)
- review deployment and OS-native installation:
  - [Production Deployment](/getting-started/production-deployment)
- review the system shape before deeper changes:
  - [Software Architecture](/architecture/software-architecture)

To preview the docs site locally:

```bash
pnpm --dir docs dev
```
