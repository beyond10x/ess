---
format: aep.planning-md/3
id: story:feature-request-448
kind: story
status: active
title: specify validate stops at the first parse-level refusal of a file
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#448
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/edge/ess-cli/tests/fixtures/parse-refusals/parse-plus-semantic.yaml
- confidence: inferred
  path: crates/edge/ess-cli/tests/fixtures/parse-refusals/two-unparsable-compact.yaml
- confidence: inferred
  path: crates/edge/ess-cli/tests/fixtures/parse-refusals/two-unparsable-filters.yaml
- confidence: inferred
  path: crates/edge/ess-cli/tests/validate_parse_refusals.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/binding/condition.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/related_guard.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/command/row_set.rs
- confidence: cited
  path: crates/specify/ess-domain/src/entity.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/selection.rs
- confidence: cited
  path: crates/specify/ess-domain/src/view.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/predicate.rs
- confidence: cited
  path: website/docs/reference/diagnostics.md
revision: 13
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T14:53:29Z", actor: "human:timo", revision: 12, decided_on: {"recorded":{"review_outcome":4}}}
- {from: "proposed", to: "active", at: "2026-10-05T14:53:29Z", actor: "human:timo", revision: 13, decided_on: {"recorded":{"review_outcome":4}}}
---
## Outcome
Resolve beyond10x/ess#448: specify validate stops at the first parse-level refusal of a file.

## Origin
beyond10x/ess#448, filed 2026-10-05; an adopter tracking known refusals per issue saw only the first parse-level refusal of a file (0.32.1) and now writes one view per file to work around it.

## Fit review
1. Need: one file with N predicate positions that do not parse must report all N, each with its declaration and location, as semantic refusals already do. No authored syntax is involved; the request is about diagnostics. Reproduced on the installed ess 0.52.0 (`<fit-review scratch>/probe-448/run.sh`), one entity and two views in one file:
   - `b`: filters `{label: {matches: "a"}}` and `{label: {suffix: "b"}}` print one line, `system.yaml: cannot parse predicate "label: {matches: …}": unknown operator "matches"; …`. The second view is not mentioned. There is no declaration path, no line and no `ESS-*` code.
   - `c`: compact `label ~= "a"` and `label =~ "b"` give one refusal, the same way.
   - `e`: one unparsable filter plus one `nosuch == "b"` filter. Only the parse refusal is printed, and the semantic one is hidden too.
   - `a`: `starts_with` / `ends_with` under `format: ess/7` gives two `[unsupported_format_version] view.….filter` refusals, each with `ESS-VIEW-009` and `system.yaml:41:5` / `:47:5`. `d`: two unobservable facts give two refusals. So the issue's original operators now parse, and only a real parse failure stops validation.
   Requester's proposal (theirs): "Collect parse-level refusals per declaration and report them all, or document the stop rule".
