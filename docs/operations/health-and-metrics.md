# Health and Metrics

The services expose both health and metrics endpoints.

## Health Endpoints

- gateway: `http://127.0.0.1:8080/health`
- admin: `http://127.0.0.1:8081/admin/health`
- portal: `http://127.0.0.1:8082/portal/health`

## Metrics Endpoints

- gateway: `http://127.0.0.1:8080/metrics`
- admin: `http://127.0.0.1:8081/metrics`
- portal: `http://127.0.0.1:8082/metrics`

## Example Checks

```bash
curl http://127.0.0.1:8080/health
curl http://127.0.0.1:8081/admin/health
curl http://127.0.0.1:8082/portal/health
```

```bash
curl http://127.0.0.1:8080/metrics
curl http://127.0.0.1:8081/metrics
curl http://127.0.0.1:8082/metrics
```

## Operational Expectations

Use health endpoints for:

- liveness checks
- startup validation
- smoke tests

Use metrics endpoints for:

- Prometheus scraping
- request-rate monitoring
- latency tracking
- service-level troubleshooting

## Runtime Rollout Observability

Multi-node runtime rollout is now inspected through the admin API rather than a dedicated metrics series.

Endpoints:

- create rollout: `POST /admin/extensions/runtime-rollouts`
- list rollouts: `GET /admin/extensions/runtime-rollouts`
- inspect rollout: `GET /admin/extensions/runtime-rollouts/{rollout_id}`
- create standalone config rollout: `POST /admin/runtime-config/rollouts`
- list standalone config rollouts: `GET /admin/runtime-config/rollouts`
- inspect standalone config rollout: `GET /admin/runtime-config/rollouts/{rollout_id}`

Operational notes:

- active gateway and admin nodes heartbeat into the shared admin store for extension-runtime rollout
- active gateway, admin, and portal nodes heartbeat into the shared admin store for standalone config rollout
- rollout status is derived from per-node participant records and rollout deadlines
- aggregate statuses are `pending`, `applying`, `succeeded`, `failed`, or `timed_out`
- participant rows expose `node_id`, `service_kind`, `status`, optional failure `message`, and `updated_at_ms`
