---
format: aep.planning-md/3
id: task:land-the-2026-10-13-port
kind: task
status: draft
title: 'Compile, gate and merge the port branch (head 30de58532) (#542-#549, #552) under ess/27'
summary: The 2026-10-13 integration, rebased onto 0.62.0, lands on the integration branch with the batch's formats
refs:
- provider: github
  reference: beyond10x/ess#542
- provider: github
  reference: beyond10x/ess#543
- provider: github
  reference: beyond10x/ess#544
- provider: github
  reference: beyond10x/ess#545
- provider: github
  reference: beyond10x/ess#546
- provider: github
  reference: beyond10x/ess#547
- provider: github
  reference: beyond10x/ess#549
- provider: github
  reference: beyond10x/ess#552
relations:
- decomposes: epic:downstream-reported-gaps
- delivers: story:system-issued-command-is-awaited-not-driven
- delivers: story:system-issued-command-await-in-go-and-typescript-runners
- delivers: story:payload-source-reads-the-head-of-an-ordered-selection
- delivers: story:authored-exact-value-on-generated-field-is-refused
- delivers: story:minted-value-present-and-shared-with-its-payload
revision: 1
---
## Outcome

The port branch (head `30de58532`, 67 commits on `2c0378386`, two commits
behind `origin/main`) compiles, passes its package gates and is merged into
`integrate/ess-downstream-20261018`, so #542 (both stages), #543, #544, #545, #546, #547, #549 and
#552 ship under one source format (`ess/27`), one suite format pair (`ess-conformance/52` and
`/53`) and one delta format (`ess-diff/20`) shared with the batch's in-flight units.

## What the branch holds

