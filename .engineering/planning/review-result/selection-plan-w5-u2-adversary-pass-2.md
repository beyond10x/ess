---
format: aep.planning-md/3
id: review-result:selection-plan-w5-u2-adversary-pass-2
kind: review-result
status: active
title: Adversary pass 2, wave 5 unit U2 (synthesis before-query)
relations:
- reviews: story:synthesis-reads-selection-plan
revision: 1
---
```
unit: story:synthesis-reads-selection-plan U2, pass-2 working tree on base 3bfa27aed1
verdict: nothing found (the adversary could not run its cases under the disk floor; the coordinator ran them)
cases: executed 245→254 (coordinator run), red 7
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: none new
needs-coordinator: run the pass-2 file
```

Cases in `adversary_selection_plan_w5_u2_pass2.rs`, run by the coordinator on the unit build:
- b1a, b1b (green): on ess/18 sign-in with `express` declared before `no-redirect-entry`, the interpreter answers `express` and the authored claim of `no-redirect-entry` is refused naming `express`; the second `not_taken` change agrees with the interpreter.
- b2a-c (red, not findings): the ess/21 two-row model does not validate ("a command guarding on more than one related row requires specification format ess/22"), so the format risk is not reachable this way; the `wrong_state:` variant goes to unit 4.
- b3 x4 (red, not U2 findings): whole-suite phase exchanges over 4 models fail on helpers units 3-5 own, and on models whose real-order suite already reports `Unsupported`; parked in scratch for units 3-5.

Also reported: with the pass-1 kind gate restored, the bytes table stays green, so only `a2_exchanged` sees that gate.

```findings
[]
```
