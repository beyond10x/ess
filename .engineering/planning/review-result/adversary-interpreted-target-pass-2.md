---
format: aep.planning-md/1
id: review-result:adversary-interpreted-target-pass-2
kind: review-result
status: active
title: 'Adversary pass 2: the unqualified claim moved from the module to the published guide'
summary: 'Red: 1 introduced warning, no blocker; corrected and verified by the coordinator'
relations:
- reviews: story:interpreted-target-selection
revision: 1
---
# Adversary pass 2 — story:interpreted-target-selection

Verdict **red**, one warning, no blocker. Cases executed 937 → 938, 1 red. Origin: introduced 1,
pre-existing 1, undecided 0. Corrected and verified by the coordinator; see the outcome record.

## Confirmed by re-measurement

The support block regeneration is **byte-faithful**, not a hand-adjusted line: the generator emits 29
lines, the marked region is 29 lines, `cmp` exit 0 and `md5sum` identical on both
(`2aeaac8fdb6bf0bbba9dbed749b6b993`).

No non-empty suite can be green, established by argument rather than sampling: `begin_scenario`
unconditionally refuses, `runner.rs:394-410` records exactly one check and runs neither the steps nor
`end_scenario`, `Status::Passed.worst(Unsupported)` is `Unsupported`, and `report.rs:566-570` returns
`Failed` if any scenario is `Failed | Unsupported`. All three committed suites measured: billing
30/30, oracle-fixture 31/31, gatepass 12/12 unsupported, each exit 1, each scenario carrying exactly
one `ESS-CF-TARGET` check. The oracle suite exercises periodic, halt, elapsed and redelivery step
kinds; none reached a method. No path produced exit 3. Two consecutive runs byte-identical.

The coordinator's acceptance of the re-aimed empty-suite case was judged correct and the case
stronger than as delivered: the cross-target control asserts `billing` and `interpreted` give *equal*
answers before pinning the value, which is what decides whether the exception is a runner property or
a target capability. One note, no case: `interpret.rs:32` names `oracle-fixture` too and the control
pins only `billing`; all three were measured equal.

```findings
- file: website/docs/guides/verify-conformance.md
  line: 149
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "the correction that added `interpreted` to the adopter guide also wrote the unqualified claim that the run fails, which is the sentence round 1 removed from interpret.rs: for the zero-scenario selection the same guide documents at line 238 the run reports passed and exits 0."
- file: docs/design/review-public-support-claims.md
  line: 66
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: pre-existing
  message: "leaving the pinned wave-14 record stale is correct: its header pins it to commit ecb7efc2, its citations at main.rs:576-580 and 2518-2522 already resolved to unrelated code at the unit's base commit, and docs/design is not served by the Docusaurus site, so no published support claim depends on it."
```
