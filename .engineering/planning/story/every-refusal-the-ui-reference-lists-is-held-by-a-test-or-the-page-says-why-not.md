---
format: aep.planning-md/3
id: story:every-refusal-the-ui-reference-lists-is-held-by-a-test-or-the-page-says-why-not
kind: story
status: draft
title: Every refusal the UI reference lists is held by a test, or the page says why not, and the support page's mutation row names the report formats the code writes
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/support.rs
- confidence: cited
  path: crates/ui/ess-ui-check/src/rules.rs
- confidence: cited
  path: crates/ui/ess-ui-check/tests/adversary_from_schema_pass1.rs
- confidence: inferred
  path: crates/ui/ess-ui-check/tests/placement_refusals.rs
- confidence: cited
  path: crates/ui/ess-ui-docs/src/model.rs
- confidence: inferred
  path: crates/ui/ess-ui-docs/tests/reference.rs
- confidence: cited
  path: crates/ui/ess-ui-react/src/emit.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/mutate.rs
- confidence: cited
  path: schemas/ui/ess-ui.schema.yaml
- confidence: cited
  path: website/docs/reference/ess-ui.md
- confidence: cited
  path: website/docs/status/where-this-stands.md
revision: 13
---
## Outcome

Each refusal that `website/docs/reference/ess-ui.md` lists for placing state is refused by a test
that names it, or is listed with the reason no tool of this repository refuses it. A new refusal
added to `schemas/ui/ess-ui.schema.yaml` fails a test until one of the two is true. The mutation
row of the generated status page names every `ess-mutation-report/N` the code can write, and a test
fails when a format constant exists that the row does not name.

## Need

Two findings of a documentation review, each checked against the source at `16f2fc88c2`.

**Three of five listed refusals are held by nothing.**
`PlacementProfile.resolution.refusals` in `schemas/ui/ess-ui.schema.yaml` (lines 1569-1575) lists
five rules, each `result: refuse`, and `website/docs/reference/ess-ui.md:2553-2557` renders them as
the refusals of placement:

1. `{state.sensitive: true, store: [url, session_storage, local_storage]}` is refused by
   `ess ui check` (`crates/ui/ess-ui-check/src/rules.rs:1466`) and by the React emitter
   (`crates/ui/ess-ui-react/src/emit.rs:396`, `fn refusal`), and held by
   `sensitive_state_in_the_url_or_browser_storage_is_refused`
   (`crates/ui/ess-ui-check/tests/adversary_from_schema_pass1.rs:49`) and
   `crates/ui/ess-ui-react/tests/adversary_pass1.rs:117`, `:135`.
2. `{state.class: credential, store_not_in: [memory, server_session]}` is refused at
   `rules.rs:1475` and `emit.rs:396`, held by `a_credential_in_local_storage_is_refused`
   (`adversary_from_schema_pass1.rs:65`) and `adversary_pass1.rs:148`.
3. `{store: url, value_serializable: false}`.
4. `{store: url, value_bytes_over: 2048}`.
5. `{renderer.thin_only: true, resolved_by_profile: fat}`.

Rules 3 to 5 appear nowhere but the schema and its rendered page: `git grep -n
'value_serializable\|value_bytes_over\|thin_only\|resolved_by_profile' -- crates` prints no line
(the only `2048` that is related is the `max_bytes_per_page: 2048` constraint of the `url` store,
`schemas/ui/ess-ui.schema.yaml:1496`, which no code reads either). `ess ui check` does not run
them, the React emitter's `fn refusal` has two cases and not these, and no test names them. The
header of `adversary_from_schema_pass1.rs` says its job was "rules the schema states as refusals
that the loader accepts", routed to the checks story; these three were not part of it.

