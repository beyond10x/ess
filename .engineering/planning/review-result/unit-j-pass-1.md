---
format: aep.planning-md/3
id: review-result:unit-j-pass-1
kind: review-result
status: active
title: Unit J adversary pass 1
relations:
- reviews: story:synthesis-at-ess26-does-not-stop-with-invalid-replay
revision: 1
---
## Adversary pass 1, unit J (ess-0700 wave)

Worktree ess-0700-j, base ee38a47b2; adversary added tests/adversary_j_pass1.rs (7 cases, 1 red). Cases executed 6 to 13.

Held: the read-back fix in either branch order at ess/25, ess/26 and ess/32; Go and TypeScript parity on the fixture; the fixture served by a network component owes and passes its grant scenarios; withdrawals are deterministic and counted once each in the coverage inventory; byte tables change only by header and new lines; view_grant.rs shapes admit.

```findings
- {file: crates/verify/ess-conformance/src/synthesize/grant.rs, line: 577, category: acceptance, severity: warning, verdict: NEEDS-CHANGE, origin: pre-existing, message: "for a served model with a declared replay the grant-denied template is the replay scenario, whose kept prefix leaves a result capture open, so the new pass withdraws the grant check"}
- {file: crates/verify/ess-conformance/src/synthesize/self_admission.rs, line: 1, category: diagnostics, severity: note, verdict: NEEDS-CHANGE, origin: introduced, message: "the withdrawal refusal reads as a garbled witness sentence and counts steps from 0 while other diagnostics name steps by path"}
```
