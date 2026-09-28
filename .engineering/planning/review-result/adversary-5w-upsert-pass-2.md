---
format: aep.planning-md/3
id: review-result:adversary-5w-upsert-pass-2
kind: review-result
status: active
title: Adversary pass 2, upsert (the-5-waves)
relations:
- reviews: story:upsert-outcome-by-existence
revision: 1
---
Adversary pass 2 against story:upsert-outcome-by-existence (#164), aep:adversary, 2026-09-28, the-5-waves wave 4, after correction round 1.

verdict: NEEDS-CHANGE
cases: added 11, red 6 (executed 1845→1856)
origin: introduced 6, pre-existing 0, undecided 0

New cases in `adversary_upsert_pass2_{witness,domain}.rs`. Red: no scenario sends the refused input for a stored identity; identities shared with an unknown_instance refusal and with the literal-fallback invocation; only the first creating branch is witnessed; the #164 follow-up optional id is refused. Pass-1 corrections hold.

Ledger against pass 1: carried 0, new 6, resolved 4. Coordinator decision: change the witness (not the docs) for finding 1. Final correction round; the coordinator verifies the diff.

```findings
[{"file":"docs/design/outcome-shapes.md","line":0,"category":"mutant","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"No scenario sends an input-guarded refusal input for an identity a record already carries, so existence-first targets pass."},
 {"file":"crates/verify/ess-conformance/src/synthesize.rs","line":6768,"category":"concurrency","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"The upsert creation uses the unknown witness another unknown_instance refusal on the entity sends."},
 {"file":"crates/verify/ess-conformance/src/synthesize.rs","line":0,"category":"concurrency","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"The literal-fallback invocation of a creating unknown_instance branch bypasses fresh_created."},
 {"file":"crates/verify/ess-conformance/src/synthesize/existence.rs","line":309,"category":"mutant","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"Only the first creating branch of a create-or-refuse gets the two-call witness."},
 {"file":"crates/specify/ess-domain/src/command/outcome_shapes.rs","line":382,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"The #164 follow-up optional caller id is refused as unreachable."},
 {"file":"crates/specify/ess-compiler/src/ir.rs","line":653,"category":"contract-drift","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"The ExistingInstance doc comment contradicts the input-first precedence."}]
```
