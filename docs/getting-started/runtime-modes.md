# Runtime Modes

SDKWork API Server supports two primary runtime shapes: standalone services and an embedded desktop-oriented model.

## Standalone Server Mode

Standalone mode is the shared-service deployment shape.

Characteristics:

- services run as independent binaries
- gateway, admin, and portal APIs are exposed over HTTP
- PostgreSQL is the preferred deployment database
- upstream credentials are expected to be managed by a server-side secret backend

Choose this mode when:

- you want a browser-accessible shared environment
- you need multiple operators or portal users
- you are deploying behind a reverse proxy or service manager

## Embedded Mode

Embedded mode is the desktop-oriented deployment shape.

Characteristics:

- the runtime can be hosted in-process through the runtime host abstraction
- the Tauri shell hosts the operator console while the portal stays browser-first
- loopback binding is the default trust boundary
- SQLite is the preferred local persistence strategy

Choose this mode when:

- you want a desktop-first operator experience
- you are running locally on a single machine
- you want the admin console to be available in both browser and desktop form

## Browser and Tauri Together

In development:

- `pnpm --dir console tauri:dev` uses the Vite dev server
- the same admin Vite URL stays reachable from a browser
- `start-workspace --tauri` keeps backend services plus the desktop shell in one startup flow while the portal continues on `http://127.0.0.1:5174/`

## Where To Go Next

- onboarding and local startup:
  - [Source Development](/getting-started/source-development)
- compilation and packaging:
  - [Build and Packaging](/getting-started/build-and-packaging)
- deep architecture and runtime supervision:
  - [Runtime Modes Deep Dive](/architecture/runtime-modes)
