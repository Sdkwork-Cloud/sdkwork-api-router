# SDKWork Router Admin

`sdkwork-router-admin` is a standalone React workspace for the SDKWork Router super-admin surface.

## Goals

- independent engineering project under `apps/`
- product-grade super-admin UI for daily router operations
- follows `ARCHITECT.md` package ownership and composition rules
- ships as a standalone React workspace with its own theme, routing shell, and package graph
- operates on live admin-control-plane data for day-to-day management, audit, and runtime posture

## Workspace Layout

```text
apps/sdkwork-router-admin/
├── src/        # root app shell composition only
├── packages/   # reusable foundation and business modules
├── tests/      # structure and architecture checks
└── dist/       # production build output
```

## Package Map

### Foundation

- `sdkwork-router-admin-types`
- `sdkwork-router-admin-commons`
- `sdkwork-router-admin-core`
- `sdkwork-router-admin-admin-api`

### Business

- `sdkwork-router-admin-auth`
- `sdkwork-router-admin-overview`
- `sdkwork-router-admin-users`
- `sdkwork-router-admin-tenants`
- `sdkwork-router-admin-coupons`
- `sdkwork-router-admin-catalog`
- `sdkwork-router-admin-traffic`
- `sdkwork-router-admin-operations`

## Data Sources

- live admin API:
  - operator users
  - portal users
  - tenants
  - projects
  - gateway keys
  - channels
  - providers
  - provider credentials
  - models
  - coupons
  - usage records
  - billing summary
  - routing decision logs
  - provider health
  - runtime status

## Product Surfaces

- `Overview`: global posture, operator alerts, top portal users, and hottest projects
- `Users`: operator and portal user CRUD, password rotation, status management, and usage visibility
- `Tenants`: tenant/project CRUD, gateway key issuance, revoke/restore, deletion, and workspace ownership checks
- `Coupons`: live campaign CRUD and activation posture
- `Catalog`: channel, proxy provider, provider credential, and model CRUD with credential coverage and secret rotation workflow
- `Traffic`: multi-filter request query console, CSV export, usage records, billing rollups, request-log visibility, user traffic leaderboard, and project hotspots
- `Operations`: provider health, runtime posture, and runtime reload controls

## Commands

```bash
pnpm install
pnpm typecheck
pnpm build
pnpm dev
```

The Vite dev server proxies `/api/admin/*` to `http://127.0.0.1:8081/admin/*`.
