# Rust Audit Governance Alignment Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** align the Rust dependency-audit policy, CI gate, and historical hardening documents with the now-clean workspace audit result.

**Architecture:** keep the runtime dependency graph untouched and treat this as a governance-closure slice. The implementation should remove the stale `RUSTSEC-2026-0097` exception, make the PR workflow execute the governance tests it already watches, and update the affected hardening docs so they no longer describe closed RustSec items as current unresolved debt.

**Tech Stack:** Node test runner, GitHub Actions workflow YAML, Cargo audit, Rust workspace lockfile governance docs.

---

## File Map

- Modify: `scripts/check-rust-dependency-audit.policy.json`
  - remove the stale `RUSTSEC-2026-0097` allowlist entry so the policy matches the active audit result.
- Modify: `scripts/check-rust-dependency-audit.test.mjs`
  - keep the policy regression focused on an empty allowlist when the workspace audit is warning-free.
- Modify: `.github/workflows/rust-verification.yml`
  - make the `dependency-audit` lane run the governance Node tests in addition to the matrix audit script.
- Modify: `scripts/rust-verification-workflow.test.mjs`
  - assert that the workflow actually runs the governance tests it claims to protect.
- Modify: `docs/superpowers/specs/2026-04-15-openapi-paste-retirement-design.md`
  - rewrite stale current-tense security statements so they reflect closure instead of live debt.
- Modify: `docs/superpowers/plans/2026-04-15-openapi-paste-retirement.md`
  - remove stale residual-debt wording that still lists `paste` and `rand 0.8.5` as current unresolved items.
- Modify: `docs/superpowers/plans/2026-04-15-pingora-daemon-mode-retirement.md`
  - update residual-risk language so it no longer reports already-closed `paste` and `rand` warnings as current.

### Task 1: Re-anchor the policy regression on the clean audit state

**Files:**
- Modify: `scripts/check-rust-dependency-audit.test.mjs`
- Modify: `scripts/check-rust-dependency-audit.policy.json`

- [ ] **Step 1: Tighten the failing policy regression**

Update the policy-focused test so its description and assertions clearly express the intended rule: when the workspace audit is warning-free, `allowedWarnings` must be an empty array.

Target assertion shape:

```js
assert.deepEqual(auditPolicy.allowedWarnings, []);
```

- [ ] **Step 2: Run the narrow dependency-audit test to confirm it fails before the policy fix**

Run:

```bash
node --test scripts/check-rust-dependency-audit.test.mjs
```

Expected: FAIL in the policy test because `scripts/check-rust-dependency-audit.policy.json` still contains `RUSTSEC-2026-0097`.

- [ ] **Step 3: Remove the stale allowlist entry**

Rewrite `scripts/check-rust-dependency-audit.policy.json` to:

```json
{
  "allowedWarnings": []
}
```

- [ ] **Step 4: Re-run the narrow dependency-audit test**

Run:

```bash
node --test scripts/check-rust-dependency-audit.test.mjs
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add scripts/check-rust-dependency-audit.policy.json scripts/check-rust-dependency-audit.test.mjs
git commit -m "test: align rust audit policy with clean graph"
```

### Task 2: Make CI execute the governance tests it already watches

**Files:**
- Modify: `scripts/rust-verification-workflow.test.mjs`
- Modify: `.github/workflows/rust-verification.yml`

- [ ] **Step 1: Add the failing workflow assertion first**

Extend `scripts/rust-verification-workflow.test.mjs` to require a dedicated dependency-audit workflow step that runs:

```bash
node --test scripts/check-rust-dependency-audit.test.mjs scripts/check-rust-verification-matrix.test.mjs scripts/rust-verification-workflow.test.mjs
```

- [ ] **Step 2: Run the workflow test to confirm it fails before the workflow change**

Run:

```bash
node --test scripts/rust-verification-workflow.test.mjs
```

Expected: FAIL because `.github/workflows/rust-verification.yml` currently does not execute the governance Node tests.

- [ ] **Step 3: Add the workflow gate**

