---
layout: home

hero:
  name: SDKWork API Server
  text: OpenAI-compatible gateway, admin control plane, public portal, and extension runtime
  tagline: A cross-platform Rust workspace for running OpenAI-style APIs, routing traffic across providers, operating a control plane, and shipping browser or desktop operator experiences.
  actions:
    - theme: brand
      text: Quickstart
      link: /getting-started/quickstart
    - theme: alt
      text: API Reference
      link: /api-reference/overview
    - theme: alt
      text: Software Architecture
      link: /architecture/software-architecture

features:
  - title: OpenAI-compatible gateway
    details: Expose a broad `/v1/*` surface for chat, responses, embeddings, files, uploads, audio, images, assistants, threads, vector stores, evals, videos, and more.
  - title: Native control plane
    details: Operate channels, providers, credentials, routing policies, runtime rollouts, usage, billing, and quota management through `/admin/*`.
  - title: Public self-service portal
    details: Let end users register, sign in, inspect their workspace, review usage and billing posture, and issue gateway API keys through `/portal/*` and the standalone portal app.
  - title: Pluggable runtime
    details: Run builtin, connector, and native-dynamic provider runtimes with hot reload, health snapshots, and rollout-aware supervision.
---

## Documentation Map

SDKWork now follows a documentation structure closer to mature API platforms such as OpenAI's official docs:

- [Getting Started](/getting-started/installation): install prerequisites, run from source, compile binaries, and package browser or Tauri artifacts
- [Script Lifecycle](/getting-started/script-lifecycle): understand what every startup script does and how build, install, start, verify, stop, and service registration fit together
- [Architecture](/architecture/software-architecture): understand the standalone services, workspace layering, extension runtime, and module boundaries
- [API Reference](/api-reference/overview): navigate the gateway, admin, and portal surfaces with the right base path and auth model
- [Operations](/operations/configuration): configure, observe, and troubleshoot standalone deployments
- [Reference](/reference/api-compatibility): inspect compatibility labels, repository layout, and build tooling

## Start Here

Choose the path that matches what you need right now:

- First verified local run:
  - [Quickstart](/getting-started/quickstart)
- First-time setup:
  - [Installation](/getting-started/installation)
- Local development:
  - [Source Development](/getting-started/source-development)
- Script responsibilities:
  - [Script Lifecycle](/getting-started/script-lifecycle)
- Compilation and packaging:
  - [Build and Packaging](/getting-started/build-and-packaging)
- Deployable artifacts:
  - [Release Builds](/getting-started/release-builds)
- System design:
  - [Software Architecture](/architecture/software-architecture)
- Endpoint inventory:
  - [Gateway API](/api-reference/gateway-api)

## Product Surfaces

| Surface | Base path | Purpose |
|---|---|---|
| gateway-service | `/v1/*` | OpenAI-compatible data plane |
| admin-api-service | `/admin/*` | operator control plane |
| portal-api-service | `/portal/*` | self-service auth, workspace, and API key lifecycle |
| router-web-service | `/admin/*`, `/portal/*`, `/api/*` | Pingora public site delivery and API proxy entry |
| apps/sdkwork-router-admin | browser or Tauri | standalone super-admin experience |
| apps/sdkwork-router-portal | browser | standalone developer self-service portal |
| docs | `/` | VitePress documentation site |

## Common Local Ports

Managed script defaults:

| Surface | Default bind |
|---|---|
| gateway | `127.0.0.1:9980` |
| admin | `127.0.0.1:9981` |
| portal | `127.0.0.1:9982` |
| web host | `127.0.0.1:9983` |
| admin web app | `127.0.0.1:5173` |
| portal web app | `127.0.0.1:5174` |

Raw binary defaults remain `8080`, `8081`, and `8082` unless overridden.

## Fast Paths

Run the managed development stack:

```bash
./bin/start-dev.sh
```

Run the full stack from source with raw workspace control:

```bash
node scripts/dev/start-workspace.mjs
```

Run the full stack with the desktop shell and shared Pingora host:

```bash
node scripts/dev/start-workspace.mjs --tauri
```

Compile the standalone release binaries:

```bash
cargo build --release -p admin-api-service -p gateway-service -p portal-api-service
```

Build the admin app:

```bash
pnpm --dir apps/sdkwork-router-admin build
```

Build the standalone portal app:

```bash
pnpm --dir apps/sdkwork-router-portal build
```

Build the docs site:

```bash
pnpm --dir docs build
```
