---
format: aep.planning-md/3
id: task:salvage-the-related-rows-quantifier-tree
kind: task
status: draft
title: 'Preserve and finish the uncommitted #551 implementation on the integration branch'
summary: The 30-file tree ess-u551-impl becomes the rows quantifier at ess/27
refs:
- provider: github
  reference: beyond10x/ess#551
relations:
- decomposes: epic:generic-row-queries
- depends_on: task:land-the-2026-10-13-port
revision: 1
---
## Outcome

The uncommitted implementation of https://github.com/beyond10x/ess/issues/551 in the managed tree
`ess-u551-impl` is preserved, brought onto `integrate/ess-downstream-20261018` after
`task:land-the-2026-10-13-port`, finished against `story:related-rows-quantifier-in-a-selector`
(arrives with the port) and merged, so a row selector can test each candidate's related rows
(`exists: {in: {entity: <E>}, as: <b>, that: <P>}`) from `ess/27`.

## What exists

- Design: `docs/design/related-rows-quantifier.md` on the port branch (head `30de58532`) (status
  "design, not implemented"); story `story:related-rows-quantifier-in-a-selector` (draft) on the
  same branch, with its seven-question fit review.
- Tree `ess-u551-impl` (branch `impl/ess-551-r1r2`, which has no commit of its own: its head
  `16b0bdd9c` is the 2026-10-13 integration head after #549 U3+U4). The tree holds 30 uncommitted
  files, +2154/−113, dated 2026-10-13: new `crates/specify/ess-domain/src/command/related_rows.rs`
  (the quantifier's admission rules, written for `ess/25`), `Collection::Rows` in
  `crates/specify/ess-primitives/src/predicate.rs` and `facts.rs`, new tests
  `crates/specify/ess-domain/tests/related_rows_quantifier.rs`,
  `crates/specify/ess-primitives/tests/related_rows_quantifier.rs`,
  `crates/specify/ess-compiler/tests/related_rows_ir.rs`, and edits in `ess-domain`
  (`command.rs`, `command/{row_set,set_effects}.rs`, `expression.rs`, `expression/lexical.rs`,
  `view/measure.rs`), `ess-conformance` (`input.rs`, `now_offset.rs`, `synthesize.rs`,
  `synthesize/{aggregate.rs,aggregate/contrast.rs,identity.rs,subject_fact.rs}`, `witness.rs`),
  `ess-synth` (`go/invariant.rs`, `rust/invariant.rs`), `ess-entity-runtime/src/lib.rs` and
  `ess-cli-project/tests/consumer_cli_types.rs`. Nothing in the tree records which tests passed.

Status: half-done, uncommitted, on a base 102 commits behind `origin/main` and below the format
move to `ess/27`.

## Steps

1. Preserve first, never discard: the owning session commits the tree on `impl/ess-551-r1r2` as
   one `wip(related-rows)` commit, or the integrator runs `git -C <tree> diff > <patch>` and
   `git -C <tree> diff --binary --stat` and keeps the patch beside the integration tree. No
   `git checkout`, `restore` or `reset` in that tree without the operator's word.
2. After `task:land-the-2026-10-13-port`: `git apply --3way` the patch (or rebase the commit) onto a
   managed worktree of the integration branch; move every `ess/25` gate in it to `ess/27`
   (`FormatVersion::V27`), as the port did for its own lines.
3. Finish per the story's acceptance: admission in the `where:` of `{related: {entity, where,
   field}}`, `{related: {entity, where, order_by, first}}` and a row-set `when_related:`'s `where:`
   and `forall:`; refusal by name at every other site; `forall` refused with its `not exists`
   rewrite; one level only; the body tied to the candidate by a top-level `<b>.<f> == <candidate
   field>`; synthesis arranging a higher-scored candidate that fails the test so a target that
   ignores it fails; the interpreter, generated Rust and Go behaviour (or a named refusal) and
   Entity Runtime lowering (refusal by name) agree.
4. Package gates: `ess-primitives`, `ess-domain`, `ess-compiler`, `ess-conformance` (one `--test`
   at a time), `ess-synth`, `ess-entity-runtime`, `task test-xtask`, `typed_diagnostics`, `task
   fmt-check`, clippy `-D warnings`; re-pin the synthesis suite bytes table.
5. Merge (`--no-ff --no-commit`), bot commit, `aep plan artifact validate`; move
   `story:related-rows-quantifier-in-a-selector` to `implemented` with the gates as evidence; add
   the CHANGELOG bullet (the port's `[Unreleased]` has none for #551).
6. `worktree finish --discard-cache --archive ess-u551-impl`; delete `impl/ess-551-r1r2` and
   `impl/ess-551-design` (local only).

## Done when

The design page's three refused spellings validate or are refused as it states, the story's named
tests are green, and #551's reproduction (best slot among those whose two sides each hold a
`Present` membership) synthesizes a witness in which a higher-scored slot failing the test is
present.

## Risks

Nothing in the tree says what compiled or passed; the patch may need more than a format move after
the port's `row_set.rs` rewrite (multi-selector `Plan`, owner links) and U6's related-value changes.
Confirm with the operator that no session is still working in `ess-u551-impl` before step 1.
