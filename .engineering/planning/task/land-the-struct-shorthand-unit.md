---
format: aep.planning-md/3
id: task:land-the-struct-shorthand-unit
kind: task
status: draft
title: 'Gate and merge unit/ess-u594 (#594, shorthand: on a struct, ess/27)'
summary: The half-done shorthand unit is finished against the batch's formats and lands
refs:
- provider: github
  reference: beyond10x/ess#594
relations:
- decomposes: epic:downstream-reported-gaps
- depends_on: task:land-the-schema-tools-wave-branch
- depends_on: task:land-the-2026-10-13-port
revision: 1
---
## Outcome

Branch `unit/ess-u594` (head `76e662863`, seven commits on `2c0378386`) is gated and merged into
`integrate/ess-downstream-20261018`, so https://github.com/beyond10x/ess/issues/594 ships as the
operator decided on 2026-10-17: option (c), a `kind: struct` type may name one field as its
`shorthand:` and a value of it may be written as that field's scalar alone, in source format
`ess/27`.

## What the branch holds

`story:json-type-discriminated-union` moved to `active` and
`decision-blocker:json-type-discriminated-union` cleared with the decision (both store files are
descendants of the draft/open versions on `integrate/ess-wave-20261017`, so they merge cleanly
after `task:land-the-schema-tools-wave-branch`); the toolchain model admitting `ess/27`
(`68574cd00`); the construct in `ess-domain` (`shorthand.rs`), the IR (`shorthands` beside the
types, omitted when empty), the JSON Schema, OpenAPI and AsyncAPI projection (`oneOf` of the
field's schema and the closed object, `x-ess-shorthand`), the generated Rust, Go and TypeScript
model types, `ess verify diff` (`type/<name>/shorthand-changed`, `ess-diff/20`), synthesis of a
Rust or Go server or the web app refusing such a struct by name, conformance withdrawing each
scenario that would observe one; CHANGELOG, `website/docs/reference/{formats,spec-versions}.md`,
`website/docs/start/first-specification.md`, `docs/design/review-typed-diagnostics.md`.

Status: half-done, committed. Commit bodies record package gates green for `ess-domain`,
`ess-compiler`, `schema-contract` and `ess-diff`; the hand-back notes in the tree (`RESUME.md`,
`.resume/`, untracked, not to be committed) record that `ess-gen`, `ess-synth`, `ess-conformance`
and `ess-cli` were never compiled for this change (builds were refused for disk), and that
`cargo xtask schema`, `task projection-check`, `task test-xtask`, `cargo test -p ess-compiler
--test typed_diagnostics` and `task fmt-check` are owed. The branch allocates `ess/27` and
`ess-diff/20` on its own base; the integrator unifies them with
`task:land-the-2026-10-13-port`'s rows.

## Steps

1. After `task:land-the-schema-tools-wave-branch` and `task:land-the-2026-10-13-port`: merge the
   integration branch into `unit/ess-u594` (or rebase it), resolving
   `crates/specify/ess-domain/src/system.rs` (one `V27` constant, one `FORMAT_HISTORY` row 27
   listing `shorthand:` beside the port's and U3's items), `crates/verify/ess-diff/src/delta.rs`
   (one `ess-diff/20` vocabulary), `models/toolchain/domains/specify.yaml` (`ess/27` once),
   `schemas/generated/ess.schema.json` (regenerate), `CHANGELOG.md`,
   `website/docs/reference/{formats,spec-versions}.md`, and the two pinned tables
   (`lowered-definitions-table.tsv`, `synthesis-suite-bytes-table.tsv`).
2. Gates: `cargo test -p ess-gen --test undeclared_fields --locked`, `cargo test -p ess-synth
   --test refusal_policy --locked`, `cargo test -p ess-conformance --test undeclared_fields
   --locked`, `cargo test -p ess-cli --test validate_undeclared_fields --locked`, `cargo test -p
   schema-contract --test model_ergonomics --locked`, `cargo test -p ess-diff --test
   undeclared_fields --locked`, clippy `-D warnings` on each, `cargo xtask schema`, `task
   projection-check`, `task test-xtask`, `cargo test -p ess-compiler --test typed_diagnostics
   --locked`, `task fmt-check`.
3. Merge into the integration branch (`--no-ff --no-commit`), bot commit, `aep plan artifact
   validate`; move `story:json-type-discriminated-union` to `implemented` with the step-2 gates
   as evidence.
4. Retire `unit/ess-u594` (local only) and `worktree finish --discard-cache --archive ess-u594`.

## Done when

`ess specify validate` admits `shorthand: <field>` on a struct at `ess/27` and refuses it below,
naming `ess/27`; the projection of such a struct is `oneOf` the scalar and the object; the gates of
step 2 are green on the merged head.

## Scope

As the branch's diff against `2c0378386` (58 files): `crates/specify/ess-domain/src/{shorthand.rs,
types.rs, spec.rs, system.rs, undeclared_fields.rs, …}`, `crates/specify/ess-compiler/src/{ir.rs,
resolve.rs}`, `crates/generate/ess-gen/src/{docs.rs,types.rs}`,
`crates/generate/schema-contract/src/realize*.rs`, `crates/generate/ess-synth/src/{failure.rs,
lib.rs, go/mod.rs, rust/mod.rs, web/mod.rs}`,
`crates/verify/ess-conformance/src/synthesize/shorthand.rs`, `crates/verify/ess-diff/src/*.rs`,
`models/toolchain/domains/{generate,specify}.yaml`, `schemas/generated/ess.schema.json`,
`website/docs/reference/{formats,spec-versions}.md`, `website/docs/start/first-specification.md`.
