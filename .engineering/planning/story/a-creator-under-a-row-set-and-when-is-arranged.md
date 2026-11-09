---
format: aep.planning-md/3
id: story:a-creator-under-a-row-set-and-when-is-arranged
kind: story
status: draft
title: A creator guarded by a row set and a when is arranged
tags:
- defect
- ess-d368-close
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/row_set.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/a_creator_under_a_row_set_is_arranged.rs
- confidence: inferred
  path: examples/
revision: 5
---
## Finding

Wave D368 U4 found this in a first-round implementor note, outside that story's Acceptance. A
creator branch guarded by a row set is never used to arrange a row, so nothing that needs a row it
lays is reached through it (`crates/verify/ess-conformance/src/synthesize.rs:13903`,
`arrange_toward_with`). Only the per-input pass, which runs only beside captured rows, sends a
creator guarded by a row set alone (`by_rows`, `synthesize.rs` in `arrange_toward_with`). The plain
pass asks `selects_branch` without rows and never selects it, and a row set beside a `when:`
(`RelatedSet { input: Some(_) }`) is never sent in any pass.

## Problem

Re-checked on 0.92.0. A uniqueness-checked creator, the common shape:

```yaml
commands:
  - name: wiring.board.Connect
    input:
      - {name: from_port, type: String}
      - {name: priority, type: Integer}
    outcomes:
      - name: source-taken
        when_related: {entity: wiring.board.Wire, where: from_port == input.from_port, exists: true}
        error: wiring.board.SourceTaken
      - name: connected
        when_related: {entity: wiring.board.Wire, where: from_port == input.from_port, exists: false}
        when: priority >= 1
        creates: wiring.board.Wire
        instance: wire_id
        sets: {from_port: input.from_port, priority: input.priority}
        emits: [wiring.board.Connected]
        payload: {wiring.board.Connected: {wire_id: {generated: true}}}
      - name: too-weak
        error: wiring.board.TooWeak
  - name: wiring.board.Cut          # moves Wire Live -> Cut; wrong_state: not-live
```

`ess verify conform synthesize` writes `Connect/outcome/connected` (the creator itself is witnessed)
and then refuses `Connect/source-taken` (ESS-SYNTH-001 "its own row set is False"), and refuses
`Cut/outcome/cut`, `Wire/transition/cut/...` and `Wire/state/Cut/refuses/...` with ESS-SYNTH-004:
"the route runs through `wiring.board.Connect/connected`, which no input reaches". That claim is
false, because the same suite holds a scenario that reaches `connected`. Without the `when:` (row
set alone) the same five refusals stand. Without the row set (`when:` alone) synthesis writes all 7
scenarios and refuses none.

## Acceptance

- `crates/verify/ess-conformance/tests/a_creator_under_a_row_set_is_arranged.rs`,
  `a_creator_under_a_row_set_and_when_is_arranged`: for the model above, synthesis records no
  refusal; `Cut/outcome/cut` and `Connect/outcome/source-taken` each create their `Wire` through a
  `Connect` step whose `priority` holds the `when:`; the interpreted target passes every scenario.
  Red on 0.92.0.
- Same file, `a_creator_under_a_row_set_alone_is_arranged`: the same model without the `when:`
  gives the same result.
- Where no input can hold the creator's `when:` (a contradictory guard), the refusal is
  ESS-SYNTH-003 naming the creator's guard, not ESS-SYNTH-004 "which no input reaches".
- Committed example suites that move are named in the change.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize.rs
- inferred: crates/verify/ess-conformance/src/synthesize/row_set.rs
- inferred: crates/verify/ess-conformance/tests/a_creator_under_a_row_set_is_arranged.rs
- inferred: examples/

confidence: medium
