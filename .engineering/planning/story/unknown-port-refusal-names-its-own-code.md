---
format: aep.planning-md/3
id: story:unknown-port-refusal-names-its-own-code
kind: story
status: draft
title: An unknown-port scenario refusal names its own code, not ESS-SYNTH-008
summary: In enforced_reference_served.rs the unknown-port scenario is refused as ESS-SYNTH-008, the internal-drift code; 0.67.0 prints the same at ess/29. (adversary pass 2026-10-22, F3).
tags:
- defect
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:an-unknown-identity-beside-a-related-guard-is-synthesized
revision: 3
---
## Problem

Found by the 0.68.0 release adversary pass (2026-10-22, finding F3; `review-result:release-0680-adversary-pass-1`) and
still present on 0.92.0. A command whose first branch is `unknown_instance: true` and whose next branch is a
`when_related:` guard read through a stored field of the addressed subject gets no `<command>/outcome/<unknown>`
scenario. Synthesis refuses it as `ESS-SYNTH-008` with the `arrange_related_row` reason, which is about a branch the
related row selects; the unknown-identity branch is selected by the absence of the addressed record and reads no
related row. Removing the `when_related:` branch makes the same scenario appear, so the refusal comes from the
command-wide `arrange_related_row` strategy (`crates/verify/ess-conformance/src/synthesize.rs:1622-1624`,
`:1966-1972`, `related_guard::unarranged` at `crates/verify/ess-conformance/src/synthesize/related_guard.rs:415`)
applied to a branch that does not read the row.

Minimal reproduction: `crates/verify/ess-conformance/tests/fixtures/enforced-reference.yaml` (`ess/30`) with an error
`PortNotFound` and, as `Rename`'s first outcome, `{name: unknown-port, unknown_instance: true, error: PortNotFound}`
before `node-sealed` (`when_related: {via: node_id, predicate: state == Sealed}`).

```text
$ ess verify conform synthesize --path spec --out spec/suite.json
refused: refusal[ESS-SYNTH-008]: outcome integrity.board.Rename/unknown-port has no scenario `integrity.board.Rename/outcome/unknown-port`
  its command's strategy is `arrange_related_row`: the row its `when_related:` guard reads — the one its input names, or the one a stored field of the subject it addresses names (ess/22) — selects its branch, and this scenario family arranges none
13 scenario(s) (0 authored), 1 refusal(s), written to spec/suite.json
```

Control, the same model without `node-sealed`: `9 scenario(s) (0 authored), 0 refusal(s)`, and the suite holds
`Rename/outcome/unknown-port`.

This is the stored-field variant of `story:an-unknown-identity-beside-a-related-guard-is-synthesized` (input-named
variant, beyond10x/ess#554 triage); one fix in the not-found family covers both.

## Acceptance

- `ess verify conform synthesize` over the reproduction above reports no refusal for `Rename/unknown-port`, emits
  `integrity.board.Rename/outcome/unknown-port`, and `ess verify conform run --target interpreted --report-format 2`
  reports it `passed`. Test: `crates/verify/ess-conformance/tests/unknown_identity_beside_related_guard.rs`, case
  `an_unknown_identity_beside_a_stored_reference_guard_is_synthesized`.
- A faulty interpreted target that answers `node-sealed` (or accepts) for an identity no port carries fails that
  scenario. Same file, case `a_target_answering_the_related_guard_for_an_unknown_port_fails`.
- `crates/generate/ess-synth/tests/enforced_reference_served.rs`: the doc comment at :22-24 ("Synthesis arranges no
  unknown port ... (`ESS-SYNTH-008`)") is updated, and the served Rust and Go targets pass the new scenario.
- Where an unknown-identity branch still cannot be synthesized, the refusal does not name the `arrange_related_row`
  strategy for a branch that reads no related row.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize.rs
- cited: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- cited: crates/generate/ess-synth/tests/enforced_reference_served.rs
- inferred: crates/verify/ess-conformance/tests/unknown_identity_beside_related_guard.rs

confidence: medium
