---
format: aep.planning-md/3
id: review-result:consumer-initial-state-pass2
kind: review-result
status: active
title: Scenario initial state bounded source review pass 2
relations:
- reviews: story:feature-request-312
revision: 1
---
approve

```json
[]
```

Approval is limited to the frozen source unit, not full completion of issue312. Reviewed patch6a873ad73df30936056136f41c05ebd4ac0c9723b0c0e26c59dd4a6d53a146e4 in ess-backlog-initial-state-20261002. Reviewer executions:0. Production/test changes from pass1 and the full initial-state reader/display boundary were reviewed; no edits or builds were performed by this reviewer.

The silent same-command gap identified in pass1 is corrected for attribute-free and attributed caller-insensitive commands: the first invocation arranges under one declared principal and later invocations act under the other, preserving the same row identity. New tests exercise actual InstallSwitch/already-installed traces and a partitioned-store mutant for both actor forms, and inspect the actual create/update upsert timeline. Reversal uses the existing independent-append machinery; inability to append is explicitly distinguished from inability to form a mixed witness. Singleton cleanup drops only notes for scenarios actually withdrawn.

Caller-sensitive same-command arrangements remain incomplete. They are retained without mechanically changing their expected authorization or payload, and now emit CrossCallerUnwitnessed with the exact per-invocation planning limitation. This limitation is source-visible, separately tested, and root has explicitly retained issue312 active. This approval must not be cited as satisfying every caller-sensitive direct/related family or the entire story acceptance. Finishing that remaining case requires mixed-invocation synthesis with source-authorized expectations, not merely actor-byte substitution.

Typed empty logical state is mandatory on new/34–35 provenance and forbidden on legacy/1–33; selected coverage keeps the parent bytes and requirement. Diagnostics describe the namespace precondition without alleging a backend reset. Browser replay keeps its supported-step boundary while admitting appropriate/35 references. No source or CountReport/2 schema expansion is introduced.

Producer reports final focused34/0/0, scoped strict Clippy/fmt/diff checks green; the corrected parity test now compares diagnostic code sets symmetrically for native/Go/TS/WASM alongside counts/outcomes. Final browser/CLI rerun was pending at review handoff and must be confirmed by the producer/root before integration. Generated fixture/version expectation migration remains root-owned follow-up. No broad package, remote gate or release-readiness claim is made here.