2. Class: gap in diagnostics, with no authored surface. Cause: `Predicate` is parsed inside serde (`crates/specify/ess-primitives/src/predicate.rs:3901-3906`, `from_node(..).map_err(serde::de::Error::custom)`), so the first predicate error aborts deserialization of the whole document. Raw declarations hold `Predicate` directly: `RawView.filter` (`crates/specify/ess-domain/src/view.rs:1789`), `RawOutcome.when` (`crates/specify/ess-domain/src/command.rs:5461`), and 28 `Predicate`-typed fields in ess-domain (`grep -c`). The stop rule is not documented. `website/docs/reference/diagnostics.md:50` files "a predicate refused while it is read" under `SPEC` but does not say that it ends validation of the file.
3. Existing idiom: none that reports all of them. The workaround is one view per file. Precedent for the repair: aggregate `where:` already reads a `Node` and calls `Predicate::from_node` itself (`view.rs:586-596`), and `Invariant::from_node` does the same (`crates/specify/ess-domain/src/entity.rs:643-644`).
4. Fit: raw predicate positions hold `ess_primitives::node::Node`. The existing per-declaration validation pass, which already accumulates `unsupported_format_version` and `unobservable_fact` with paths and spans (probe `a`, `d`), calls `Predicate::from_node` and records each failure. The refusal keeps today's sentence and gains the declaration path (`view.<name>.filter`, `command.<name>.outcomes.<o>.when`, …), a location, and the family code for "refused while it is read" (`diagnostics.md:50`). A declaration whose predicate failed is withheld from checks that read the predicate, so it does not cascade into misleading secondary refusals. Every predicate position moves together: filters, `when`, `when_subject.predicate`, `when_related`, invariants, binding `where`, aggregate `where`. This touches no target, no generated artifact and no format, because only the loader changes. Structural serde refusals (missing field, unknown key, wrong YAML shape) still stop the document, and that rule is written down beside the refusal format.
5. Second adopter: someone retrofitting an existing service drafts every guard of a twenty-command domain in one file, misspells two operators, and should get both refusals in one validate run.
6. Cost: no `ess/23`, no `ess-conformance/N` bump, no new keyword. A diagnostic-code consequence: a predicate refused while the document is read has no construct and is `SPEC` (`crates/specify/ess-compiler/src/resolve.rs:519-525`); once it is parsed per declaration it has one, so `ESS-SPEC-017` (null comparison) becomes e.g. `ESS-VIEW-017`, and an unparsable operator gets a construct-family `009`/`012` code. Decided (coordinator, 2026-10-05): the codes stay `SPEC`, so gated refusal lists keep matching. A source-API change in ess-domain: public `Raw*` fields change from `Predicate` to `Node`, which breaks Rust callers that build raw declarations by hand (inferred; I don't know of any outside this workspace). Stderr wording changes. Canonical IR bytes do not change, because parsing still produces the same `Predicate`.
7. Alternatives: document the stop rule only. Cheapest, but it leaves the one-view-per-file workaround and the missing location, so refused. On a serde failure, re-walk the YAML `Value` for predicate keys: refused, because it duplicates where predicates live and drifts. Collect every serde structural error with a non-serde loader: refused, too large for the need. Chosen: move predicate parsing out of serde into the accumulating pass, and document the structural stop rule.

## Decisions
accept, redesigned. Predicate positions deserialize as `Node` and are parsed in the per-declaration pass. Every unparsable predicate in a file is reported with its declaration path, line and code, and the file's other semantic refusals are still reported. Structural YAML/serde refusals still stop the file, and `website/docs/reference/diagnostics.md` says so. Refusal codes stay in their current `ESS-SPEC-*` family when a predicate error moves into the per-declaration pass (coordinator decision 2026-10-05). The declaration path goes into the message, not the code, so adopters' lists of known refusal codes keep matching. No format bump. Overlaps: #452 shares `TryFrom<RawOutcome> for Outcome` in `command.rs` and runs concurrently in wave 2 on disjoint lines, with a coordinator `git merge-tree` dry run before the second merge; #437, #445, #450 and #459 depend on this story (edges recorded) and land after it.

## Acceptance
- two_unparsable_view_filters_in_one_file_report_both: the committed fixture `crates/edge/ess-cli/tests/fixtures/parse-refusals/two-unparsable-filters.yaml` (fit-review case `b`: filters `{label: {matches: "a"}}` and `{label: {suffix: "b"}}`) prints two refusals naming `view.probe.item.FirstView.filter` and `…SecondView.filter`, each with a line.
- unparsable_compact_predicates_report_each_declaration: the committed fixture `crates/edge/ess-cli/tests/fixtures/parse-refusals/two-unparsable-compact.yaml` (case `c`: `label ~= "a"` and `label =~ "b"`) gives two refusals, one per view.
- parse_refusal_does_not_hide_semantic_refusals: the committed fixture `crates/edge/ess-cli/tests/fixtures/parse-refusals/parse-plus-semantic.yaml` (case `e`: `{label: {matches: "a"}}` and `nosuch == "b"`) prints the parse refusal and the `unobservable_fact` refusal.
- every_predicate_position_collects: one file with one bad predicate in each of filter, `when`, `when_subject.predicate`, `when_related`, invariant and binding `where` reports one refusal each.
- failed_predicate_does_not_cascade: a declaration whose predicate failed produces no secondary refusal about that predicate.
- predicate_refusal_codes_unchanged_after_per_declaration_parse: a null comparison (`note == null`) in a view filter, an outcome `when` and an invariant is refused `ESS-SPEC-017` at each site, as today. An unknown operator keeps the `ESS-SPEC-*` code it has today. No predicate parse refusal moves to an `ESS-VIEW-*`, `ESS-COMMAND-*` or `ESS-ENTITY-*` code.
- structural_refusal_still_stops_and_is_documented: a missing required key still yields one refusal. `diagnostics.md` states the rule beside the `SPEC` row (:50), and a case in `crates/edge/ess-cli/tests/validate_parse_refusals.rs` reads that row and fails if the sentence naming the structural stop is missing.
- canonical_ir_unchanged_for_valid_specifications: `examples/billing` compile bytes unchanged (`cargo xtask site-data --check`).

## Scope
- crates/specify/ess-primitives/src/predicate.rs  cited — serde `Deserialize` that aborts the document (3901-3906)
- crates/specify/ess-domain/src/view.rs  cited — `RawViewSpec.filter` (1789) becomes `Node`; parsed in `TryFrom<RawViewSpec> for ViewSpec` (1807-1887); the aggregate `where` reader in `RawAggregate`'s `Deserialize` (563-651, `Predicate::from_node` at 594) is the precedent, unchanged
- crates/specify/ess-domain/src/command.rs  cited — `RawOutcome.when` (field at 5461, struct 5456-5637) and `RawSubjectFact`'s `Deserialize` (5409-5439) hold `Node`; parsed in `TryFrom<RawOutcome> for Outcome` (5858-6070), at the `raw.when` / `outcome_condition` call (5900-5946), through `outcome_condition` (5695-5784). #452 edits the same `TryFrom` at other lines (`set_effects::affects` 5878, `affects_beside` 6008): the two run concurrently in wave 2 on disjoint line ranges, and the coordinator dry-runs `git merge-tree` before the second of them merges. #459 edits 6008 after this story (edge recorded). Other `command.rs` stories touch `subject_of` (6161-6271), `scalar_representation`/`literal_representation` (4389-4525; #445 and #450, both after this story by edge) and the input-coverage site (3333-3340)
- crates/specify/ess-domain/src/entity.rs  cited — `RawInvariant`'s `Deserialize` (687-692) keeps the `Node`; `Invariant::from_node` (643-655) is called by the per-declaration pass. Disjoint from `validate_relations` (1368), which #437 edits
- crates/specify/ess-domain/src/binding/condition.rs  inferred — binding `where` predicate
- crates/specify/ess-domain/src/command/related_guard.rs  cited — `RawRelatedGuard` (93-121): `filter`, `predicate` and `forall` hold `Node`; #437 adds an advisory function beside `validate` (891-973) after this story (edge recorded)
- crates/specify/ess-domain/src/command/row_set.rs  inferred — row-set predicate
- crates/specify/ess-domain/src/selection.rs  inferred — selection `where`
- crates/specify/ess-compiler/src/resolve.rs  cited — `family_of_kind` (1025-1055) and `family_of` (1095-1116) pick a code's family from its site; a predicate parse refusal keeps `family::SPEC` (as `NULL_COMPARISON`, 519-525) although it now has a construct site
- website/docs/reference/diagnostics.md  cited — stop rule and `SPEC` family row
- crates/edge/ess-cli/tests/validate_parse_refusals.rs  inferred — new CLI scenarios above
- crates/edge/ess-cli/tests/fixtures/parse-refusals/two-unparsable-filters.yaml  inferred — fit-review case `b`, committed
- crates/edge/ess-cli/tests/fixtures/parse-refusals/two-unparsable-compact.yaml  inferred — fit-review case `c`, committed
- crates/edge/ess-cli/tests/fixtures/parse-refusals/parse-plus-semantic.yaml  inferred — fit-review case `e`, committed
