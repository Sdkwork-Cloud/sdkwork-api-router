# Multimodal Canonical Request Pricing Batch 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** move the first fixed-price multimodal routes onto the canonical account hold/capture/release path so commercial billing evidence is no longer limited to chat-style routes.

**Architecture:** reuse the existing canonical account kernel primitives in `sdkwork-api-interface-http` instead of creating a second media billing path. Introduce a request-priced canonical admission helper for fixed-request media endpoints, then wire image and audio fixed-price routes through it while preserving the existing usage-record and billing-event writes.

**Tech Stack:** Rust, Axum, sqlx, SQLite, cargo test

---

### Task 1: Lock the behavior with multimodal canonical settlement tests

**Files:**
- Modify: `crates/sdkwork-api-interface-http/tests/canonical_account_admission.rs`

- [x] **Step 1: Add a failing image-route test proving canonical holds capture on successful fixed-price media requests**
- [x] **Step 2: Run `cargo test -p sdkwork-api-interface-http stateful_images_route_captures_canonical_account_hold_from_request_price -- --nocapture` and verify it fails for the expected reason**
- [x] **Step 3: Add a failing transcription-route test proving canonical holds capture on successful fixed-price audio requests**
- [x] **Step 4: Run `cargo test -p sdkwork-api-interface-http stateful_audio_transcriptions_route_captures_canonical_account_hold_from_request_price -- --nocapture` and verify it fails for the expected reason**

### Task 2: Implement request-priced canonical media admission

**Files:**
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`

- [x] **Step 1: Add a canonical admission helper that estimates charge from `ModelPriceRecord.request_price` for fixed-price routes**
- [x] **Step 2: Wire `/v1/images/generations` through canonical hold/capture/release while preserving existing usage record writes**
- [x] **Step 3: Wire `/v1/audio/transcriptions`, `/v1/audio/translations`, and `/v1/audio/speech` through the same canonical hold/capture/release path**
- [x] **Step 4: Keep legacy behavior as fallback when canonical subject, account kernel, or request-priced model price is unavailable**

### Task 3: Verify and tighten the batch

**Files:**
- Modify: `crates/sdkwork-api-interface-http/tests/canonical_account_admission.rs`
- Modify: `docs/superpowers/plans/2026-04-04-commercial-hardening-and-launch-plan.md`

- [x] **Step 1: Run `cargo test -p sdkwork-api-interface-http stateful_images_route_captures_canonical_account_hold_from_request_price -- --nocapture`**
- [x] **Step 2: Run `cargo test -p sdkwork-api-interface-http stateful_audio_transcriptions_route_captures_canonical_account_hold_from_request_price -- --nocapture`**
- [x] **Step 3: Run `cargo test -p sdkwork-api-interface-http canonical_account_admission -- --nocapture`**
- [x] **Step 4: Update the commercial hardening plan status notes for the completed portion of Task 5**
