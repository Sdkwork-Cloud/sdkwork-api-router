# Portal API

The portal service exposes the self-service user boundary under `/portal/*`.

## Base URL and Auth

- default local base URL: `http://127.0.0.1:8082/portal`
- health: `GET /portal/health`
- auth boundary: portal JWT, independent from admin JWT

Minimal registration example:

```bash
curl -X POST http://127.0.0.1:8082/portal/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email":"portal@example.com",
    "password":"hunter2!",
    "display_name":"Portal User"
  }'
```

Default local demo login:

```bash
curl -X POST http://127.0.0.1:8082/portal/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email":"portal@sdkwork.local",
    "password":"ChangeMe123!"
  }'
```

Password rotation:

```bash
curl -X POST http://127.0.0.1:8082/portal/auth/change-password \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <portal-jwt>" \
  -d '{
    "current_password":"ChangeMe123!",
    "new_password":"PortalPassword456!"
  }'
```

## Route Families

| Family | Routes | Purpose |
|---|---|---|
| health | `GET /portal/health` | liveness |
| auth | `POST /portal/auth/register`, `POST /portal/auth/login`, `GET /portal/auth/me`, `POST /portal/auth/change-password` | end-user registration, session lifecycle, and password rotation |
| workspace | `GET /portal/workspace` | inspect the caller-owned default workspace |
| dashboard | `GET /portal/dashboard` | workspace identity plus a combined usage and billing snapshot for the active project |
| usage | `GET /portal/usage/records`, `GET /portal/usage/summary` | recent requests, token-unit history, and aggregate request counts |
| billing | `GET /portal/billing/summary`, `GET /portal/billing/ledger`, `GET /portal/billing/events`, `GET /portal/billing/events/summary` | quota posture, used or remaining units, ledger visibility, and workspace-scoped Billing 2.0 event inspection |
| API keys | `GET /portal/api-keys`, `POST /portal/api-keys`, `POST /portal/api-keys/{hashed_key}/status`, `DELETE /portal/api-keys/{hashed_key}` | self-service gateway API key lifecycle inside the caller-owned workspace |
| API key groups | `GET /portal/api-key-groups`, `POST /portal/api-key-groups`, `PATCH /portal/api-key-groups/{group_id}`, `POST /portal/api-key-groups/{group_id}/status`, `DELETE /portal/api-key-groups/{group_id}` | self-service API key group lifecycle scoped to the authenticated workspace |
| routing | `GET /portal/routing/summary`, `GET /portal/routing/profiles`, `POST /portal/routing/profiles`, `GET /portal/routing/preferences`, `POST /portal/routing/preferences`, `GET /portal/routing/snapshots`, `POST /portal/routing/preview`, `GET /portal/routing/decision-logs` | workspace-scoped routing posture, reusable profile discovery and save-from-posture flows for group binding, compiled route evidence, preview, and decision evidence |

## Typical User Journey

1. register a portal account
2. log in and receive a portal JWT
3. inspect the default tenant and project workspace
4. open the dashboard snapshot for recent requests, token units, and quota posture
5. review usage and billing detail views
6. inspect workspace billing events and event summaries for group, capability, and accounting-mode visibility
7. review workspace routing posture and save the current posture as a reusable routing profile when a group should inherit a stable route plan
8. create one or more API key groups for stable policy attachment
9. issue a gateway API key, optionally binding it to a group with `api_key_group_id`
10. use that key against the gateway `/v1/*` surface

## Browser App

The portal browser experience is a dedicated app:

- `http://127.0.0.1:5174/`

## Related Docs

- product flow:
  - [Public Portal](/getting-started/public-portal)
- operator control plane:
  - [Admin API](/api-reference/admin-api)

## API Key Group Notes

- portal API key group routes are workspace-scoped and never expose groups from another tenant or project
- `slug` is optional on create or update; when omitted it is derived from `name`
- `default_routing_profile_id` is optional on create or update; when present it must resolve to an active routing profile inside the caller workspace
- `GET /portal/routing/profiles` returns only routing profiles inside the authenticated workspace tenant and project
- `POST /portal/routing/profiles` creates a routing profile inside the authenticated workspace tenant and project
- `GET /portal/routing/snapshots` returns only compiled routing snapshots inside the authenticated workspace tenant and project
- `POST /portal/routing/preview` and `GET /portal/routing/decision-logs` surface routing evidence fields when available, including:
  - `compiled_routing_snapshot_id`
  - `fallback_reason`
- portal routing profile creation does not accept caller-supplied `tenant_id`, `project_id`, or `profile_id`; the portal boundary derives those fields from the authenticated workspace and generates a profile id server-side
- routing profile discovery intentionally keeps inactive workspace profiles visible so existing group bindings remain editable and auditable
- compiled routing snapshot discovery is read-only in portal and is intended to explain the effective route state without exposing control-plane mutation
- `default_accounting_mode` is optional on create or update; when present it is normalized to one of:
  - `platform_credit`
  - `byok`
  - `passthrough`
- gateway billing inference for workspace-issued keys now reads `default_accounting_mode` from the bound API key group and otherwise falls back to `platform_credit`
- `POST /portal/api-keys` accepts optional `api_key_group_id`
- key creation rejects groups outside the caller workspace, groups with another environment, and inactive groups

## Billing Event Notes

- `GET /portal/billing/events` returns only billing events inside the authenticated workspace tenant and project
- `GET /portal/billing/events/summary` aggregates only workspace-visible billing events by:
  - project
  - API key group
  - capability
  - accounting mode
- event summaries expose multimodal dimensions already captured by the gateway event ledger, including:
  - token totals
  - `image_count`
  - `audio_seconds`
  - `video_seconds`
  - `music_seconds`