Update `.github/workflows/rust-verification.yml` so the `dependency-audit` lane runs a dedicated Node test step before the existing matrix verification step. Keep the existing `node scripts/check-rust-verification-matrix.mjs --group ${{ matrix.group }}` step intact.

Expected workflow command:

```bash
node --test scripts/check-rust-dependency-audit.test.mjs scripts/check-rust-verification-matrix.test.mjs scripts/rust-verification-workflow.test.mjs
```

- [ ] **Step 4: Re-run the workflow test**

Run:

```bash
node --test scripts/rust-verification-workflow.test.mjs
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add .github/workflows/rust-verification.yml scripts/rust-verification-workflow.test.mjs
git commit -m "ci: run rust audit governance tests"
```

### Task 3: Correct the stale hardening documents

**Files:**
- Modify: `docs/superpowers/specs/2026-04-15-openapi-paste-retirement-design.md`
- Modify: `docs/superpowers/plans/2026-04-15-openapi-paste-retirement.md`
- Modify: `docs/superpowers/plans/2026-04-15-pingora-daemon-mode-retirement.md`

- [ ] **Step 1: Review the stale statements to be corrected**

Run:

```bash
rg -n "two informational warnings|paste 1\\.0\\.15|rand 0\\.8\\.5|remaining unresolved|remaining unresolved advisories" docs/superpowers/specs/2026-04-15-openapi-paste-retirement-design.md docs/superpowers/plans/2026-04-15-openapi-paste-retirement.md docs/superpowers/plans/2026-04-15-pingora-daemon-mode-retirement.md
```

Expected: matches show the outdated current-tense statements that still describe already-closed warnings as active debt.

- [ ] **Step 2: Rewrite the affected sections in factual closure language**

Make the docs say, in substance:

- `paste` retirement is complete
- daemon mode retirement is complete
- the active workspace audit graph is now clean
- any mention of `rand 0.8.5` or `paste 1.0.15` is historical implementation context, not current residual risk

- [ ] **Step 3: Re-run the targeted drift scan**

Run:

```bash
rg -n "remaining unresolved advisories|`RUSTSEC-2026-0097` via|`paste 1\\.0\\.15` via|`rand 0\\.8\\.5` via" docs/superpowers/specs/2026-04-15-openapi-paste-retirement-design.md docs/superpowers/plans/2026-04-15-openapi-paste-retirement.md docs/superpowers/plans/2026-04-15-pingora-daemon-mode-retirement.md
```

Expected: no matches.

- [ ] **Step 4: Commit**

```bash
git add docs/superpowers/specs/2026-04-15-openapi-paste-retirement-design.md docs/superpowers/plans/2026-04-15-openapi-paste-retirement.md docs/superpowers/plans/2026-04-15-pingora-daemon-mode-retirement.md
git commit -m "docs: align rust hardening closure notes"
```

### Task 4: Run the full governance closure verification

**Files:**
- Modify: none unless verification exposes a real defect

- [ ] **Step 1: Run the governance Node test suite**

Run:

```bash
node --test scripts/check-rust-dependency-audit.test.mjs scripts/check-rust-verification-matrix.test.mjs scripts/rust-verification-workflow.test.mjs
```

Expected: PASS.

- [ ] **Step 2: Run the dependency-audit matrix lane exactly as CI will**

Run:

```bash
node scripts/check-rust-verification-matrix.mjs --group dependency-audit
```

Expected: PASS.

- [ ] **Step 3: Re-run the authoritative audit command**

Run:

```bash
cargo audit --json --no-fetch --stale
```

Expected: JSON result with:

- `"found": false`
- `"count": 0`
- no `warnings.unsound`, `warnings.notice`, or `warnings.unmaintained` entries

- [ ] **Step 4: Confirm the policy remains empty after final verification**

Run:

```bash
Get-Content scripts/check-rust-dependency-audit.policy.json
```

Expected:

```json
{
  "allowedWarnings": []
}
```

- [ ] **Step 5: Commit any verification-driven follow-up**

```bash
git status --short
```

Expected: no unexpected diffs beyond the planned governance-alignment files. If a verification fix was required, commit it with a focused message before handoff.
