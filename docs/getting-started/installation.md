# Installation

This page covers prerequisites, repository bootstrap, and first-run verification for Windows, Linux, and macOS.

## What You Are Installing

SDKWork API Server is not a single binary. The repository contains:

- three standalone Rust services
- a browser/Tauri operator console under `console/`
- a standalone browser portal app under `apps/sdkwork-router-portal/`
- a VitePress documentation site
- a Rust extension and provider runtime workspace

The fastest way to become productive is:

1. install the required toolchain
2. clone the repository
3. install `console/`, `apps/sdkwork-router-portal/`, and `docs/` dependencies
4. continue to [Quickstart](/getting-started/quickstart)

## Required Tooling

Install these on every platform:

- Rust stable with Cargo
- Node.js 20 or newer
- pnpm 10 or newer

Recommended optional tooling:

- PostgreSQL 15 or newer for PostgreSQL-backed deployments
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
git clone https://github.com/Sdkwork-Cloud/sdkwork-api-server.git
cd sdkwork-api-server
```

If pnpm is not already available, enable Corepack first:

```bash
corepack enable
```

Install console dependencies:

```bash
pnpm --dir console install
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

If you plan to work on the UI shell as well:

```bash
pnpm --dir console build
pnpm --dir apps/sdkwork-router-portal build
```

## Next Steps

- Run a verified first request flow:
  - [Quickstart](/getting-started/quickstart)
- Run the full stack locally:
  - [Source Development](/getting-started/source-development)
- Compile and package services, admin console, portal app, docs, or Tauri:
  - [Build and Packaging](/getting-started/build-and-packaging)
- Prepare deployment artifacts:
  - [Release Builds](/getting-started/release-builds)
- Review the system shape before deeper changes:
  - [Software Architecture](/architecture/software-architecture)

To preview the docs site locally:

```bash
pnpm --dir docs dev
```
