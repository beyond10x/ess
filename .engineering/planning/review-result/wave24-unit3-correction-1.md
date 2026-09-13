---
format: aep.planning-md/1
id: review-result:wave24-unit3-correction-1
kind: review-result
status: archived
title: 'Unit 3 correction round 1: green, and pass 1''s needle premise refuted'
relations:
- reviews: story:enum-variant-in-an-entity-invariant
revision: 2
---
# Unit 3, correction round 1: green, and one of pass 1's premises refuted

Worktree `wt-ebe43ade0b00`, branch `impl/enum-variant-in-an-entity-invariant`, uncommitted over base
`bd722fa964bd225b9755f272e22b45fab334449f`.

```
 crates/specify/ess-compiler/src/resolve.rs                          | 170 ++++++--
 crates/specify/ess-compiler/tests/billing.rs                        | 198 +++++++++
 crates/specify/ess-compiler/tests/fixtures/.../repeated_names.yaml  |  18 +
 crates/specify/ess-compiler/tests/typed_diagnostics.rs              |  84 +++---
 crates/specify/ess-domain/src/expression.rs                         |  32 +-
 crates/specify/ess-domain/tests/expression.rs                       | 142 +++++++
 6 files changed, 601 insertions(+), 43 deletions(-)
```

Plus two adopted adversary files, renamed: `tests/locator_citations.rs` (414 lines) and
`tests/enum_variant_refusals.rs` (110 lines). The assertions inside are byte-identical to what pass 1
wrote; only the doc comments and headers changed.

## Gate

Each command's own exit status, from
`/var/tmp/ess-wave-24/scratch/unit3-correction1/final-exits.txt`:

```
cargo test   --locked -p ess-domain   --no-fail-fast                       0    447 passed
cargo test   --locked -p ess-compiler --no-fail-fast                       0    122 passed, 2 ignored
cargo clippy --locked -p ess-domain   --all-targets -- -D warnings         0
cargo clippy --locked -p ess-compiler --all-targets -- -D warnings         0
RUSTDOCFLAGS='-D warnings' cargo doc --locked --no-deps -p ess-domain      0
RUSTDOCFLAGS='-D warnings' cargo doc --locked --no-deps -p ess-compiler    0
cargo fmt --package ess-domain --package ess-compiler -- --check           0
cargo test --workspace --exclude ess-xtask --locked                        0   2899 passed, 2 ignored
cargo test -p ess-xtask (Taskfile profile)                                 0    169 passed
```

## F1 — the unit refuted pass 1's premise, with a measurement

Pass 1 offered two fixes and argued one of them on the grounds that the trailing-key needle
"cannot match its target, so it can only mislead".

**The unit deleted the key needle and ran the suite: `passed=116 failed=7`, exit 101.** Five of the
seven are shipped assertions that the key needle is what makes correct — for example
`a_reference_across_files_is_cited_in_the_file_that_wrote_it`, which without it falls from
`line: 17, column: 13` (the payload key `headline:` itself) to `line: 7, column: 5` (the command
declaration, coarse).

The needle does match its target for a payload mapping target and for a binding mapping key. Pass
1's premise holds for an **outcome** name, written `- name: filed`, and not for the needle in
general. Option 1 would have demoted five correct precise citations to coarse ones.

So the unit took option 2:

```rust
fn whole_name_matters(needle: &str) -> bool {
    !needle.ends_with(':')
}
```

with the rule written at the code: *`whole_name` narrows monotonically, and narrowing is sound only
for a needle whose every match is the declaration the path names. It may be applied to a needle that
ends in the name being sought; it must not be applied to a needle that is a guess.*

`needles_from_tokens` is the single construction site — `needles_for` and `needles_of_site` both
route through it — and builds two kinds: the three declaration needles `name:`/`id:`/`component:`
followed by the declared name, and the trailing-key guess. The unit added
`the_whole_name_filter_applies_to_declaration_needles_and_not_to_the_trailing_key_guess`, which walks
every needle the module builds for a shared shape corpus and asserts each is one of the two kinds.
Whether that is machine-checkable or a hand-maintained table one level up is pass 2's question.

## F2 — fixed, not deferred

`ValueType` gained `declaring_variants: Option<String>`, filled from the resolution's `terminal` —
the type after transparent unwrapping — whenever `variants` is `Some`. The message now names the
enum and keeps the wrapper beside it:

```
`wrapped_channel == Fax` compares `wrapped_channel` to `Fax`, which `sample.Channel`
(Text, reached through `sample.WrappedChannel`) does not declare as an enum variant;
values: `Email`, `Post`, `Portal`
```

The weakened `contains("sample.")` and the comment that explained it away are gone. The assertion is
now three `contains` calls on `` `sample.State` ``, `` `Ready` `` and `Fax`.

## F5 — fixed, and a second member found that no finding named

Rule the unit wrote: *a comment beside a guard must state what was measured, not a universal the
guard does not prove.*

- `tests/typed_diagnostics.rs:140` (the finding's instance) now states the measurement: 1 declaration
  in `nested.yaml`; 2 of 31 declared names in `examples/billing`.
- `src/resolve.rs`, `a_needle_that_is_only_the_front_of_a_longer_name_is_not_an_occurrence`, carried
  the same universal and was rewritten the same way. **This one was in no finding.** The unit found
  it by grepping its own added lines for universals.

## F3 and F4 — `#[ignore]`d, as instructed

`cargo` prints each with its story:

```
a_command_name_declared_twice_is_refused_even_when_the_first_declaration_is_also_refused ... ignored,
  story:a-masked-first-declaration-hides-a-duplicate-name
an_undeclared_variant_refusal_names_the_file_it_was_read_from ... ignored,
  story:a-refusal-records-the-document-it-was-read-from
```

## Three environmental reds, proved rather than asserted

The unit's first workspace run came back exit 101 with three failures outside its two packages, and
it cleared each rather than labelling it:

| Case | Cause | Cleared by |
|---|---|---|
| `ess-cli` `an_output_created_after_preflight_is_preserved` | `observed_bindings.rs:518` refuses output under a Git checkout; the home directory is itself a git checkout and `TMPDIR` sat inside it | `TMPDIR` on `/var/tmp` → `passed=534 failed=0`, exit 0 |
| `ess-cli` `failed_live_collection_is_json_unknown_and_does_not_use_an_old_snapshot` | same | same |
| `ess-xtask` `current_compiled_provider_executes_one_guard_…` | `Taskfile.yml:42` excludes `ess-xtask` from the workspace run and runs it under pinned profile knobs; run naively it fails on `DEBUG` | the documented invocation → `passed=169 failed=0`, exit 0 |

The third is `story:the-metadata-guard-rejects-every-build-but-one`, filed from unit 2's pass 2. Two
units reached it independently in this wave.
