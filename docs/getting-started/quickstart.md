# Quickstart

This page is the fastest path from clone to a verified local SDKWork stack.

It follows a straightforward onboarding flow:

1. start the runtime
2. verify the control plane
3. sign in as the seeded admin
4. sign in as the seeded portal user
5. create a gateway API key
6. make the first authenticated gateway call

## Before You Start

Make sure you already completed:

- [Installation](/getting-started/installation)

You need:

- Rust + Cargo
- Node.js 20+
- pnpm 10+

## Step 1: Start the Full Stack

The recommended quickstart path uses the managed development scripts because they:

- avoid common `808x` conflicts by default
- bring up the built-in unified web host automatically
- print the clickable URLs and seeded credentials after startup

Linux or macOS:

```bash
./bin/start-dev.sh
```

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1
```

Default managed development URLs:

- unified admin app: `http://127.0.0.1:9983/admin/`
- unified portal app: `http://127.0.0.1:9983/portal/`
- unified gateway health: `http://127.0.0.1:9983/api/v1/health`
- direct gateway health: `http://127.0.0.1:9980/health`
- direct admin health: `http://127.0.0.1:9981/admin/health`
- direct portal health: `http://127.0.0.1:9982/portal/health`

Seeded local credentials:

- admin: `admin@sdkwork.local / ChangeMe123!`
- portal: `portal@sdkwork.local / ChangeMe123!`

If you specifically want the standalone Vite frontends instead of the unified web host, use:

- `./bin/start-dev.sh --browser`
- `powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1 -Browser`

## Step 2: Verify the Runtime Is Healthy

```bash
curl http://127.0.0.1:9980/health
curl http://127.0.0.1:9981/admin/health
curl http://127.0.0.1:9982/portal/health
```

Expected result: each endpoint returns `ok`.

## Step 3: Log In To The Admin Control Plane

The admin API seeds a default local operator account on first use:

- email: `admin@sdkwork.local`
- password: `ChangeMe123!`

```bash
curl -X POST http://127.0.0.1:9981/admin/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email":"admin@sdkwork.local",
    "password":"ChangeMe123!"
  }'
```

Then inspect the current admin identity:

```bash
export ADMIN_JWT="<paste token>"
curl http://127.0.0.1:9981/admin/auth/me \
  -H "Authorization: Bearer $ADMIN_JWT"
```

In the browser, the admin UI is available at:

- `http://127.0.0.1:9983/admin/`

## Step 4: Log In To The Portal

The public portal seeds a demo local account:

- email: `portal@sdkwork.local`
- password: `ChangeMe123!`

```bash
curl -X POST http://127.0.0.1:9982/portal/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email":"portal@sdkwork.local",
    "password":"ChangeMe123!"
  }'
```

Save the returned token:

```bash
export PORTAL_JWT="<paste token>"
```

In the browser, the portal UI is available at:

- `http://127.0.0.1:9983/portal/`

## Step 5: Inspect The Portal Workspace

```bash
curl http://127.0.0.1:9982/portal/workspace \
  -H "Authorization: Bearer $PORTAL_JWT"
```

This confirms the default workspace bootstrap for the local portal account.

## Step 6: Create A Gateway API Key

```bash
curl -X POST http://127.0.0.1:9982/portal/api-keys \
  -H "Authorization: Bearer $PORTAL_JWT" \
  -H "Content-Type: application/json" \
  -d '{"environment":"live"}'
```

The response returns a `plaintext` key one time. Save it immediately:

```bash
export GATEWAY_KEY="<paste plaintext key>"
```

## Step 7: Make The First Gateway Call

Use the key against the OpenAI-compatible gateway:

```bash
curl http://127.0.0.1:9980/v1/models \
  -H "Authorization: Bearer $GATEWAY_KEY"
```

Expected result:

- `200 OK`
- a standard OpenAI-style list response
- `data` may be empty until you configure catalog models and providers through the admin API

## Step 8: Stop The Managed Development Runtime

Linux or macOS:

```bash
./bin/stop-dev.sh
```

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\stop-dev.ps1
```

## Where To Go Next

- full script responsibilities and lifecycle:
  - [Script Lifecycle](/getting-started/script-lifecycle)
- source-native startup options:
  - [Source Development](/getting-started/source-development)
- understand the three HTTP surfaces:
  - [API Reference Overview](/api-reference/overview)
- configure models, providers, credentials, and routing:
  - [Admin API](/api-reference/admin-api)
- understand the runtime shape:
  - [Software Architecture](/architecture/software-architecture)
- compile binaries or frontend artifacts:
  - [Build and Packaging](/getting-started/build-and-packaging)
