---
format: aep.planning-md/3
id: story:an-authored-step-expecting-a-shadowed-related-refusal-is-refused
kind: story
status: active
title: An authored step expecting a related refusal that an earlier one over the same row shadows is refused
summary: conform author accepts a step the interpreter then fails, because the first declared refusal over one related row answers (row 27, 2026-10-23).
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-23T04:52:43Z", actor: "human:timo", revision: 2, correlation: "ess-0690"}
- {from: "proposed", to: "active", at: "2026-10-23T04:52:43Z", actor: "human:timo", revision: 3, correlation: "ess-0690"}
---
## Need

Reported by an adopter (harness row 27, triaged on 0.69.0). A command declares two refusals over the
same related row, both `when_related: {via: input.locker_id, predicate: accepting == false}`. An
authored scenario expects the second. `ess verify conform author` accepts it (exit 0, 1 scenario,
0 refusals) and `ess verify conform run --target interpreted` then fails it, because the first
declared refusal answers (`docs/design/cross-record-and-stored-field-guards.md:828-830`, `:868`:
"two refusals over one related row, the first declared"). The interpreter is right; author is wrong.

## Where it lives

- `crates/verify/ess-conformance/src/authored.rs:1740` (`not_taken`): decides input-guarded refusals
  and accepting `when:` branches only; its doc says a related row "is not decided".
- `authored.rs:3172` (call site, `decided` is input-only).
- `crates/verify/ess-conformance/src/synthesize.rs:7590` (`sibling_refusals`, input-guarded only),
  `synthesize/precedence.rs:32`.

## Acceptance

- `ess verify conform author` refuses a step that expects a `when_related:` refusal when a refusal
  the precedence order answers earlier reads the same related row with a guard the expected one
  implies (identical included). The refusal names the earlier branch and cites the precedence rule.
- On the reproduction (brand-free copy), author refuses the step and the interpreted run has nothing
  left to fail.
- An expected related refusal whose guard does not imply the earlier one's is still admitted.

## Format

None: state-free check at authoring, no format version, no ADR.

## Side finding

`ess verify conform synthesize` refuses both refusals of the reproduction with ESS-SYNTH-003; only
the second follows from the shadowing. Check while here; file separately if it is a different gap.
