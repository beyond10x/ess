---
format: aep.planning-md/3
id: story:a-scenario-with-a-repeated-key-is-refused
kind: story
status: implemented
title: A scenario document that repeats a mapping key is refused, as a model file is
summary: ess-scenario/1 silently takes the last of two equal keys; model YAML refuses the same repetition.
tags:
- downstream
- ess-issues-3
relations:
- informed_by: story:ess-yaml-owns-its-parser-and-emitter
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/verify/ess-conformance/assets/coverage-admission.js
- confidence: cited
  path: crates/verify/ess-conformance/src/authored.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/runtime.ts
- confidence: cited
  path: crates/verify/ess-conformance/tests/authored_outcome_guards.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/coverage_refusal_catalogue.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/diagnostic_catalogue.rs
- confidence: inferred
  path: website/docs/guides/verify/author-scenarios.md
- confidence: cited
  path: website/docs/reference/diagnostics.md
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-10-29T17:06:38Z", actor: "human:timo", revision: 9, executor: "agent:ess-issues-lead", correlation: "ess-issues-3"}
- {from: "proposed", to: "active", at: "2026-10-29T17:06:38Z", actor: "human:timo", revision: 10, executor: "agent:ess-issues-lead", correlation: "ess-issues-3"}
- {from: "active", to: "implemented", at: "2026-11-01T01:28:12Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"test_result":1}}}
---
## Problem

An authored scenario (`ess-scenario/1`) that repeats a mapping key is read without complaint, and
the last value wins. A model file that repeats a key is refused ("duplicate entry with key
`version`"), so the two readers of one repository's YAML disagree. Reported 2026-10-29 by an
adopter on ESS 0.75.0: a step's `input:` named `tenant` twice.

Checked on the 0.77.0 head (84671d6, `ess 0.77.0`) against the adopter's model at its then-current commit:

- With the key repeated with the same value, `ess verify conform synthesize` exits 0: "183
  scenario(s) (34 authored), 0 refusal(s)". `ess specify validate` exits 0 and reports the
  scenarios valid.
- With the second value changed to `tenant-other`, synthesis still exits 0, and the compiled step
  sends `"tenant":{"kind":"literal","value":"tenant-other"}`. The first value is dropped without
  a word.
- The same repetition in `system.yaml` is refused by validate.

## Acceptance

The tests use a neutral `demo.*` model with an `Integer` and a `String` input: one command, one
view.

**Detector and refusal form.** New file `crates/verify/ess-conformance/tests/authored_repeated_key.rs`:

- `a_key_repeated_with_another_value_in_a_step_input_is_refused`: one refusal, `ESS-AUTHOR-047`,
  whose message names the origin, `timeline[0].input`, the key `"label"` and `at line`. No
  scenario is compiled for the file.
- `a_key_repeated_with_the_same_value_is_refused`: the same refusal; an equal value is no excuse.
- `a_key_repeated_in_a_nested_assertion_mapping_is_refused`: a repeat inside
  `assert[0].contains.<field>` is refused, with that path.
- `equal_keys_in_separate_mappings_stay_valid`: `label` in two steps' inputs and `$instance` in two
  sibling mappings compile without a refusal.
- `a_repeated_document_key_keeps_its_unreadable_refusal`: `summary:` twice is still
  `ESS-AUTHOR-001` "duplicate field `summary`", with no 047 beside it.

Run: `cargo test -p ess-conformance --locked --test authored_repeated_key`

New file `crates/edge/ess-cli/tests/authored_repeated_key.rs`:

- `validate_and_synthesize_refuse_a_repeated_key_alike`: `ess specify validate` on a directory
  whose `ess-inputs.yaml` lists the file, and `ess verify conform synthesize --scenarios`, both
  exit 1 with `refusal[ESS-AUTHOR-047]`. Synthesize writes no suite.

Run: `cargo test -p ess-cli --locked --test authored_repeated_key`

**Warning form, only if the replay refuses an adopter.** It replaces the two refusal tests above:

- `crates/edge/ess-cli/tests/authored_repeated_key.rs`,
  `validate_and_synthesize_warn_of_a_repeated_key_and_keep_the_last_value`: both commands exit 0
  and print `warning[ESS-AUTHOR-047]: <file>` with the same message on standard error. The suite
  is byte-identical to the one the same input gives without the warning (last value kept).
- `crates/verify/ess-conformance/tests/authored_repeated_key.rs`,
  `a_repeated_key_is_reported_and_the_scenario_still_compiles`: `compile` returns the scenario and
  no refusal, and the public detector beside it reports one `ESS-AUTHOR-047` for the file.

**The code, in either form:**

- `cargo test -p ess-conformance --locked --test diagnostic_catalogue author_codes_run_from_001_to_047`
  (renamed from `author_codes_run_from_001_to_046`).
- `cargo test -p ess-conformance --locked --test authored_outcome_guards every_coverage_reader_admits_the_refusal_and_no_code_after_it`
  (newest becomes 047, unknown becomes 048).
- `cargo test -p ess-conformance --locked --test coverage_refusal_catalogue`: the Rust, Go,
  TypeScript and browser readers each admit 047 and refuse 048.
- `cargo xtask diagnostics --check`: `website/docs/reference/diagnostics.md` lists 047.

**Must stay green:** `cargo test -p ess-conformance --locked --test authored`, `--test
authored_foreign_type`, `--test authored_caller` and `--test authored_structured_instances`;
`cargo test -p ess-cli --locked --test authored_scenarios` and `--test
authored_scenarios_adversary`. No committed `ess-scenario` file, example or corpus scenario is
newly refused: a preview found none.

## Fit review

1. Need: one repository's YAML is read one way. An authored scenario that repeats a mapping key
   keeps the last value without a word, while a model file with the same repeat is refused.
   Reproduced on 0.77.0 with a minimal `demo.board` model (a repro outside the repository): step
   input `{label: morning, limit: 3, page: 1, label: evening}` validates ("1 scenario(s), valid",
   exit 0) and synthesizes ("2 scenario(s) (1 authored), 0 refusal(s)", exit 0), and the step sends
   `"label": {"kind": "literal", "value": "evening"}`. The same value repeated, and a repeat inside
   `assert[0].contains.board_id`, both give a suite byte-identical to the one without the repeat.
   The same repeat in `system.yaml` is refused: `system.yaml: entities[0].fields[0]: duplicate
   entry with key "type" at line 12 column 9`. A repeated top-level `summary:` in a scenario is
   already refused, as `ESS-AUTHOR-001` "unreadable: duplicate field `summary`". The adopter's
   proposal is the story's draft acceptance: refuse at any depth and name both positions.
2. Class: defect. `ess-yaml` documents that its behaviour on a repeated key depends on the target
   (`crates/specify/ess-yaml/src/lib.rs:8-9`). Per `docs/design/yaml-parser-replacement.md:59-72`,
   a derived struct refuses the repeat, a `BTreeMap` silently keeps the last value, and
   `ess_yaml::Value` refuses the repeat with the key. The scenario `Document`
   (`crates/verify/ess-conformance/src/authored.rs:373`) holds `caller`, `input`, `fields`,
   `payload`, `params`, `contains`, `excludes` and `response` as `BTreeMap`s (`authored.rs:423-675`)
   and reads them straight from the text (`read_document`, `authored.rs:2874-2898`), so those maps
   keep the last value. The model reader reads through `ess_yaml::Value` first specifically to
   refuse this (`crates/specify/ess-domain/src/spec.rs:175-195`).
3. Already expressible: not applicable. Nothing to express; the reader is wrong.
4. Fit: no authored surface and no format. The check to share is the model reader's: read the text
   as `ess_yaml::Value`, whose mapping visitor refuses a repeat at any depth, naming the path, the key
   and the line and column of the mapping. Run with 0.77.0 against the scenario file above, it
   prints `timeline[0].input: duplicate entry with key "label" at line 12 column 12`.
   `header_type` (`authored.rs:2913-2931`) already does this read and discards the error. Positions
   of both keys are not available: the backend reports one position per error, the mapping's, and
   no span per key (`yaml-parser-replacement.md:38-49`). Spans come with
   story:ess-yaml-owns-its-parser-and-emitter.
5. Second adopter: anyone who copies a step and edits its `input:`, or merges two branches that
   each added a field to one assertion.
6. Cost: one diagnostic code. The ESS-AUTHOR catalogue ends at 046 (`authored.rs:2380`;
   `tests/diagnostic_catalogue.rs:43-46` pins `author_codes_run_from_001_to_046`). 047 is the next
   free code: `tests/authored_outcome_guards.rs:611-618` uses it as "a code no cause numbers" and
   moves to 048. Four coverage readers restate the newest code and move with it: Rust through the
   catalogue (`names_file_refusal`, `authored.rs:1402-1407`), Go (`src/go/runtime.go:62`),
   TypeScript (`src/ts/runtime.ts:918`) and the browser (`assets/coverage-admission.js:472`).
   `website/docs/reference/diagnostics.md` is rendered again. This refuses input that 0.77.0
   accepted, so D214 applies: a refusal only if the batch's adopter replay refuses no adopter.
   Preview, not the replay: used as a detector, the 0.77.0 model reader finds no repeated key in
   any of the 28 committed `ess-scenario` files, the 143 scenario files of the batch-2 replay
   corpus, or the reporting adopter's 35 files at its later head (a repro outside the
   repository).
7. Considered:
   - Change nothing. Rejected: it is a defect, and a silently dropped input value is a false check.
   - Read through `Value` first and then `from_value`, as `spec.rs` does. Rejected:
     `from_value` has no text positions, so every other unreadable refusal of a scenario would lose
     its ` at line L column C` (inferred from `yaml-parser-replacement.md:46-49`). That changes
     messages that are fine today.
   - Keep the typed read and its refusals as they are, then do the `Value` read. A failure there,
     after the typed read succeeded, is a repeated map key. Chosen.
   - File the repeat under `ESS-AUTHOR-001`. That would need no new code and no reader bumps, but
     001 would then mean two things. Rejected because the batch asks for a code of its own.

## Decisions

Accept, redesigned. Three changes from the draft acceptance.

- **What the refusal names.** It names the file, the path of the mapping, the key, and the line
  and column the YAML reader gives for that mapping. This is the wording a model file's repeat
  already gets, for example `timeline[0].input: duplicate entry with key "label" at line 12
  column 12`. It does not name both key positions: the backend has no span per key
  (`docs/design/yaml-parser-replacement.md:38-49`). That half waits for
  story:ess-yaml-owns-its-parser-and-emitter.
- **Where it applies.** The repeats the typed read already refuses keep their refusal and their
  code: a repeated document key or step key (a derived struct field) stays `ESS-AUTHOR-001`
  "duplicate field". The new check covers what is silent today, a repeated key in any map the
  document reads as a map (`input`, `caller`, `fields`, `payload`, `params`, `contains`, `excludes`,
  `response`, and any mapping inside a literal), at any depth.
- **Code.** The new code is `ESS-AUTHOR-047`, "A mapping in the document repeats a key." Repair:
  "write each key once; YAML keeps only one of two equal keys, and which one is not the author's
  choice". It is allocated in either form below.

**Form (D214, as D186 ruled for U10):**

- Refusal form, the default. It ships if the batch-3 adopter replay refuses no adopter scenario
  with `ESS-AUTHOR-047`. The file produces no scenario, and validate and synthesize exit 1.
- Warning form, the fallback, if the replay refuses one. Validate and synthesize print
  `warning[ESS-AUTHOR-047]: <file>` with the same message and help on standard error. The exit
  status and the suite bytes stay those of 0.77.0, and the last value is kept. The refusal then
  waits until that adopter is fixed.
- Both forms share one detector, so switching is a call-site change in `compile_one` and the CLI.

No source format, ADR or operator choice is needed. Nothing is split out.

## Scope

**Primary surface:** the authored scenario reader in `ess-conformance`. Size S to M: the detector
is a few lines, and the code allocation touches four readers.

- `crates/verify/ess-conformance/src/authored.rs`:
  - `read_document` (2874-2898) runs the `ess_yaml::Value` read after the typed read succeeds. A
    failure there becomes a new `Cause::RepeatedKey { detail }`.
  - `header_type` (2913-2931) may share that read.
  - Add the catalogue entry `=> 47` after `NotIssuedClaims => 46` (2380).
  - Warning form only: a public detector beside `compile` (2698).
- `crates/verify/ess-conformance/src/go/runtime.go:62`, `src/ts/runtime.ts:918` and
  `assets/coverage-admission.js:472`: the newest authored code becomes 47.
- `crates/edge/ess-cli/src/main.rs`, warning form only: `validate` (around 2685) and
  `compiled_authored` (around 4749) print the warning.
- Tests: new `crates/verify/ess-conformance/tests/authored_repeated_key.rs` and
  `crates/edge/ess-cli/tests/authored_repeated_key.rs`. Edits to
  `crates/verify/ess-conformance/tests/diagnostic_catalogue.rs` and `authored_outcome_guards.rs`.
- Docs: `website/docs/reference/diagnostics.md` (generated) and one sentence in
  `website/docs/guides/verify/author-scenarios.md`. A `CHANGELOG.md` line, which says which form
  shipped.
- Not touched: `crates/specify/ess-yaml` (its `Value` read is reused as it is) and the model reader.

**Collisions with the other batch-3 units:**

- U11, U13 and U14 change synthesis (`synthesize.rs`, `synthesize/related_guard.rs`, arrangement).
  This unit changes no synthesis file.
- If one of them adds an `ESS-SYNTH` code, it edits `newestGeneratedRefusal` and
  `NEWEST_GENERATED_REFUSAL`, the line beside this unit's (`runtime.go:61-62`,
  `runtime.ts:917-918`), and `tests/coverage_refusal_catalogue.rs`. That is a textual merge,
  resolved at integration.
- Within U15 there is no file shared with the integer story.
