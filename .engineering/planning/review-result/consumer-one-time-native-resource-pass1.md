---
format: aep.planning-md/3
id: review-result:consumer-one-time-native-resource-pass1
kind: review-result
status: active
title: Native disclosure resource accounting review
relations:
- reviews: story:feature-request-389
revision: 1
---
needs-revision

Early independent coordinator source review of the native observer while implementation remains in progress. Own tests: 0. The observation budget in runner/disclosure.rs adds raw text/key lengths and node increments; it does not enforce the accepted serialized JSON byte limit. This review is confined to that concrete issue, not an approval or complete audit of the pending native implementation.

Escaped control characters can serialize to more than six times their raw UTF-8 size, and numeric leaves contribute only a node increment. Consequently payloads over 1 MiB may be accepted. The native owner acknowledged the finding and owns shared exact-boundary controls and the correction; both ports have been told to consume the same finalized semantics. The agreed precise definition is recorded in story:feature-request-389 under Exact private observation resource limits.

```findings
[{"file":"crates/verify/ess-conformance/src/runner/disclosure.rs","line":419,"category":"correctness","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"Payload budget counts raw key/text lengths and node increments instead of complete compact JSON bytes, excluding escape expansion, numeric spelling and delimiters. Enforce the accepted inclusive 1 MiB bound with depth/member validation followed by a bounded counting serializer; share exact boundary controls with both runtime ports."}]
```
