---
format: aep.planning-md/3
id: review-result:adversary-5w-seteff-pass-1
kind: review-result
status: active
title: Adversary pass 1, seteff (the-5-waves)
relations:
- reviews: story:set-effects-over-filtered-instances
revision: 1
---
Adversary pass 1 against story:set-effects-over-filtered-instances (#167, #175), aep:adversary, 2026-09-28, the-5-waves wave 4 (dispatched late).

verdict: NEEDS-CHANGE
cases: added 14, red 11 (executed 2007→2021)
origin: introduced 4, pre-existing 0, undecided 1

New cases in `crates/specify/ess-domain/tests/adversary_seteff_pass1_domain.rs` and `crates/verify/ess-conformance/tests/adversary_seteff_pass1_conformance.rs`. Red: a scenario filed and passing when no view shows the state or the `sets:` fields; `sets:` writing the identity admitted; a target dropping one conjunct of a multi-conjunct `where` passing; a target refusing zero matches or reporting a constant count passing; malformed set-effect keys under ess/15 getting shape errors instead of `unsupported_format_version`. Not broken: leaked, skipped and out-of-`from` rows, a wrong count, the subject included in `affects:`, partial `sets:`, ess-diff.

Coordinator routing: all five to the implementor's final correction (observation required, identity refused, one non-matching row per conjunct, three matching rows and a zero-match call, format refusal first); all 11 red cases green afterwards.

```findings
[{"file":"crates/verify/ess-conformance/src/synthesize/set_effects.rs","line":112,"category":"acceptance","severity":"blocker","verdict":"CONFIRMED","origin":"introduced","message":"a set-effect scenario is filed and passes against a target changing non-matching rows when the only observing view publishes neither the state nor the sets fields"},
 {"file":"crates/specify/ess-domain/src/command/set_effects.rs","line":447,"category":"boundary","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"sets on instances or affects may write the entity identity, giving every selected row one id"},
 {"file":"crates/verify/ess-conformance/src/synthesize/set_effects.rs","line":350,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"undecided","message":"one non-matching row falsifies one conjunct, so a target dropping another conjunct of where, including a subject field, passes"},
 {"file":"crates/verify/ess-conformance/src/synthesize/set_effects.rs","line":41,"category":"mutant","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"no zero-match scenario and a single count value let a target refusing zero matches or reporting a constant count pass"},
 {"file":"crates/specify/ess-domain/src/command.rs","line":4644,"category":"contract-drift","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"under ess/15 malformed instances or affects are refused at parse time without unsupported_format_version at that key"}]
```
