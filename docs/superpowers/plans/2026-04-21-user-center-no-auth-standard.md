# User Center No-Auth Standard Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the top-level `auth` contract from the shared user-center package and router portal bridge, and standardize the public API around `mode`, `provider`, `integration`, `routes`, `storagePlan`, and `storageTopology`.

**Architecture:** Rewrite the shared appbase `@sdkwork/user-center-pc-react` contract so runtime and bridge configs expose only the standard route, provider, integration, and storage surfaces. Keep canonical token and handshake headers as derived runtime helpers, and move provider-specific secret/signature headers behind `resolveAuthHeaders` at runtime instead of configuration-time models.

**Tech Stack:** TypeScript, Node test runner, Vitest, jiti, Tauri Rust contract checks

---

### Task 1: Lock the public contract with failing tests

**Files:**
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-appbase\packages\pc-react\identity\sdkwork-user-center-pc-react\tests\userCenterConfig.test.ts`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-appbase\packages\pc-react\identity\sdkwork-user-center-pc-react\tests\userCenterPlugin.test.ts`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-appbase\packages\pc-react\identity\sdkwork-user-center-pc-react\tests\userCenterNodeContract.test.ts`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-user-center-standard.test.mjs`

- [ ] Step 1: Update the tests to assert there is no public top-level `auth` field on config, bridge, or plugin outputs.
- [ ] Step 2: Update the tests to assert simplified integration profiles without `authMode`, `handshakeEnabled`, `secretResolverKind`, or `validationStrategy`.
- [ ] Step 3: Run the targeted tests and confirm they fail for the expected old-auth reasons.

Run:
```powershell
pnpm exec vitest run packages/pc-react/identity/sdkwork-user-center-pc-react/tests/userCenterConfig.test.ts packages/pc-react/identity/sdkwork-user-center-pc-react/tests/userCenterPlugin.test.ts
node --test packages/pc-react/identity/sdkwork-user-center-pc-react/tests/userCenterNodeContract.test.ts
node --test apps/sdkwork-router-portal/tests/portal-user-center-standard.test.mjs
```

### Task 2: Rewrite the shared user-center public contract

**Files:**
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-appbase\packages\pc-react\identity\sdkwork-user-center-pc-react\src\types\userCenterTypes.ts`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-appbase\packages\pc-react\identity\sdkwork-user-center-pc-react\src\domain\userCenterStandard.ts`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-appbase\packages\pc-react\identity\sdkwork-user-center-pc-react\src\domain\userCenterConfig.ts`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-appbase\packages\pc-react\identity\sdkwork-user-center-pc-react\src\domain\userCenterBridge.ts`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-appbase\packages\pc-react\identity\sdkwork-user-center-pc-react\src\domain\userCenterPlugin.ts`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-appbase\packages\pc-react\identity\sdkwork-user-center-pc-react\src\domain\userCenterRuntimeClient.ts`

- [ ] Step 1: Remove top-level auth input/output types from runtime, bridge, and plugin public interfaces.
- [ ] Step 2: Simplify integration profile types and normalization logic to the standard `builtin-local` and `spring-ai-plus-app-api` shapes.
- [ ] Step 3: Derive canonical runtime handshake headers from `mode`, `provider`, and `integration` instead of a normalized auth contract.
- [ ] Step 4: Keep token storage and runtime header helpers working with the canonical token bundle store.

### Task 3: Rewrite the router portal bridge to follow the shared standard

**Files:**
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-types\src\userCenter.ts`

- [ ] Step 1: Remove portal bridge support for `options.auth` and stop re-exporting or synthesizing `auth` on returned objects.
- [ ] Step 2: Keep portal-specific route patching for auth manifests only, without reintroducing a public auth contract.
- [ ] Step 3: Ensure portal runtime helpers still consume storage plan and token-store utilities from the canonical package.

### Task 4: Verify and harden the standard

**Files:**
- Modify if needed: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-appbase\packages\pc-react\identity\sdkwork-user-center-pc-react\README.md`
- Modify if needed: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-user-center-standard.test.mjs`

- [ ] Step 1: Run targeted appbase and router tests until green.
- [ ] Step 2: Search for stale public-auth patterns and remove any remaining old contract residue.
- [ ] Step 3: Re-run the shared contract and router product checks that cover this surface.

Run:
```powershell
pnpm exec vitest run packages/pc-react/identity/sdkwork-user-center-pc-react/tests/userCenterConfig.test.ts packages/pc-react/identity/sdkwork-user-center-pc-react/tests/userCenterPlugin.test.ts
node scripts/run-user-center-standard-contracts.mjs
node --test apps/sdkwork-router-portal/tests/portal-user-center-standard.test.mjs
```