What "not checked" means in the generator: nothing. The review's wording comes from the unmerged
branch `origin/docs/currency-and-unslop`, commit `398fe73609` ("docs(reference): correct the
sources of the generated reference pages"), which moves rules 3 to 5 into a key beside `refusals`:

```
not_checked:
  reason: a renderer's concern; ess ui check does not run them
  refusals: [rules 3, 4, 5]
```

`crates/ui/ess-ui-docs` has no `not_checked` in `src/` at either commit; the generator renders an
unknown key as an extra, in schema order, like any other (`model.rs`, `extras`), so the page shows
the words and nothing reads them. The branch therefore states the gap honestly but leaves it
unheld: one reason for three rules, no test that the three really are outside `ess ui check`, and no
test that fails when a sixth refusal is added to the schema and enforced nowhere.

Which of the three could be checked by the document alone:

- Rule 3, `value_serializable: false`: the type language of the schema has no type a value of which
  cannot be serialized (`primitives`: `string`, `integer`, `number`, `boolean`, `duration`,
  `timestamp`, `date`, `time`, `json`, `expr`, `name`, `token_name`; `constructors`: `list`, `map`,
  `optional`, `enum`, `one_of`, `record`, `ref`, `const`). By reading, it cannot occur in a document
  that validates.
- Rule 4, `value_bytes_over: 2048`: a run-time size of a value. A document states a value only as a
  state's `default:`, so `ess ui check` could compare the serialized default of a `url` state with
  2048 bytes; a value the user types is the renderer's.
- Rule 5, `renderer.thin_only: true, resolved_by_profile: fat`: needs a renderer that declares
  `thin_only`. `git grep -n 'thin_only' -- crates` finds none, so no renderer of this repository
  is thin-only and the rule has no subject here.

**The mutation row names two of six report formats.**
`crates/edge/ess-xtask/src/support.rs:610` writes the row "Mutation audit" of
`website/docs/status/where-this-stands.md:99` from a literal: "writes `ess-mutation-report/3`, or
`/4` for a component, a known-failure declaration or an `emit-swap` site with no alternative".
`crates/verify/ess-conformance/src/mutate.rs:72-97` defines `REPORT_FORMAT` (`/3`) and
`REPORT_FORMAT_4` to `REPORT_FORMAT_8`, and the help text of `verify conform mutate --report-out`
(`crates/edge/ess-cli/src/main.rs:1027`) lists `/3`, `/4`, `/5`, `/6` and `/8`. The row is stale by
`/5`, `/6`, `/7` and `/8`. Nothing could have caught it: `cargo xtask support --check` compares the
page with what `support.rs` generates, and the generator holds the literal, so the page and its
source agree and both disagree with the code. The other rows read what they say from the binary
(`choices(&mutate_help, "--class")`, `json_marker(&detailed, "/summary/format")`, support.rs
lines 340-365); this one's format list is the one that does not. (The help text omits `/7`; the row
and the help are to be fixed from one source.)

## Fit review

A documentation-defect finding with a generic cause, not an adopter request: a generated page states
a property ("refused", "writes") that no test ties to the code. The answer is a mechanical tie, not a
rewrite: one test that reads the schema, one that reads the constants. No format, no generated
byte of a spec and no `ess/N` moves. It depends on `docs/currency-and-unslop` for the
`not_checked` key: if that branch has not merged when this story starts, the story's first change is
the key itself (Design 1), with the branch's wording.

## Design

1. The schema states the status of every refusal in one place: `refusals` holds those a tool
   refuses, and `not_checked.refusals` holds the others with a `reason` per entry instead of one
   shared sentence. Entries read `{when: ..., result: refuse, reason: ...}`; the generated page
   renders the reason beside each. `ess-ui-docs` renders it through its existing extras path, and
   no generator code changes.
2. A test in `crates/ui/ess-ui-check/tests/` (`placement_refusals.rs`) reads the schema's
   `PlacementProfile.resolution`, `refusals` and `not_checked.refusals`, and holds a table with one
   row per refusal, keyed by the `when:` flow text: the refusals of the first list each carry a
   fixture document that `ess ui check` rejects with a finding naming the rule (rows 1 and 2
   exist as tests; the table adopts them), and the refusals of the second list each carry a fixture
   document that `ess ui check` accepts, which proves the page's claim that the check does not run
   it. The test fails when a refusal of the schema has no row, and when a row's list is not the
   schema's list. Adding a sixth refusal to the schema fails the test until it is held or stated.
