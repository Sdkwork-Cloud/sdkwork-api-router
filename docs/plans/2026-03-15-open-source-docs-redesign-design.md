# Open Source Documentation Redesign Design

**Date:** 2026-03-15

**Status:** Approved by the user's standing instruction to proceed autonomously without waiting for interactive checkpoints

## Context

The repository already ships a VitePress documentation site, but the current information architecture is still too shallow for a professional open source product:

- navigation is weighted toward operational pages and historical references instead of a complete product map
- architecture guidance is fragmented across one runtime-modes page and many historical plan documents
- API reference material is split between a compatibility summary and a matrix, without a durable reference entry point for gateway, admin, and portal surfaces
- build and packaging instructions exist, but they are distributed across README files and a release page rather than presented as a coherent compilation story
- Chinese and English content cover only a subset of the material needed for onboarding, architecture review, and day-two operations

The codebase, however, already contains enough verified implementation detail to support a much stronger documentation product:

- three standalone services with hot-reload-aware runtime wiring
- a broad OpenAI-compatible `/v1/*` gateway surface
- native `/admin/*` and `/portal/*` control-plane APIs
- a browser and Tauri console
- explicit script entry points for local startup and packaging
- a well-structured workspace split across interface, app, domain, storage, provider, and runtime crates

## Goal

Upgrade `docs/` into a professional open source documentation surface that feels closer to the structure and clarity of OpenAI's official docs while remaining grounded in the actual SDKWork implementation.

The new docs should make it easy for a reader to answer:

- what this project is
- how to run it locally
- how to compile and package it
- how the architecture is assembled
- which functional modules exist
- which APIs are exposed and how they are grouped
- which operational and compatibility constraints apply

## Reference Model

OpenAI's official documentation is a good structural reference because it separates durable documentation concerns instead of mixing them:

- getting started
- conceptual or architectural guidance
- API reference
- operational or scaling guidance
- supporting reference material

SDKWork should adopt the same structural discipline, but with repository-specific content:

- `Getting Started`
- `Architecture`
- `API Reference`
- `Operations`
- `Reference`

## Approaches

### Option A: Patch the current pages in place without changing nav structure

Pros:

- lowest immediate effort
- minimal sidebar churn

Cons:

- does not solve the core information architecture problem
- keeps architecture, API reference, and compilation guidance under-exposed
- still feels like a small ops guide rather than a full product documentation system

### Option B: Rebuild the docs navigation and add missing core pages while preserving existing deep references

Pros:

- improves the first-run reader experience immediately
- keeps current URLs and deep reference pages where possible
- allows architecture, API reference, and compilation docs to become first-class sections
- scales cleanly for future parity and deployment work

Cons:

- requires updating both English and Chinese docs navigation
- adds several new pages that must stay synchronized

### Option C: Generate a full machine-derived API reference from route registration

Pros:

- strongest long-term API precision

Cons:

- larger effort than this docs pass
- risks over-indexing on endpoint inventory before the core site structure is fixed
- still needs human-written getting-started and architecture material

## Recommendation

Choose **Option B**.

This is the highest-value move now: fix the documentation product structure first, add curated API reference pages, and keep the existing compatibility matrix as the deeper truth source.

## Target Information Architecture

### Top-Level Navigation

- `Getting Started`
- `Architecture`
- `API Reference`
- `Operations`
- `Reference`
- locale switch

### Getting Started

Purpose: help new contributors and operators succeed quickly.

Target pages:

- installation and prerequisites
- source development
- build and packaging
- release builds
- runtime modes
- public portal

### Architecture

Purpose: explain how the system is assembled and where responsibilities live.

Target pages:

- software architecture
- functional modules
- runtime modes detail

### API Reference

Purpose: give a durable entry point similar to OpenAI's API reference hierarchy.

Target pages:

- API reference overview
- gateway API reference
- admin API reference
- portal API reference

The compatibility summary and full matrix remain linked from these pages instead of acting as the only reference surface.

### Operations

Purpose: day-two runtime behavior and troubleshooting.

Target pages:

- configuration
- health and metrics

### Reference

Purpose: supporting material and durable repo facts.

Target pages:

- API compatibility
- repository layout
- build and tooling reference
- compatibility matrix

## Content Design Principles

- lead each page with reader intent, not internal history
- keep commands copy-paste ready for Windows and Linux or macOS
- separate "source run" from "compile or package" instead of mixing them
- document the actual standalone services and routes that exist today
- use tables for endpoint grouping, ports, binaries, and module boundaries
- keep historical plan documents out of the main onboarding path
- maintain bilingual parity for the top-level product documentation

## API Reference Strategy

This pass should not attempt field-by-field request schema duplication for every OpenAI-compatible endpoint. That would be expensive to maintain and redundant with upstream-compatible schemas.

Instead, the API reference should document:

- base URLs and auth boundaries
- service ownership
- route families and nested resources
- capability notes
- mode semantics where they matter
- cross-links to compatibility truth

This yields a practical open source API reference without pretending to be a separate generated schema portal.

## Scope

This redesign should include:

- new or rewritten home pages
- updated VitePress nav and sidebars
- new architecture pages
- new API reference pages
- a dedicated build and packaging page
- a build and tooling reference page
- Chinese mirrors for the primary pages
- refreshed README files only if needed to point more clearly into the site

## Non-Goals

This pass should not:

- generate OpenAPI specifications
- rewrite every historical design note
- promise support for routes or deployment modes not present in code
- replace the compatibility matrix with a much larger table system

## Verification Strategy

Use concrete verification:

1. build the docs site after the navigation and page changes
2. inspect for broken local links through the VitePress build
3. confirm that new architecture and API reference pages are reachable from the sidebar
4. verify that the English and Chinese top-level sections remain symmetrical
