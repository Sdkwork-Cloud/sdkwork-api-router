# API Compatibility

SDKWork tracks compatibility with five execution-truth labels:

- `native`
- `relay`
- `translated`
- `emulated`
- `unsupported`

## High-Value API Families

Currently implemented gateway families include:

- `/v1/models`
- `/v1/chat/completions`
- `/v1/completions`
- `/v1/responses`
- `/v1/embeddings`
- `/v1/files`
- `/v1/uploads`
- `/v1/audio/*`
- `/v1/images/*`
- `/v1/assistants`
- `/v1/threads`
- `/v1/conversations`
- `/v1/vector_stores`
- `/v1/batches`
- `/v1/fine_tuning/jobs`
- `/v1/webhooks`
- `/v1/evals`
- `/v1/videos`

The control plane also exposes:

- `/admin/*`
- `/portal/*`

## How To Read Compatibility

- use the API reference pages to understand ownership, base paths, and auth
- use this compatibility view to understand execution semantics
- use the full matrix when you need route-family-level truth across stateful and stateless modes

Primary entry points:

- [API Reference Overview](/api-reference/overview)
- [Gateway API Reference](/api-reference/gateway-api)
- [Admin API Reference](/api-reference/admin-api)
- [Portal API Reference](/api-reference/portal-api)

## Detailed References

Read the full data-plane and control-plane matrix here:

- [Full Compatibility Matrix](/api/compatibility-matrix)
