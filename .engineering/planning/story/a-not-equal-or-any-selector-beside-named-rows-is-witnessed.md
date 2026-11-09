---
format: aep.planning-md/3
id: story:a-not-equal-or-any-selector-beside-named-rows-is-witnessed
kind: story
status: draft
title: A not-equal or any selector beside named rows is witnessed
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
  path: crates/specify/ess-domain/tests/fixtures/identity-beside-row-set.yaml
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/row_set.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/adversary_d368_u4_pass1.rs
revision: 5
---
## Finding

Found by wave D368 U4's adversary pass (`review-result:adversary-d368-u4-pass-1`), out of that
story's Acceptance: a row-set selector with no equality conjunct (only `!=`, or `any:`) beside named
rows refuses every branch of the command, the named-row branches included. The refusal is honest
that no scenario can decide the rows, but such a command has no suite at all, and each refusal
calls the branch itself a `row set` without naming the selector.

The gate is `decidable` (`crates/verify/ess-conformance/src/synthesize/row_set.rs:3128-3140`): a
selection not `scoped` by an equality between a String or Uuid field and the input or the subject
refuses.

Reproduction on 0.92.0: the fixture
`crates/specify/ess-domain/tests/fixtures/identity-beside-row-set.yaml` (a `Connect` command with
four named-row branches on `input.from_port` / `input.to_port` and one row-set branch
`source-taken`) synthesizes 7 scenarios and 0 refusals. Change only the `source-taken` selector:

```yaml
      - name: source-taken
        when_related:
          entity: wiring.board.Wire
          where: from_port != input.from_port      # or {any: [from_port == input.from_port, to_port == input.to_port]}
          exists: true
        error: wiring.board.SourceTaken
```

```text
$ ess specify validate --path ne
wiring v1 — 1 file(s), valid
$ ess verify conform synthesize --path ne --out suite-ne
refused: refusal[ESS-SYNTH-001]: outcome wiring.board.Connect/no-source has no scenario `wiring.board.Connect/outcome/no-source`
  no witness: `wiring.board.Connect/no-source` is `row set`, which selects rows by no equality between a String or Uuid field and the input or the addressed subject, so the rows it counts are not only the scenario's own
... (the same for no-sink, source-not-output, sink-not-input, source-taken, connected)
1 scenario(s) (0 authored), 6 refusal(s), written to suite-ne
```

The `any:` selector gives the same six refusals.

## Acceptance

- For both variants above, `ess verify conform synthesize` writes a passing scenario (against
  `ess verify conform run --target interpreted`) for each `Connect` branch where an arrangement
  exists: at least `no-source`, `no-sink`, `source-not-output` and `sink-not-input`, which a target
  can reach with no `Wire` row stored.
- Each `Connect` branch still refused is refused naming the `source-taken` selector expression
  (`from_port != input.from_port`, or the `any:`), not calling the branch itself a `row set`.
- The unmodified fixture keeps its 7 scenarios and suite bytes; any committed model under
  `examples/` or `suites/generated/` whose suite moves is listed in the change's CHANGELOG entry.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize/row_set.rs
- cited: crates/specify/ess-domain/tests/fixtures/identity-beside-row-set.yaml
- inferred: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- inferred: crates/verify/ess-conformance/tests/adversary_d368_u4_pass1.rs
- inferred: CHANGELOG.md

confidence: medium
