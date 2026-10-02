---
format: aep.planning-md/3
id: review-result:consumer-interpreted-caller
kind: review-result
status: active
title: Invocation-local caller independent review
relations:
- reviews: task:consumer-backlog-20261002
revision: 1
---
approve

```json
[]
```

Independent source review of frozen caller patch 1fb9a86f15c62f507e902d01a2d8830edc0e2e5f9572dec283e0d13307b8ea17 against base ee6962765749d25e019fa819c7b0019799aad464; seven staged files inspected. Reviewer executions: 0. Reviewed producer's retained reports, production diff, changed tests, common command completion/grant implementation, caller-value source contract and predicate validation.

Grant refusal precedes attribute binding; binding validates the actual named actor with Total completeness before borrowing scenario state or consuming an external control. Nonempty attributes without an actor are invalid rather than being assigned an arbitrary role. Invocation holds immutable per-call references, with no caller fields added to Scenario, Store, generated slots or bindings. Public sequential entry points carry no authentication context.

Input guards overlay caller facts only when an ordinary input root named caller does not shadow them. Stored/related predicates apply the same rule against the stored field declarations, while their additional input predicates use input declarations. Typed facts retain primitive representation and ordering metadata. Event/stored/error value copies clone the validated attribute, preserve optional omission, and recurse through struct leaves. Request validation and generated control preservation have direct tests; actual synthesized caller suites, role swapping and Boolean attributes execute the interpreter.

I considered whether presence-only predicates could turn missing authentication into a factual absence. The admitted caller construct permits only equality/inequality against declared fields; caller presence predicates are rejected by source validation. This speculative path is therefore not a finding for the admitted source surface.

Limitations: no additional executions or exhaustive capability coverage by this reviewer. The producer reports targeted neighbor tests and strict scoped checks; full integration with typed keys and later existence work remains the root's responsibility. Existing unsupported retained replay and unrelated value sources are outside this patch.
