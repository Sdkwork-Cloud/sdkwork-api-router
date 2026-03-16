# Quickstart

This page is the fastest path from clone to a verified local SDKWork stack.

It follows the same structure that mature API platforms use for onboarding:

1. start the runtime
2. verify the control plane
3. create an end-user account
4. issue a gateway API key
5. make a first authenticated gateway call

## Before You Start

Make sure you already completed:

- [Installation](/getting-started/installation)

You need:

- Rust + Cargo
- Node.js 20+
- pnpm 10+

## Step 1: Start the Full Stack

Linux or macOS:

```bash
node scripts/dev/start-workspace.mjs
```

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1
```

Default local services:

- gateway: `http://127.0.0.1:8080`
- admin: `http://127.0.0.1:8081`
- portal: `http://127.0.0.1:8082`
- landing: `http://127.0.0.1:5173/`
- admin app: `http://127.0.0.1:5173/admin/`
- portal app: `http://127.0.0.1:5174/`

## Step 2: Verify the Runtime Is Healthy

```bash
curl http://127.0.0.1:8080/health
curl http://127.0.0.1:8081/admin/health
curl http://127.0.0.1:8082/portal/health
```

Expected result: each endpoint returns `ok`.

## Step 3: Log In to the Admin Control Plane

The admin API seeds a default local operator account on first use:

- email: `admin@sdkwork.local`
- password: `ChangeMe123!`

```bash
curl -X POST http://127.0.0.1:8081/admin/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email":"admin@sdkwork.local",
    "password":"ChangeMe123!"
  }'
```

Then inspect the current admin identity:

```bash
export ADMIN_JWT="<paste token>"
curl http://127.0.0.1:8081/admin/auth/me \
  -H "Authorization: Bearer $ADMIN_JWT"
```

This validates that the control plane is reachable and issuing JWTs correctly. In the browser, the
operator UI now lives at `http://127.0.0.1:5173/admin/`.

## Step 4: Register a Portal User or Use the Default Portal Account

The public portal is the self-service boundary for end users. Local development also seeds a demo
portal account:

- email: `portal@sdkwork.local`
- password: `ChangeMe123!`

Default login:

```bash
curl -X POST http://127.0.0.1:8082/portal/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email":"portal@sdkwork.local",
    "password":"ChangeMe123!"
  }'
```

Or create a new user:

```bash
curl -X POST http://127.0.0.1:8082/portal/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email":"portal@example.com",
    "password":"hunter2!",
    "display_name":"Portal User"
  }'
```

The response includes:

- a portal JWT
- the created user
- the default tenant and project workspace

Save the returned token:

```bash
export PORTAL_JWT="<paste token>"
```

## Step 5: Inspect the Portal Workspace

```bash
curl http://127.0.0.1:8082/portal/workspace \
  -H "Authorization: Bearer $PORTAL_JWT"
```

This confirms the default workspace bootstrap for the newly created user.

## Step 6: Create a Gateway API Key

```bash
curl -X POST http://127.0.0.1:8082/portal/api-keys \
  -H "Authorization: Bearer $PORTAL_JWT" \
  -H "Content-Type: application/json" \
  -d '{"environment":"live"}'
```

The response returns a `plaintext` key one time. Save it immediately:

```bash
export GATEWAY_KEY="<paste plaintext key>"
```

## Step 7: Make the First Gateway Call

Use the key against the OpenAI-compatible gateway:

```bash
curl http://127.0.0.1:8080/v1/models \
  -H "Authorization: Bearer $GATEWAY_KEY"
```

Expected result:

- `200 OK`
- a standard OpenAI-style list response
- `data` may be empty until you configure catalog models and providers through the admin API

## Where To Go Next

- open the browser apps:
  - admin: `http://127.0.0.1:5173/admin/`
  - portal: `http://127.0.0.1:5174/`
- understand the three HTTP surfaces:
  - [API Reference Overview](/api-reference/overview)
- configure models, providers, credentials, and routing:
  - [Admin API](/api-reference/admin-api)
- understand the runtime shape:
  - [Software Architecture](/architecture/software-architecture)
- compile binaries or frontend artifacts:
  - [Build and Packaging](/getting-started/build-and-packaging)
