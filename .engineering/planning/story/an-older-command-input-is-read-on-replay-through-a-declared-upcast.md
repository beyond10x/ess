---
format: aep.planning-md/3
id: story:an-older-command-input-is-read-on-replay-through-a-declared-upcast
kind: story
status: draft
title: Replay reads a recorded command input in an older shape through a declared upcast
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Need

An adopter changes a command input field's type: a text field becomes a record
(`receipt: String` → `receipt: {revision, evidence}`). Recorded invocations made with the old
shape must still replay. On ess 0.56.0 replay of such history fails with
`body.receipt: expected an object, found a string`, and the specification has no declared way to
say how an older input is read: neither an input version nor an upcast from the old shape to the
new one. The adopter's own conversion design validates but does not generate (2 unmet
capabilities); without the conversion it generates (132 capabilities, synthesis 181 scenarios,
0 refusals) and replay still fails. The adopter keeps a hand-written host guard until ESS can say
this.

## Reproduction

Not yet minimised. A reproducer with neutral nouns goes into `.engineering/repro/` with the fit
review: one command whose input field changes from String to a record between two specification
revisions, and one recorded invocation in the old shape.

## Next

Assess with `.agents/skills/assessing-external-requests/SKILL.md` before any build. Questions for
the fit review: whether an older input is a new authored construct (and so a source format
`ess/N`), whether `ess verify diff` already classifies the change and can name the reading
obligation, and which surfaces it touches (replay in `ess-conformance`, generated readers in
`ess-gen`/`ess-synth`). Where it lands is not established; whether it needs `synthesize/` decides
whether it may go before the other session's synthesis waves end.
