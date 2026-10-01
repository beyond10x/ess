---
format: aep.planning-md/3
id: review-result:adversary-gaps-265-pass-2
kind: review-result
status: active
title: Adversary pass 2, downstream gaps unit feature-request-265
relations:
- reviews: story:feature-request-265
revision: 1
---
unit: story:feature-request-265, tree gaps-265
verdict: NEEDS-CHANGE, red 7 (introduced 7)
cases: executed 2543→2550

- blocker (synthesize/grant.rs:295): denied scenarios borrowed a wrong-state template (gatepass AdmitVisitor, SignOutVisitor), so a run-then-refuse surface passed.
- warning (runner.rs, go/runtime.go, ts/runtime.ts): a repeated event matching an earlier one was excused.
- warning (synthesize/grant.rs:53): one attributed actor disabled admitted-actor rotation for the whole model, with no note.
- warning (go/runtime.go, ts/runtime.ts): events handed back with a refusal were remembered as seen.
- blocker (ts/runtime.ts decodeStep): unpublished was dropped, so the TS runner never checked the log.
- note (authored.rs): an act with no actor could send a served command granted to no actor.

Correction 2 (coordinator-verified): accepting templates only, else GrantDeniedUnwitnessed; occurrence counting in all three runners; per-command rotation with GrantRotationSkipped; a refusal handing back events fails; TS decodes unpublished; ESS-AUTHOR-040 refuses the authored act.
Coordinator rerun in gaps-265: adversary_265 pass1 4/4, pass2 7/7, grant_denied 7/7. Unit gates: ess-gen/ess-synth/ess-conformance 2550 passed, ess-cli 867, gatepass 11, billing 15, projection-check and generate --check current.
