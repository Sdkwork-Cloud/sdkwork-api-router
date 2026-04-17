## Goal

Publish `images.aliyun` without inventing wrapper routes by reusing DashScope's official async image HTTP surface, while preserving the already-published `images.kling` family on the same public paths.

## Problem

DashScope-family image providers share the same official public paths:

- `POST /api/v1/services/aigc/image-generation/generation`
- `GET /api/v1/tasks/{task_id}`

That means the current `images.kling`-only handler shape does not scale. A second provider-specific family such as `images.aliyun` cannot be added safely by registering another distinct router path, because the public path is identical.

## Design

Treat the DashScope image transport as a shared official path family with provider-specific governance tags:

- public paths stay exactly official
- OpenAPI operation tags include both `images.kling` and `images.aliyun`
- runtime provider selection for create uses the request `model` value together with routing and a constrained set of mirror identities (`kling`, `aliyun`)
- runtime provider selection for task polling uses recorded task ownership (`reference_id` / `task_id`) to recover the exact provider that created the task

This keeps strict base-URL-switch compatibility while allowing multiple providers to coexist on the same official path surface.

## Provider Resolution

Two new runtime capabilities are required:

1. Planned execution constrained by multiple mirror identities, not just one.
2. Planned execution resolved by exact `provider_id` after task ownership lookup.

Task ownership lookup should reuse persisted billing events because they already store:

- `tenant_id`
- `project_id`
- `capability`
- `provider_id`
- `reference_id`
- `created_at_ms`

The lookup rule is: for `capability == "images"`, find the latest billing event matching the tenant, project, and `reference_id == task_id`, then route subsequent task polling to that provider.

## Usage Recording

DashScope image create responses are async and do not include final image counts. This slice should still persist lightweight zero-cost usage and billing evidence on successful create so ownership exists for later task polling.

Provider-specific route keys should remain explicit:

- `images.kling.generation`
- `images.aliyun.generation`
- `provider.kling.tasks.get`
- `provider.aliyun.tasks.get`

The shared handler chooses the route key from the selected provider's mirror identity.

## Stateless and Stateful Behavior

- stateless mode relays the shared DashScope image paths only when the configured upstream mirror identity is `kling` or `aliyun`
- stateful mode selects among `kling` and `aliyun` for create, and resolves exact provider ownership for task polling
- no wrapper paths such as `/images/kling/*` or `/images/aliyun/*` are introduced

## Documentation Impact

English and Chinese gateway docs and compatibility docs should stop describing `images.aliyun` as reserved-only. They should describe DashScope async image paths as an active shared official transport published under both `images.kling` and `images.aliyun`.
