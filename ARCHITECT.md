# SDKWork Architecture Standard

## Purpose

This document defines a reusable architecture standard for SDKWork frontend projects.

It is based on the current `magic-studio-v2` implementation, but the rules are generalized so the same standard can be reused to create other business projects such as:

- `sdkwork-game-core`
- `sdkwork-game-user`
- `sdkwork-trade-order`
- `sdkwork-education-course`

The goal is to keep every project:

- modular
- easy to scale
- easy to maintain
- consistent across businesses

## Standard Stack

The standard frontend stack is:

- `pnpm` for monorepo package management
- `Turbo` for multi-package task orchestration
- `React` for UI
- `TypeScript` for type safety
- `Vite` for development and build
- `Tauri` for desktop host integration when desktop capability is required

## Monorepo Standard

Each business project should use a `pnpm workspace` monorepo.

Standard root structure:

```text
project-root/
├── src/                # App shell and composition layer
├── packages/           # Reusable business and foundation packages
├── src-tauri/          # Optional desktop-native host
├── scripts/            # Build, audit, and release scripts
├── docs/               # Architecture and design documents
├── package.json        # Root scripts
├── pnpm-workspace.yaml # Workspace definition
├── turbo.json          # Task pipeline
├── tsconfig.json       # Root TypeScript config
└── vite.config.ts      # Root Vite config
```

Reference from the current project:

- root `src/` is the application shell
- `packages/` contains reusable modules
- `src-tauri/` is the desktop-native boundary

## Package Naming Standard

All packages must use this naming format:

```text
sdkwork-<project>-<module>
```

Examples:

- `sdkwork-game-core`
- `sdkwork-game-user`
- `sdkwork-game-auth`
- `sdkwork-game-trade`
- `sdkwork-game-notification`

Naming rules:

- `sdkwork` is the fixed organization prefix
- `<project>` is the business project name
- `<module>` is the package responsibility
- use lowercase letters and hyphens only
- one package should represent one clear bounded context

## Module Classification Standard

Modules should be divided into two categories.

### Foundation Modules

These modules are reusable and cross-cutting:

- `sdkwork-<project>-types`
- `sdkwork-<project>-i18n`
- `sdkwork-<project>-commons`
- `sdkwork-<project>-core`

Recommended responsibilities:

- `types`: shared domain types and contracts
- `i18n`: localization resources and helpers
- `commons`: shared UI components, hooks, constants, and common utilities
- `core`: router helpers, platform abstraction, store factories, event bus, shared runtime services

### Business Modules

These modules represent concrete business capabilities:

- `sdkwork-<project>-user`
- `sdkwork-<project>-auth`
- `sdkwork-<project>-trade`
- `sdkwork-<project>-drive`
- `sdkwork-<project>-notes`
- `sdkwork-<project>-chat`

Recommended responsibilities:

- business pages
- business components
- business store
- business services
- business entities
- business-specific i18n assets

## Dependency Standard

Dependencies must remain directional and predictable.

Allowed dependency direction:

```text
types
  -> i18n / commons
  -> core
  -> business modules
  -> app shell
```

Detailed rules:

- `types` must not depend on business modules
- `commons` must not depend on business modules
- `core` may depend on `types`, `i18n`, `commons`, and infrastructure modules
- business modules may depend on `types`, `i18n`, `commons`, and `core`
- business modules should avoid depending on other business modules unless the ownership boundary is explicit
- the app shell may depend on all packages
- packages must never depend on the root app

## pnpm Package Standard

### Workspace Rules

- use `pnpm` as the only package manager
- register all local packages in `pnpm-workspace.yaml`
- use `workspace:*` for internal package dependencies
- use `catalog:` for shared third-party versions at the workspace level
- keep shared dependency versions centralized in the workspace catalog

Standard example:

```yaml
packages:
  - 'packages/*'

catalog:
  react: ^19.0.0
  react-dom: ^19.0.0
  typescript: ^5.0.0
  vite: ^7.0.0
```

### Package Rules

Each package should include:

- `package.json`
- `src/index.ts`
- `tsconfig.json`
- `vite.config.ts`

