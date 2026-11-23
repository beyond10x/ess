---
format: aep.planning-md/3
id: story:a-run-with-component-says-what-it-covers-in-text-and-in-json
kind: story
status: draft
title: 'A run with --component says what it covers: the help names the whole scope rule and the JSON output lists the operations no component realises'
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/validate_completeness.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/cli_reference.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/component-unrealised/components.yaml
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/component-unrealised/system.yaml
- confidence: inferred
  path: crates/verify/ess-conformance/tests/mutation_component.rs
- confidence: cited
  path: website/docs/reference/cli.md
revision: 10
---
## Outcome

Whoever runs `ess verify conform synthesize`, `ess specify validate` or `ess verify conform mutate
--emit` with `--component` can read, from `--help` alone, which operations the run covers, and can
read from the JSON or YAML output, without parsing a text line, which operations no component
realises and so no `--component` run reports.

## Need

Found in review of release 0.97.3 (tag `0.97.3`, commit `cd9698fd65`), whose `--component` change
is breaking and is the first thing an adopter of one component reads about. Checked on the
released binary (`ess 0.97.3`, Linux x86_64 archive) and, where a run is shown, on the fixture
`crates/verify/ess-conformance/tests/fixtures/component-unrealised/` at that tag (component
`desk-service` owns `shop.desk` and accepts `shop.desk.Open`; `shop.ledger` is owned and accepted by
no component).

1. **The help does not say the whole scope rule.** The code is
   `handles`/`emits`/`owns_view` in `crates/verify/ess-conformance/src/synthesize.rs:4545-4575`: a
   component handles a command it accepts **or whose domain it owns**, emits an event it publishes
   **or whose domain it owns**, owns a view of a domain it owns, and a binding is its when it
   invokes a command it handles. The help of `ess verify conform synthesize --component` reads
   (`ess verify conform synthesize --help`):

   > Synthesizes only the operations this component realises: the commands it accepts, the views it owns and the events it publishes.

   It leaves out the domain-ownership clause for commands and events, and the bindings. The help of
   `ess specify validate --component` says only "as `ess verify conform synthesize --component` scopes
   it", and that of `ess verify conform mutate --component` says "scope every suite to this declared
   component", with no scope rule at all.
2. **The text run names the unrealised operations, the JSON run does not.**
   `ess verify conform synthesize --path <fixture> --component desk-service` prints:

   > scope: only `desk-service`'s operations are synthesized, so whether the system refuses or notes anything about another component's is not known to this run; `ess verify conform synthesize` without `--component` reports the whole system's
   > unrealised: 2 operation(s) no component realises, so no `--component` run reports their scenarios or refusals; `ess verify conform synthesize` without `--component` does: `command shop.ledger.Close`, `event shop.ledger.Closed`

   The same run with `--format json` writes nothing to standard error and prints the suite on
   standard output: its top-level keys are `provenance` and `scenarios`, `provenance.component` is
   `desk-service`, and neither `scope` nor `unrealised` appears anywhere (`grep -c unrealis` on
   standard output and error: 0). So a script that runs every component in JSON loses the one
   fact the text run had.
3. **The other two reports have no field either.** `ess specify validate --path <fixture>
   --component desk-service --format json` has the keys `commands`, `components`, `domains`,
   `events`, `files_read`, `system`, `unresolved_references`, `valid`, `version` and no
   `completeness` (without `--component`, `completeness` has `counts`, `outside`, `unanswered`,
   `unscenarioed`). `ess verify conform mutate --emit <dir> --component desk-service --format json`
   writes a manifest with the keys `baseline`, `component`, `format`, `mutants`, `spec_digest`,
   `specification`, `unavailable_sites`, `unmutated_sites`. The changelog of 0.97.2 says it for
   validate ("Its JSON and YAML reports have no field for unrealised operations").

The function that finds them exists and is ordered and typed: `pub fn unrealised(ir: &EssIr) ->
Vec<EssSemanticRef>`, from a `BTreeSet`, empty when the model declares no component
(`synthesize.rs:4274`); only `crates/edge/ess-cli/src/main.rs` (line 5426 at the tag) prints it,
as text.

