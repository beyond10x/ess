---
format: aep.planning-md/3
id: story:a-related-selector-compares-a-row-field-with-a-subject-field
kind: story
status: draft
title: A related selector that compares a row field with a subject field gets a witness
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 2
---
## Need

An adopter's specification guards a command with a `when_related` selector whose `where` compares a row field with a field of the command's subject: `workspace_id == subject.workspace_id`, or `group_key == subject.group_key` beside `id != subject.id` ("another row in the same group is running"). ess 0.56.0 synthesizes no witness for the outcomes the selector separates and prints `no witness: … reads a subject field the steps leave undetermined`. In the adopter's specification this leaves 39 of 180 scenarios without a witness for one selector and 20 for another.

## Reproduction

Not yet minimised. A reproducer with neutral nouns (`catalog`, `items`, `group_key`) goes into `.engineering/repro/` with the fit review.

## Next

Assess with `.agents/skills/assessing-external-requests/SKILL.md` before any build. The fix is expected in synthesis (`crates/verify/ess-conformance/src/synthesize/`), so it is scheduled after the other session's synthesis waves leave those files.

## Downstream report 2026-10-19

Neutral reproduction from a downstream report (2026-10-19, finding 10): `.engineering/repro/related-from-subject-field/row-set-subject.yaml` — `Rename` refuses a name another port of the subject's node holds, `when_related: {entity: Port, where: {all: [node_id == subject.node_id, port_id != subject.port_id, name == input.name]}, exists: true}`. Integration build: `ESS-SYNTH-001` on both `name-taken` and `renamed` ("reads a subject field the steps leave undetermined"). The downstream specification lost 16 scenarios to one such selector.
