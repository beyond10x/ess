---
format: aep.planning-md/3
id: review-result:consumer-ui-read-cache-pass1-20261002
kind: review-result
status: active
title: Review of UI row-filter shared request isolation and freshness
relations:
- reviews: story:feature-request-365
revision: 1
---
## Outcome

Two introduced findings confirmed against the UI365 working candidate over55061600. Coordinator own executions: 0. Implementor reproduced both counterexamples in25-cache-red.log: six passed, two failed. The staggered-refetch case returned revision1 instead of2; the authority-change case returned the previous actor's cached rows. No resolution or final gate is claimed in this immutable review; later fix evidence must answer it separately.

## Evidence

Source inspection of templates/runtime/data.ts.tmpl: setAuthorization only assigned the global header, while sharedReads retained fulfilled promises and useRead keyed them by seen plus a hook-local nonce. Equal local counters were incorrectly treated as one refresh generation, and authorization was absent from request identity. The implementor is correcting the cache and retaining concurrent-read sharing, last-subscriber abort behavior, and adapter-replacement controls.

```findings
- file: crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl
  line: 528
  category: correctness
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: Two mounted readers share nonce0. ReaderA refreshes to nonce1 and retains its fulfilled promise. ReaderB refreshes later to its own nonce1 and reuses that old result without a fresh adapter request; changed backend data is missed.
- file: crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl
  line: 272
  category: authorization
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: After setAuthorization changes caller identity, a newly mounted reader of the same view and params reuses a fulfilled promise held by an old-authority reader, exposing the previous actor's rows instead of making a request under the new authority.
```
