---
format: aep.planning-md/3
id: story:generated-rust-and-go-keep-a-view-that-reads-a-parameter-s-row-as-a-named-obligation-entity-runtime-refuses-it
kind: story
status: draft
title: Generated Rust and Go keep a view that reads a parameter's row as a named obligation, Entity Runtime refuses it by name, and `ess verify diff` classifies the change
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:from-ess-39-a-view-s-filter-reads-the-row-a-parameter-names-one-hop-and-walks-that-row-s-list-fields
- depends_on: story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess
revision: 4
---
## Need

The fit-review rule "every target handles it or refuses it by name"
(`.agents/skills/assessing-external-requests/SKILL.md`, question 4) holds for the `ess/39` filter of
`g08-view-param-row` only in validation and the interpreter. The `ess/28` navigation shows the
pattern for the rest: "Generated Rust and Go keep such a view's query as a named obligation. `ess
verify diff` reports a field's source moving as `field-source-changed`"
(`website/docs/guides/specify/views.md:263-264`; tests
`crates/generate/ess-synth/tests/view_reference_owed.rs`). Report item 8, targets half.

## Outcome

Generated Rust and Go view queries keep a view whose filter reads `param.<p>.<field>` as a named
obligation naming the path, as they keep an `ess/28` navigated view; Entity Runtime lowering refuses
it by name; generated documentation shows the filter; `ess verify diff` classifies adding, removing
or changing such a path.

## Acceptance

Format (D77, 2026-11-24): this story's construct lands in `ess/38`, opened by story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess; where this text names `ess/37`, `ess/39`, "the new format", "the next source format" or a format still to be decided as the format of this construct, read `ess/38`, and where it names `ess/36` as the format below it, read `ess/37` (`ess/37` shipped empty in 0.93.0); a reproduction model keeps the format it declares.

- First: a test beside `view_reference_owed.rs` generates Rust and Go for the example of
  `g08-view-param-row` and finds the obligation naming `param.order_id.lines`; today generation of
  that example fails or omits the view (it is refused at validation).
- Entity Runtime lowering of the example refuses the view with a message naming the parameter path
  and `ess/39`; a row in `lowered-definitions-table.tsv` records it if that table lists refusals.
- `ess verify diff` between the example and a copy whose filter reads another field of the named
  row reports a classified change; which class it is, and whether a new one is needed, is decided in
  the story from the existing filter classifications (I don't know them).
- The generated HTML documentation shows the filter as written.

## Fit review

Tree `bcfc51555`, read-only. Covers report item 8 with `g08-view-param-row` and
`g08-view-param-row-synthesis`.

1. **Need.** No target silently ignores the new filter.
2. **Class.** Completion of a *gap* fix across targets.
3. **Already expressible?** n/a.
4. **Fit.** The `ess/28` pattern, unchanged.
5. **Second adopter.** As `g08-view-param-row`.
6. **Cost.** Obligation text, one refusal, one diff case. No format change of its own.
7. **Considered.** Implementing the query in generated Rust and Go now: a larger change that
   `ess/28` navigation has not made either; a later story with it.

## Decisions

- None beyond P1-D6 (open, with the owner as AQ5).

- FMT-B (decided: A, 2026-11-24): this story moves to `ess/38` with the rest of the pull request (D77: no `ess/39`); the Acceptance line on the format says so.

## Scope

- inferred: `crates/generate/ess-synth/src/view_query.rs`, `crates/generate/ess-synth/src/rust/entity.rs`
- cited: `crates/generate/ess-synth/tests/view_reference_owed.rs` (beside it, a new test)
- inferred: `crates/generate/ess-entity-runtime/` (lowering refusal),
  `crates/generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv`
- cited: `crates/verify/ess-diff/src/change.rs`, `crates/verify/ess-diff/src/diff.rs`; beside `crates/verify/ess-diff/tests/view_reference.rs`, a new test
- cited: `crates/generate/ess-gen/src/html.rs`
- Size: M (about 250 changed lines).

## Depends on

- `g08-view-param-row`. Independent of `g08-view-param-row-synthesis`.