The 2026-10-13 integration the 2026-10-13 integration branch (head `786d571d6`) (`786d571d6`, 64 commits of the
`impl/ess-542*`, `impl/ess-543*`, `impl/ess-544`, `impl/ess-545`, `impl/ess-546`,
`impl/ess-547*`, `impl/ess-549*`, `impl/ess-551-design` and `impl/ess-fix-regressions`
branches, every one an ancestor of this head), merged onto the 0.62.0 head in `3da5e98c0`
(36 conflicted files resolved, named in the merge's commit body), the toolchain model admitting
`ess/27` (`b46e20dbb`), and the format move from `ess/25` to `ess/27`, suite `/50`,`/51` to
`/52`,`/53`, `ess-diff/19` to `/20` (`30de58532`, whose body also lists the compile fixes it
made).

Per construct: `docs/design/system-issued-commands.md` (#542: `system: true` actors,
`await_within`, `await_outcome`/`expect_not_issued` steps, the interpreter, the Rust runner and the
generated Go and TypeScript runtimes), `ordered-related-head.md` (#543), `authored-generated-values.md`
(#544, `ESS-AUTHOR-042`), `minted-value-sources.md` (#545), `unread-optional-inputs.md` (#546),
`set-effect-identity-filters.md` (#547, with every code target owing a set effect instead of
refusing the workspace), `published-value-operand.md` (#549), the minted-head fix (#552), and the
design page `related-rows-quantifier.md` for #551 (design only; the implementation is
`task:salvage-the-related-rows-quantifier-tree`). The `CHANGELOG.md` `[Unreleased]` section on the
branch carries one bullet per issue.

Store files on the branch: new `story:minted-head-refuses-only-its-branch`,
`story:published-value-operand-in-a-selector`, `story:related-rows-quantifier-in-a-selector`,
`story:set-effect-filters-read-the-identity`, `story:synthesis-leaves-unread-optional-inputs-absent`;
revised `story:payload-source-reads-the-head-of-an-ordered-selection` and
`story:system-issued-command-is-awaited-not-driven`. None of these is edited on
`plan/ess-reconcile-20261018`, so the merge brings them in without conflict.

Status: unshipped, never compiled green. The port tree holds an uncommitted
`RESUME.md` (hand-back notes, not to be committed) and two cosmetic uncommitted edits
(`crates/specify/ess-compiler/src/resolve.rs` import alias,
`crates/verify/ess-conformance/src/synthesize/row_set.rs` closure rename and a `Selections` type
alias). Its last recorded `cargo check --workspace --all-targets --locked` exited 101 before the
fixes in `30de58532`; nothing downstream of `ess-domain` has been compiled since.

## Steps

1. Commit the two cosmetic edits on the branch (bot commit); leave `RESUME.md` out.
2. `git merge origin/main` (two commits: `0d633ee64` re-pins
   `crates/generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv`, which the
   branch also re-pins; take the branch's and re-pin in step 4).
3. `cargo check --workspace --all-targets --locked`; fix what the merge left (expected: match arms
   for `PayloadRead`/`RelatedHead`/`Generated { present }` in crates below `ess-domain`).
4. `cargo xtask schema`, `cargo xtask format-history`, `task projection-check`, `task
   example-check`; re-pin `crates/specify/ess-compiler/tests/fixtures/selection-precedence-table.tsv`,
   `crates/generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv` and
   `crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv` with the command
   each test's REPIN message names.
5. Package gates per the AGENTS.md table: `ess-domain`, `ess-compiler`, `ess-cli`,
   `ess-conformance` (one `--test` binary at a time, `cargo clean -p ess-conformance` between
   batches), `ess-synth`, `ess-entity-runtime`, `ess-diff`, `task test-xtask`, `cargo test -p
   ess-compiler --test typed_diagnostics`, `task fmt-check`, `task ci-lint`, clippy `-D warnings`
   per crate.
6. Add the case the hand-back notes name: a `before_row:` refusal whose error payload reads
   `{related: …, first}` is refused like any related read (an `ess-domain` test).
7. Merge into `integrate/ess-downstream-20261018` after the in-flight units U1–U6 are in, with
   `git merge --no-ff --no-commit`. Expected conflicts, each resolved by reading both sides:
   `crates/specify/ess-domain/src/system.rs` (`FORMAT_HISTORY` row 27: one row listing this
   work's five items beside U3's Boolean binding leaf and, later, #594's `shorthand:`;
   `SUPPORTED_FORMATS` once), the suite catalogue rows for `/52`,`/53` (the await pair beside U5's
   wrapped-invariant observations: one pair, cumulative), `crates/verify/ess-diff` delta format 20
   (one vocabulary list), `crates/verify/ess-conformance/src/synthesize.rs`,
   `synthesize/row_set.rs`, `synthesize/caller.rs`, `synthesize/set_effects.rs` (U1 and U6),
   `CHANGELOG.md` `[Unreleased]`, `crates/edge/ess-xtask/src/docs.rs` format rows,
   `website/docs/reference/{formats,spec-versions}.md`. Re-run step 5 on the merged head.
8. Bot commit; `aep plan artifact validate` (seven store files arrive). Move the nine stories
   (`story:system-issued-command-is-awaited-not-driven`,
   `story:system-issued-command-await-in-go-and-typescript-runners`,
   `story:payload-source-reads-the-head-of-an-ordered-selection`,
   `story:authored-exact-value-on-generated-field-is-refused`,
   `story:minted-value-present-and-shared-with-its-payload`,
   `story:synthesis-leaves-unread-optional-inputs-absent`,
   `story:set-effect-filters-read-the-identity`, `story:published-value-operand-in-a-selector`,
   `story:minted-head-refuses-only-its-branch`) through their ladder to `implemented` with the
   step-5 gates as `test_result` evidence; `story:related-rows-quantifier-in-a-selector` stays
   draft until its task lands.
9. Retire after the merge is on the integration branch (all local only): the 21 `impl/ess-54*`
   and `impl/ess-fix-regressions` branches, the 2026-10-13 integration branch (head `786d571d6`),
   the port branch; the trees at `786d571d6`, `ess-fix-regress` (one cosmetic
   uncommitted edit, the same rename as step 1), `ess-regress-base` (detached at `107971db6`,
   clean), the port tree, each with `worktree finish --discard-cache --archive`.

## Done when

The integration branch compiles, every gate of step 5 is green on the merged head, `ess specify
formats` lists `ess/27` with every construct of this task and of U3, a suite carrying an
`await_outcome` step is `ess-conformance/52`, and the nine stories are `implemented`.

## Scope

As the branch's diff against `origin/main` (172 files): `crates/specify/ess-domain/src/{actor.rs,
binding.rs, command.rs, command/*.rs, entity.rs, spec.rs, system.rs, system_issued.rs, view.rs}`,
`crates/specify/ess-compiler/src/{ir.rs,resolve.rs}`, `crates/verify/ess-conformance/src/**`,
`crates/verify/ess-diff/src/**`, `crates/generate/ess-synth/src/**`,
`crates/generate/ess-entity-runtime/src/**`, `crates/edge/ess-xtask/src/docs.rs`,
`models/toolchain/domains/specify.yaml`, `schemas/generated/ess.schema.json`, eight design pages,
`website/docs/reference/{formats,spec-versions,diagnostics}.md`, `CHANGELOG.md`.
