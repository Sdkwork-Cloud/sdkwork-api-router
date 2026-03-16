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
| billing | `GET /portal/billing/summary`, `GET /portal/billing/ledger` | quota posture, used or remaining units, and ledger visibility |
| API keys | `GET /portal/api-keys`, `POST /portal/api-keys` | self-service gateway API key listing and creation |

## Typical User Journey

1. register a portal account
2. log in and receive a portal JWT
3. inspect the default tenant and project workspace
4. open the dashboard snapshot for recent requests, token units, and quota posture
5. review usage and billing detail views
6. issue a gateway API key
7. use that key against the gateway `/v1/*` surface

## Browser App

The portal browser experience is a dedicated app:

- `http://127.0.0.1:5174/`

## Related Docs

- product flow:
  - [Public Portal](/getting-started/public-portal)
- operator control plane:
  - [Admin API](/api-reference/admin-api)