## Design

- Help text. The `--component` doc comments in `crates/edge/ess-cli/src/main.rs` of the three
  commands state one rule, written once and repeated: the commands it accepts or whose domain it
  owns, the events it publishes or whose domain it owns, the views of a domain it owns, and the
  bindings that invoke a command it handles; and that `unrealised` operations (those of no component) are
  reported by none of the `--component` runs. The validate and mutate help name the rule in a
  clause of their own instead of pointing at the synthesize help.
- JSON and YAML. Where the report is `ess specify validate --component <name> --format json|yaml`,
  `completeness` carries an `unrealised` list (the same `EssSemanticRef`s as the text line, in the
  same order, each as `{kind, name}`), present for every `--component` run and empty where the model
  declares no component or every operation is realised. `completeness` is also present for
  `--component` runs whose completeness owes nothing, where today it is absent.
- `ess verify conform synthesize --component --format json` prints the suite
  (`ess-conformance/N`), which is a persisted envelope: the list does not go into it (decision A).
- Whether a new field is a format change by `AGENTS.md` § "Determinism and formats" ("A new format
  version is required when meaning, identity, references, canonicalization, names, or the
  persisted envelope changes. Internal Rust capabilities alone do not require one."):
  - not for `completeness` of `ess specify validate`: the report carries no format identifier,
    and `completeness`, `unscenarioed`'s `caused_by` and `unanswered` entries were each added
    to it without one (`CHANGELOG.md`, entries on `completeness`);
  - yes for the suite `provenance` and for the mutation manifest (`ess-mutation-manifest/N`): both
    are persisted, versioned and byte-pinned (`synthesis_suite_bytes_table`,
    the component-suites table beside it).
  So the choice is not clear-cut across the three commands, and decision A states it.

## Acceptance

- `ess verify conform synthesize --help`, `ess specify validate --help` and
  `ess verify conform mutate --help` each name the whole scope rule: command and event ownership
  by domain, views by domain, bindings by the command they invoke, and that unrealised operations
  are reported by no `--component` run. A test in `ess-cli` reads each help text and fails when
  one of the four clauses is missing from any of the three.
- `ess specify validate --path <fixture> --component desk-service --format json` has
  `completeness.unrealised` equal to `[{"kind":"command","name":"shop.ledger.Close"},
  {"kind":"event","name":"shop.ledger.Closed"}]`, in that order, and `--format yaml` the same;
  without `--component` and for a run where every operation is realised the list is empty or
  absent as decision A says. The case is added to `crates/edge/ess-cli/tests/validate_completeness.rs`
  (the fixture already exists), and its report parses with the field absent for an older reader
  (an old-reader test: the existing report keys keep their values).
- `ess-conformance`'s suites, `ess-mutation-manifest` and every byte pin keep their bytes:
  `synthesis_suite_bytes_table` (which holds the component-suites table too) and
  `synthesis_suite_bytes_table_cli_read` pass unchanged.
- `cargo xtask cli-reference` is run and `website/docs/reference/cli.md` holds the new help
  text; `cargo xtask cli-reference --check` passes, and `task test-xtask` passes.
- `CHANGELOG.md` names the new field and the help change.

## Decisions

A. Where the JSON carries the unrealised operations:
   1. `completeness.unrealised` of `ess specify validate --component` only, an additive field of an
      unversioned report; the suite and the mutation manifest keep their bytes and their formats
      (recommended: no format change, and the one command whose JSON is a report, not a document
      another tool pins);
   2. also a field of the suite `provenance` and of the mutation manifest: a suite format and a
      manifest format change, every committed suite re-pinned, a new `ess-conformance/N`;
   3. a new `--format json` envelope of `ess verify conform synthesize` (`{suite, unrealised}`),
      which breaks every caller that reads the suite from its standard output.

## Out of scope

- Changing which operations a component realises, or the text lines.
- Making the runs of all components cover the whole suite (the 0.97.2 breaking entry says they do
  not).
