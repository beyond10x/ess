---
format: aep.planning-md/3
id: story:a-related-selector-compares-a-row-field-with-a-subject-field
kind: story
status: draft
title: A related selector that compares a row field with a subject field gets a witness
tags:
- defect
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/row_set.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/row_sets.rs
revision: 6
---
## Need

Specifications guard a command with a `when_related` row-set selector whose `where` compares a row
field with a field of the command's subject. For example: "another port of the same node holds
this name", `{node_id == subject.node_id, port_id != subject.port_id, name == input.name}`, or
"another row in the same group is running". Synthesis writes no witness for the outcomes such a
selector separates. Two downstream specifications lost 39+20 and 16 scenarios to such selectors
(reports against 0.56.0 and 2026-10-19, finding 10).

## Reproduction

`.engineering/repro/related-from-subject-field/row-set-subject.yaml`, still present on 0.92.0:
`Node {node_id, label}` with lifecycle `Open → Sealed`, and `Port {port_id, node_id, name}`
referencing its node, created by `AddPort(node_id, name)`. `Rename(port_id, name)` refuses
`name-taken`
(`when_related: {entity: Port, where: {all: [node_id == subject.node_id, port_id != subject.port_id, name == input.name]}, exists: true}`)
and otherwise `renamed` (`updates: Port`, `sets: {name: input.name}`).

```text
$ ess verify conform synthesize --path .
refused: refusal[ESS-SYNTH-001]: outcome wiring.board.Rename/name-taken has no scenario …
  no witness: `…(node_id == subject.node_id and port_id != subject.port_id and name == input.name)`
  reads a subject field the steps leave undetermined` is `row set`, which has no arrangement …
  help: let a command that creates the entity write each field the filter reads from an input …
refused: refusal[ESS-SYNTH-001]: outcome wiring.board.Rename/renamed has no scenario … (same)
6 scenario(s) (0 authored), 2 refusal(s)
```

The help is misleading: `AddPort` already writes `node_id` and `name` from inputs.

Probes on 0.92.0:

- `where: {all: [port_id != subject.port_id, name == input.name]}` gives the same two refusals,
  so the generated identity `subject.port_id` alone triggers "undetermined".
- `where: {all: [node_id == subject.node_id, name == input.name]}` synthesizes `name-taken` but
  refuses `renamed`: "`name-taken` is not decidedly passed over: its row set is True". The subject
  itself is a member of its own row set unless `input.name` differs from its stored name.

Where it lives: `crates/verify/ess-conformance/src/synthesize/row_set.rs`, `closed` (:897-921).
Subject values are written in from `self.subject` (`subject_value`). A `subject.<field>` the
arrangement does not hold as a literal (a target-minted identity such as `port_id`) is left in
the predicate and refused with the message above (:918).

## Acceptance

- `a_related_selector_comparing_a_row_field_with_a_subject_field_is_witnessed`: the reproduction
  synthesizes `Rename/name-taken` and `Rename/renamed` with 0 refusals, and every scenario passes
  `--target interpreted`. Before the fix both are refused as ESS-SYNTH-001 as above.
- The `node_id == subject.node_id, name == input.name` variant synthesizes `renamed` by sending a
  name no port of the node holds, the subject included.
- Where a `subject.<field>` truly cannot be determined, the refusal's help no longer tells the
  author to write the filter's fields from inputs when the creator already does.
- Models that synthesize today keep their suite bytes.

## Scope

- cited: `crates/verify/ess-conformance/src/synthesize/row_set.rs` (`closed` :897-921, `subject_value`)
- inferred: `crates/verify/ess-conformance/src/synthesize/row_set.rs` (row arrangement that holds the subject's identity as a captured instance)
- inferred: `crates/verify/ess-conformance/tests/row_sets.rs` (new cases)

confidence: medium
