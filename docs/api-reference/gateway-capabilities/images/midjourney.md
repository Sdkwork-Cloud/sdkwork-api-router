# images/midjourney

## Publication Status

- Status: unpublished capability note
- Live mirror family: none
- Public path family: none

## Why It Is Not Published

- SDKWork's mirror rule requires an official provider API surface that works by switching only `base_url`.
- Midjourney does not currently expose a public official API contract that fits that rule.
- Because of that, the gateway does not publish a separate `images.midjourney` mirror family today.

## What This Means

- the capability is tracked as a documentation gap, not hidden
- no wrapper route is invented just to satisfy directory symmetry
- if Midjourney later publishes an official mirrorable API, this page can become an active family page
