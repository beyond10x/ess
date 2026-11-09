---
format: aep.planning-md/3
id: story:synthesis-witnesses-a-row-addressed-through-a-struct-input-member
kind: story
status: draft
title: Synthesis witnesses a row addressed through a struct input member
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#640
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:via-and-instance-may-name-a-member-of-a-struct-input
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret/execute/existence.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret/execute/related.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/existence.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/related.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/related_list.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/set_effects.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/input_path_addressing.rs
revision: 3
---
## Need

Once `via:`, `instance:`, `each: {in:}` and `{missing:}` may name `input.<path>` into a struct input
(`story:via-and-instance-may-name-a-member-of-a-struct-input`), synthesis must arrange the
referenced rows and send the reference inside the struct. The interpreted target must read the
nested value. Today the related-row, list and instance witnesses send a top-level input, which is
the only form that validates. The top-level control `via: input.parents` synthesizes 3 scenarios and
passes on the interpreted target (triage run, `top-level-list`).

Minimal model: the parent story's `example.g.Make(settings: example.g.Settings)` with
`unknown-parent` guarded `via: input.settings.parents, exists: false`, and an `updates:` command
addressed by `instance: settings.primary`.

## Acceptance

- `ess verify conform synthesize` on the minimal model reports 0 refusals. `example.g.Make/outcome/unknown-parent`
  sends `settings.parents` holding one arranged parent and one unknown id, expects
  `example.g.ParentNotFound`, and its `{missing: input.settings.parents}` payload equals the unknown
  id. `example.g.Make/outcome/made` sends only arranged parents. The scenarios for the update
  addressed by `settings.primary` send the arranged identity inside `settings`, plus a fresh one for
  `unknown_instance:`.
- An `each: {in: input.settings.entries, as: e}` entry is witnessed as a top-level `each:` is.
- `ess verify conform run --target interpreted` passes every scenario. A mutant interpreter that
  reads a top-level `parents` instead of `settings.parents` fails `unknown-parent`.
- Suites of models without an input path keep their bytes, and no suite format moves: the
  scenario's input is the struct value, as today.

## Scope

- inferred: crates/verify/ess-conformance/src/synthesize/related.rs
- inferred: crates/verify/ess-conformance/src/synthesize/related_list.rs
- inferred: crates/verify/ess-conformance/src/synthesize/existence.rs
- inferred: crates/verify/ess-conformance/src/synthesize/set_effects.rs
- inferred: crates/verify/ess-conformance/src/interpret/execute/related.rs
- inferred: crates/verify/ess-conformance/src/interpret/execute/existence.rs
- inferred: crates/verify/ess-conformance/tests/input_path_addressing.rs
confidence: low

## Fit review

1. **Need.** An address read through a struct member must be witnessed, or a target that reads the
   wrong member passes. Reproduction: the parent story's model (triage run, `list-member`,
   `instance-member`).
2. **Class.** Gap, following the parent story.
3. **Already expressible?** Not applicable: the model does not validate today.
4. **Fit.** It reuses the related-row arrangement (a referenced row between two decoys) and the
   per-element list witness (`docs/design/generic-language-features.md` § G2, "Cost" and
   "The same format quantifies over a list of references"). Only where the identity is written in
   the sent input changes.
5. **Second adopter.** As the parent story.
6. **Cost.** No format.
7. **Alternatives.** (a) Leave synthesis refusing such a branch by name: rejected, because the
   construct would then be admitted unchecked. (b) Write the arranged identity at the path: chosen.

## Decisions

accept, redesigned, as the parent story. This story carries synthesis and the interpreted target.

## Source

github:beyond10x/ess#640