3. Per refusal, the recommended outcome (decision A offers the others):
   - Rule 3 stays not checked, with the reason "no type of the document language is unserializable;
     a renderer that adds one refuses it" (a statement the table test cannot hold, since there is
     nothing to build: the reason names the type list, and the test compares the list of
     primitives and constructors with the sentence).
   - Rule 4 becomes checked for a `url` state's `default:` (serialized as `ess ui` writes it, at
     most 2048 bytes), with the reason a typed value is the renderer's kept on the page, and a test
     of one default of 2048 bytes (accepted) and one of 2049 (refused).
   - Rule 5 stays not checked, with the reason "no renderer shipped here declares `thin_only`".
4. The mutation row of `support.rs` is generated from the constants: it names `REPORT_FORMAT` and
   each `REPORT_FORMAT_N` of `ess_conformance::mutate` (a dependency `ess-xtask` already has), with
   the condition each adds taken from the doc comment's first sentence or kept as prose per
   constant in one table of `support.rs` that a test checks has a row for every constant. The
   `--report-out` help of `verify conform mutate` is derived from the same table (or states
   `/3` to `/8` by constant), so the help and the row cannot differ.
5. `cargo xtask support` regenerates `website/docs/status/where-this-stands.md`; the support check
   already fails when the page and the generator differ.

## Acceptance

- `git grep -n 'value_serializable\|value_bytes_over\|thin_only' -- crates schemas` names, for each
  of the three, the test row that holds it (a fixture accepted or refused, as Design 3 decides).
- Adding `{when: {store: memory, value_bytes_over: 1}, result: refuse}` to `refusals` in a scratch
  copy of the schema makes `placement_refusals` fail with a message naming that refusal and the two
  ways out (held by a row, or moved to `not_checked` with a reason); shown once in the pull request.
- The generated reference page lists each refusal with its status, and a reader of
  `website/docs/reference/ess-ui.md` can tell the refusals `ess ui check` raises from those it does
  not, each with its own reason.
- The mutation row of `website/docs/status/where-this-stands.md` names `/3` and `/4` to `/8`, with
  the condition of each; adding a `REPORT_FORMAT_9` constant without a row of the table fails
  `cargo test -p ess-xtask`.
- `task test-xtask`, `cargo test -p ess-ui-check` and `cargo test -p ess-ui-docs` pass, each test
  under 60 s and the step within 120 s; the new test reads two files and builds no suite, and its
  time is named in the pull request.
- `task site-build` is not needed: the page is a generated source, and `cargo xtask support
  --check` and `projection-check` cover it.

## Decisions

A. Rule 4 (`value_bytes_over: 2048` for a `url` state).
   - Check the serialized `default:` in `ess ui check` and say on the page that a typed value is
     the renderer's (recommended: one real, static slice of the rule, a short test).
   - State it not checked, with the reason that a value's size is only known at run time (cheapest;
     the rule then has no code behind it).
   - Enforce it in the React emitter as a run-time guard (a renderer change and a behaviour change
     of generated apps, its own story).

B. The reason's shape in the schema.
   - One reason per refusal under `not_checked.refusals` (recommended: each reason is true of its
     rule and the page shows it beside it).
   - The branch's single `reason` for the group (keeps the branch's text; three rules, one
     sentence that is true of only two).

C. Where the mutation row's per-format conditions live.
   - One table in `support.rs` that a test checks against the constants (recommended: the row stays
     prose, and a new constant cannot be forgotten).
   - A doc attribute on each constant that the row reads (the constants' doc comments become a
     source of the page; the comments are long and cite issues, so a short one-line attribute is
     needed beside each).
