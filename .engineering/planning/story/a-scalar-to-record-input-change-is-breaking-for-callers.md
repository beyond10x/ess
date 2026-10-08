---
format: aep.planning-md/3
id: story:a-scalar-to-record-input-change-is-breaking-for-callers
kind: story
status: active
title: ess verify diff rates a scalar-to-record command input change as breaking for callers
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- informed_by: story:an-older-command-input-is-read-on-replay-through-a-declared-upcast
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-08T04:42:05Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-08T04:42:05Z", actor: "human:timo", revision: 4}
---
## Need

Changing a command input field from a scalar to a record (`receipt: String` → `receipt: Receipt`)
breaks every caller that still sends the old shape: the generated decoder answers 400. `ess verify
diff` on ess 0.56.0 rates the change `unknown` for callers and `compatible` for history
(`crates/verify/ess-diff/src/compatibility.rs:379`, cited by the fit review of
`story:an-older-command-input-is-read-on-replay-through-a-declared-upcast`), so
`--fail-on breaking` lets it through. The reproducer is `.engineering/repro/upcast/`
(`before.yaml`, `after.yaml`).

## Outcome

`ess verify diff` rates a command input field's change between a scalar and a record, list or map
(either direction) as breaking for callers, naming the field, and `--fail-on breaking` fails on it.
Whether this needs a new `ess-diff/N` is decided in the fit review (inferred: it does, if the
verdict is persisted).

## Next

Fit review with `.agents/skills/assessing-external-requests/SKILL.md`; no `synthesize/` change is
expected.

## Fit review

Assessed with the older-command-input fit review (`story:an-older-command-input-is-read-on-replay-through-a-declared-upcast`, same reproducer). **Class: defect.** A change every existing caller fails on (the generated decoder answers 400) is rated `unknown` for callers: the catch-all `CommandChange` arm at `crates/verify/ess-diff/src/compatibility.rs:378` gives `[U, U, C]`. **Already expressible?** No: `--fail-on breaking` is the documented gate and it passes the change. **Fit:** no authored key, no new noun; one verdict refined inside the existing diff. **Second adopter:** any service that turns an `amount: Decimal` input into `amount: {value, currency}`.

## Decisions

**accept.** Rate a command input field's change between a scalar and a record, list or map as breaking for callers, naming the field. The format consequence follows `AGENTS.md`: if a written delta's verdict for an existing change kind changes meaning, the delta gets the next `ess-diff/N` and older readers refuse it by name; if verdicts are not persisted, no format moves. The implementor establishes which, cites the code, and the unit report says so.

## Acceptance

- `ess verify diff .engineering/repro/upcast/before.yaml .engineering/repro/upcast/after.yaml` reports the input field's change as breaking for callers, and `--fail-on breaking` exits non-zero. Named test in `crates/verify/ess-diff/tests/`.
- The reverse direction (record → scalar), and scalar → list and scalar → map, are breaking for callers too.
- A scalar → scalar change the diff already classifies keeps its verdict, and `after-additive.yaml` (the Optional-input idiom) stays compatible for callers.
- Every existing `ess-diff` and `ess-cli` diff test passes unchanged, or the report names each re-pinned expectation and why.
