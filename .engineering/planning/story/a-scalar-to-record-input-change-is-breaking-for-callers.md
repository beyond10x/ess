---
format: aep.planning-md/3
id: story:a-scalar-to-record-input-change-is-breaking-for-callers
kind: story
status: draft
title: ess verify diff rates a scalar-to-record command input change as breaking for callers
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- informed_by: story:an-older-command-input-is-read-on-replay-through-a-declared-upcast
revision: 1
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
