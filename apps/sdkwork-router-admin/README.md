# SDKWork Router Admin

`sdkwork-router-admin` is the standalone React and Tauri workspace for the SDKWork Router control plane.

## Architecture Goals

- use `@sdkwork/ui-pc-react` as the single shared UI foundation
- keep the root app thin and move real composition into workspace packages
- remove legacy local UI layers and avoid compatibility baggage
- ship one consistent desktop shell across overview, users, tenants, gateway, catalog, traffic, operations, and settings
- operate directly on live admin control-plane data

## Workspace Layout

```text
apps/sdkwork-router-admin/
|- src/        # root app bootstrap only
|- packages/   # foundation and business modules
|- tests/      # architecture and product-surface verification
|- src-tauri/  # desktop host and native commands
`- dist/       # production build output
```

## Package Map

### Foundation

- `sdkwork-router-admin-types`
- `sdkwork-router-admin-core`
- `sdkwork-router-admin-shell`
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
- `sdkwork-router-admin-settings`
- `sdkwork-router-admin-apirouter`

## UI Standard

- shared styles come from `@sdkwork/ui-pc-react/styles.css`
- shell composition uses `DesktopShellFrame`, `NavigationRail`, `Toolbar`, `SettingsCenter`, and other shared primitives
- local shell CSS is limited to host layout selectors in `packages/sdkwork-router-admin-shell/src/styles/shell-host.css`
- theme state, locale, sidebar behavior, and workspace continuity are owned by `sdkwork-router-admin-core`

## Product Surfaces

- `Overview`: platform posture, alerts, leaderboard signals, and hotspot summaries
- `Users`: operator and portal identity CRUD, activation control, and lifecycle actions
- `Tenants`: tenant and project management plus project-scoped gateway key issuance
- `Coupons`: campaign management and activation posture
- `API Router`: gateway access keys, route configuration, model mappings, and usage records
- `Catalog`: channels, providers, credentials, model publications, and pricing rows
- `Traffic`: request visibility, usage rollups, CSV export, user leaderboard, and project hotspots
- `Operations`: provider health, runtime posture, and reload workflows
- `Settings`: shared settings center for locale, appearance, navigation, and workspace posture

## Shell Model

- `react-router-dom` browser routes mounted under `/admin/`
- authenticated desktop shell with a shared top toolbar, left navigation rail, and right content canvas
- login isolated outside the authenticated shell
- persisted theme mode, accent preset, sidebar width, collapse behavior, and hidden routes
- lazy-loaded page modules behind a shared loading state

## Commands

```bash
pnpm install
pnpm typecheck
pnpm build
pnpm dev
pnpm tauri:dev
pnpm tauri:build
```

The Vite dev server proxies `/api/admin/*` to `http://127.0.0.1:8081/admin/*`.

## Desktop Host

The desktop app uses the shared `sdkwork-api-product-runtime` instead of a hard-coded local web host.

That keeps the admin desktop app aligned with the server-delivered `/admin/*` contract while preserving native IPC commands for router operations.

## Relationship To Server Mode

Server mode remains owned by `apps/sdkwork-router-portal` through `pnpm server:start`, which launches `router-product-service`.

Use this workspace when you need:

- a local operator desktop shell
- bundled admin assets inside the Tauri host
- direct control-plane operations in a standalone app boundary
