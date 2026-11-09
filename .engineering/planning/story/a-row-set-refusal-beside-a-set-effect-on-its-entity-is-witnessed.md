---
format: aep.planning-md/3
id: story:a-row-set-refusal-beside-a-set-effect-on-its-entity-is-witnessed
kind: story
status: draft
title: A row-set refusal beside a set effect on its entity is witnessed
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#638
relations:
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/row_set.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/set_effects.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/row_set_beside_set_effect.rs
revision: 4
---
## Need

A command addresses an existing row by a tuple of stored fields, not by its identity. It refuses
with `404` when no row holds the tuple, and otherwise updates the row. The language states this
today: a row-set refusal plus a set effect over the same selection. Synthesis cannot witness the
accepting branch, because the command under test changes rows of the selector's entity through a
set effect, which the row-set arrangement does not follow
(`crates/verify/ess-conformance/src/synthesize/row_set.rs:266-276`, `UNFOLLOWED`/`ROWS_UNFOLLOWED`).

Minimal reproduction (0.92.0, `ess/36`, members from input):

```yaml
  - name: example.g.Disable
    input: [{name: account_id, type: String}, {name: app_id, type: String}]
    outcomes:
      - name: unknown
        when_related: {entity: example.g.Grant, where: {all: [account_id == input.account_id, app_id == input.app_id]}, exists: false}
        error: example.g.NotBound
      - name: disabled
        updates: example.g.Grant
        instances: {where: {all: [account_id == input.account_id, app_id == input.app_id]}}
        sets: {enabled: false}
```

```text
refusal[ESS-SYNTH-001]: outcome example.g.Disable/disabled has no scenario `example.g.Disable/outcome/disabled`
  no witness: ``example.g.Disable/disabled` a step of the scenario changes rows of the selector's entity in a way synthesis does not follow (a set effect, a binding or a direct setup)` is `row set`, ...
```

## Acceptance

- `ess verify conform synthesize` on the reproduction reports 0 refusals. `example.g.Disable/outcome/disabled`
  arranges a `Grant` through `example.g.Bind` with the sent tuple and a decoy that differs in one
  member. It expects `example.g.Disabled`, `enabled: false` on the selected row, and the decoy
  unchanged.
- `example.g.Disable/outcome/unknown` sends a tuple no row holds, with the decoy present, and
  expects `example.g.NotBound`.
- `ess verify conform run --target interpreted` passes both.
- With `account_id == caller.account_id` in both selections (after
  beyond10x/ess#636), the same scenarios use the caller assignment, so a second caller's identical
  `app_id` is not selected.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize/row_set.rs
- inferred: crates/verify/ess-conformance/src/synthesize/set_effects.rs
- inferred: crates/verify/ess-conformance/tests/row_set_beside_set_effect.rs
confidence: medium

## Fit review

1. **Need.** A row addressed by a non-identity tuple, with a stated not-found answer, must be
   checked. Reproduction above (triage run `address-idiom-input`).
2. **Class.** Gap in synthesis: the specification validates, and no scenario is produced.
3. **Already expressible?** In the language, yes (it validates). In the suite, no.
4. **Fit.** This is the G2 effect-over-selection row (`docs/design/generic-language-features.md`
   § G2: `instances: {where: p}` is `each r in rows(E where p)`). Synthesis already arranges
   selected rows against decoys (`generic-language-features.md` § G2, "Cost"). This extends that to
   a command whose own effect writes the selected entity: the post-state is the arranged rows with
   the effect applied, which the interpreter computes. The same gap blocks
   `story:an-update-addresses-its-record-by-an-optional-identity-or-a-unique-field` (lookup by
   email).
5. **Second adopter.** "Disable the subscription of this customer to this product", with no
   subscription id at hand.
6. **Cost.** No format. Suites of models without the pair keep their bytes.
7. **Alternatives.** (a) An `instance:` map addressing a row by a tuple: rejected by the accepted
   compound-key record (option B deferred). (b) An authored scenario: rejected, because the branch
   stays unchecked by default. (c) Let the row-set arrangement follow the command's own set effect:
   chosen.

## Decisions

accept, redesigned. Rows are addressed by a selection (G2), not by an `instance:` map. This story
makes that selection witnessable.

## Source

github:beyond10x/ess#638