Each package should define:

- `dev`
- `build`
- `typecheck`

Recommended `package.json` shape:

```json
{
  "name": "sdkwork-game-user",
  "private": true,
  "type": "module",
  "main": "./src/index.ts",
  "module": "./src/index.ts",
  "types": "./src/index.ts",
  "exports": {
    ".": {
      "import": "./src/index.ts",
      "types": "./src/index.ts"
    }
  },
  "scripts": {
    "dev": "vite build --watch",
    "build": "vite build",
    "typecheck": "tsc --noEmit"
  }
}
```

## Directory Responsibility Standard

### Root `src/`

The root `src/` directory is the app shell, not the main business implementation layer.

It should contain only:

- app bootstrap
- route composition
- layout composition
- global theme
- root-level providers
- shell-only pages

It should not become a storage area for reusable business logic.

### `packages/`

The `packages/` directory is the main implementation area for reusable modules.

Each package should use a consistent internal structure.

Business packages must be self-contained. They must not rely on the root `src/`
or on another business package to provide their basic module structure.

Every business package must include these required directories:

- `types/`
- `components/`
- `repository/`
- `services/`
- `pages/`

These directories are optional and should be added only when the module actually
needs them:

- `store/`
- `hooks/`
- `i18n/`
- `domain/`

Recommended example:

```text
packages/sdkwork-game-user/
├── src/
│   ├── types/
│   ├── components/
│   ├── repository/
│   ├── services/
│   ├── pages/
│   ├── store/
│   ├── hooks/
│   ├── i18n/
│   ├── domain/
│   └── index.ts
├── package.json
├── tsconfig.json
└── vite.config.ts
```

Directory rules:

- `types/`: public and internal type contracts owned by the package
- `components/`: reusable UI inside the package
- `repository/`: storage access, persistence strategies, and data-source adapters
- `services/`: business orchestration, application services, and use-case logic
- `pages/`: route-level pages exported by the package
- `store/`: package-local state when UI state orchestration is needed
- `hooks/`: package-local React hooks that should not leak into `core`
- `i18n/`: package-local translations
- `domain/`: domain models, rules, value objects, and policy logic
- `index.ts`: public exports only

### `src-tauri/`

Use `src-tauri/` only when native desktop capability is needed.

It should contain:

- native commands
- filesystem access
- process and shell integration
- session management
- PTY support
- native event bridge

Rule:
Frontend packages should access desktop-native capability through service abstractions or platform abstractions, not by scattering Tauri-specific logic across business modules.

## Layered Package Recommendation

For a new project, the recommended minimum package set is:

```text
sdkwork-<project>-types
sdkwork-<project>-i18n
sdkwork-<project>-commons
sdkwork-<project>-core
sdkwork-<project>-auth
sdkwork-<project>-user
```

Additional business modules can then be added by domain:

- `sdkwork-<project>-trade`
- `sdkwork-<project>-notification`
- `sdkwork-<project>-drive`
- `sdkwork-<project>-chat`
- `sdkwork-<project>-workspace`

## Governance Rules

- one package, one main responsibility
- prefer adding a new bounded package over creating a generic miscellaneous module
- shared types must go to `types`
- shared UI must go to `commons`
- cross-cutting runtime capability must go to `core`
- business rules must stay in business modules
- keep public exports clean and stable
- avoid deep internal imports across packages
- avoid circular dependencies
- keep the root app thin and compositional

## Standard Workflow

Workspace commands:

```bash
pnpm install
pnpm build:packages
pnpm typecheck
pnpm lint
pnpm dev
```

Single package commands:

```bash
pnpm --filter sdkwork-game-user dev
pnpm --filter sdkwork-game-user build
pnpm --filter sdkwork-game-user typecheck
```

## Reuse Rule

When creating a new business project, do not redesign the architecture from scratch.

Reuse this standard directly:

1. keep the same monorepo structure
2. rename packages using `sdkwork-<project>-<module>`
3. keep the same layer responsibilities
4. keep the same dependency direction
5. extend by adding new business modules only where needed

This keeps new projects consistent with the current SDKWork architecture while remaining flexible for different business domains.
