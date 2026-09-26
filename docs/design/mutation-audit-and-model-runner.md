# Mutation audit and a model-based sequence runner

Status: proposed (beyond10x/ess#114, wave 2 design unit E10d). No source, IR or format change is
made by this page. Two implementation units build what it says, one per part. They do not depend
on each other and can land in either order. Every claim about current code cites the integration
base this page was written against (`4ffcb537c`). A line marked *inferred* was not measured, and
the implementation unit confirms it before building on it (the last section lists every one).

## Behavior and authority

The `testing-conformance` guidance says a green suite means something only once a broken
implementation has turned it red. Today every adopter builds that check by hand. The issue proposes
shipping two versions of it, both driven by the canonical IR:

1. **A mutation audit.** Derive mutants from the specification. Replay each mutant's suite against
   an **unchanged** target. Report every mutant no scenario kills, and the scenarios that killed the
   others. The issue's own hand-built run had 35 mutants and 8 survivors.
2. **A model-based explorer.** A seeded random walk over the commands, checked after every step
   against a reference model interpreted from the IR, with shrinking. The issue's four engine
   mutants passed all 127 generated and authored scenarios, and each one was caught by 300 sequences
   of 80 steps.

The wave brief settled the following, and this page does not reopen them:

- Part 1 is `ess verify conform mutate`. It mutates the **specification**, not the implementation,
  and uses the issue's mutant classes.
- Part 2 is emitted by `--target typescript` and `--target go`. It uses the existing `Target`
  interface, takes a seed and a step count, shrinks, and **fails hard when a declared outcome is
  never reached**.
- Each part's first cut is limited to what one implementation unit can finish. Everything else is
  listed under "Deferred".

### What a surviving specification mutant means

Mutating the specification is the dual of mutating the implementation, and the two directions do
not measure the same thing. This page follows the brief's direction and records what that
direction can and cannot show.

- **Kill condition.** The suite synthesized from mutant specification *S′* fails against the
  target that implements *S*. The mutant is killed only if the synthesized suite asks a question
  whose answer differs between *S* and *S′*, and the target gives the *S* answer.
- **A weakening mutant can never be killed.** Dropping a `sets` entry says the field is the
  implementation's to choose (`crates/specify/ess-compiler/src/ir.rs:813-823`: "Empty is the common
  case and a statement, not a gap"). The mutant's suite then asserts nothing about the field, and a
  correct target satisfies a weaker specification by definition. Such a mutant tells a reader
  nothing, so no weakening class is generated. Each issue class is replaced by the altering version
  of the same fault (see "The classes").
- **Authored scenarios cannot kill a specification mutant.** An authored scenario's expectations are
  written by its author, not derived from the model. `act` (`crates/verify/ess-conformance/src/
  authored.rs:1723-1797`) resolves the names a step uses, and then it pushes the author's own
  outcome, error and event claims. The only thing it reads from the IR is an event's payload
  *shape* (`authored.rs:1952-1973`), and no mutant class changes an event declaration. An authored
  scenario therefore runs identically in the baseline suite and in every mutant's suite, and it
  passes in both. **Decision:** `mutate` takes no `--scenarios` option. Running authored scenarios
  once per mutant would cost time and could never change a verdict.
- **So a survivor is a finding about what synthesis asks of the model.** Either the mutant is
  equivalent (no input the witnesses can produce tells *S* from *S′*), or synthesis derives no
  scenario that observes the rule. The fix is to declare what makes the rule observable, or to file
  a synthesis gap. The fix is **not** to author a scenario. The page that renders the verb says this
  in those words.
- **The direction where authored scenarios do count** is the unchanged suite (generated and
  authored) run against a *mutant implementation*. From a specification mutant, that needs an
  executing interpreter. `crates/verify/ess-conformance/src/interpret.rs:1-50` is a seam that
  derives no behaviour, and its `begin_scenario` refuses (`interpret.rs:95`). That direction is
  deferred (see "Deferred"). Part 2's reference model is the interpreter it would need, written in
  TypeScript and Go rather than Rust.

## What exists today

| Construct | Where | What it gives this design |
|---|---|---|
| `ess verify conform run --target billing\|oracle-fixture\|interpreted` | `crates/edge/ess-cli/src/main.rs:529-563`, dispatch `:2799-2856`, `ReferenceTarget` `:621-629` | The only in-process targets. Part 1 runs against exactly these. |
| Fresh ordinary suite: synthesize, then compile authored scenarios, then `select_fresh_format` | `main.rs:2858-2877` (`fresh_legacy_run_suite`) | The suite shape Part 1 builds for the baseline and for every mutant, without the authored half. |
| Exit mapping of a run: `Passed` 0, `Failed` 1, `Error` 3 | `main.rs:2915-2920` | Part 1 reuses 0, 1 and 3 with the meanings below. |
| Parse, then assemble, then compile | `crates/edge/ess-cli/src/load.rs:27-80` | `RawSpecFile` values exist only inside this function today. Part 1 needs them back. |
| `RawSpecFile`, `RawCommandSpec`, `RawOutcome`, `RawViewSpec`, `Transition`: all fields `pub`, all `Clone` | `crates/specify/ess-domain/src/spec.rs:45-110`; `command.rs:2876-2897`, `:2995-3108`; `view.rs:1348-1378`; `entity.rs:160-172`, `:1452-1463` | Where mutants are applied. |
| `Specification` and `EssIr`: fields private; `EssIr` built only from `pub(crate)` parts | `spec.rs:116-139`; `crates/specify/ess-compiler/src/ir.rs:1719-1753`, `:1775-1778` | Why mutants are not applied there. |
| `compile` runs validation before resolution | `crates/specify/ess-compiler/src/resolve.rs:970-996` | A mutant the model does not admit is refused by the same checks an author meets. |
| Absent-event assertions: every declared event a branch does not emit is asserted absent | `crates/verify/ess-conformance/src/synthesize.rs:1339`, `:1377-1383`, `not_emitted` `:3050` | Why dropping an emitted event is an *altering* mutant. |
| Per-state refusal scenarios for a `wrong_state` branch | `synthesize.rs:1566-1571` (`is_state_refusal`), `:1654-1700` | Why dropping a `from` state is killable only where the command declares `wrong_state`. |
| Witness rule 3: guard literals tried "one either side" | `crates/verify/ess-conformance/src/witness.rs:32-35` | Why moving a comparison boundary is expected to be killed. |
| Fault matrix: one fault per implementation, and which named scenario catches it | `crates/verify/ess-conformance/src/faulty.rs:1-60`, `Fault` `:166`, `Faulty` `:318`; `tests/faults.rs` | The shape of Part 1's evidence: pinned survivors with reasons, as `Caught::Nothing` is pinned there. |
| Stable codes: `Code { family, number }` rendered `ESS-<FAMILY>-<NNN>` | `crates/specify/ess-compiler/src/diagnostic.rs:34-58`; families `SYNTH` (`synthesize.rs:519`), `AUTHOR` (`authored.rs:1079`) | Part 1 adds the family `MUTATE`. |
| Emitted TS package: `emit(suite)`, `emit_input(input)`, constant sources, `INDEX_TS` | `crates/verify/ess-conformance/src/ts/mod.rs:67-94`, `:110-129`, `:164-172`; `tsconfig` includes `src/**/*.ts` (`:221-224`) | Part 2 adds files through new functions, not by changing these two. |
| Emitted Go package: `emit(suite)`, `emit_input`, `go:embed suite.json` | `crates/verify/ess-conformance/src/go/mod.rs:49-63`, `:66-90`, `:95-105` | The same. |
| Callers of `ts::emit`/`go::emit`: the CLI at `main.rs:2944`, `:2951` and `crates/edge/ess-cli/src/coverage.rs:65`, `:75`, plus about 25 test sites | measured with `grep -rnE '(ts\|go)::emit(_input)?\('` | Why the explorer is added by new `*_with_model` functions rather than a changed signature. |
| TS runtime pieces the explorer reuses: `Target` `runtime.ts:1760`, `isUnsupported` `:171`, `strictJSON` `:530`, `goMarshal` `:599`, `Harness`/`newHarness` `:2077-2101` (8 attempts, `:2080`), `equal` `:3591`, `ranked` `:3975`; `fromNode` `predicate.ts:363`, `facts`/`bindFact` `:85`/`:93` | `crates/verify/ess-conformance/src/ts/` | No second evaluator, comparator or ordering rule. |
| Go runtime equivalents: `Target` `runtime.go:1068`, `ErrUnsupported` `:1128`, `NewHarness` `:1377-1379`, `equal` `:2716`, `ranked` `:3059`, `strictJSON` `:3183`; `fromNode` `predicate.go:186`, `facts`/`bindFact` `:71`/`:79` | `crates/verify/ess-conformance/src/go/` | The same. |
| TS `runWith(t, newTarget, suiteJSON)` runs any suite document; Go has only `Run` over the embedded suite | `runtime.ts:2320`; `runtime.go:1556` | Why replaying mutant suites in an adopter's language is deferred to its own unit. |
| `spec_digest` = SHA-256 over the **compact** IR bytes | `ir.rs:2070-2082` (`serde_json::to_vec`); `crates/generate/ess-gen/src/provenance.rs:58`, `:154-156`; `crates/verify/ess-conformance/src/scenario.rs:361` | Binds the emitted `ir.json` to `suite.json`. |
| Compiled `EssIr` is unversioned and Serialize-only, with no reader | `website/docs/reference/formats.md:109` | The explorer reads it only as bytes emitted by the same binary, and checks the digest. |
| View consistency is `read_your_writes` or `eventual`, and the default is `eventual` | `crates/specify/ess-domain/src/view.rs:71-83` | The explorer cannot exclude eventual views; it polls them. |
| Guard overlap is refused only where the input domain is finite | `crates/specify/ess-domain/src/command.rs:1985-2035` | Two `when` guards can both hold over an integer. The model must not choose between them. |
| An earlier draft `explore.ts` against `a1cf7233fe`: typechecked, never run, not wired in | issue #114, first comment | Part 2 starts from it. Five of its choices are overruled below, each with the reason. |

## Part 1 — `ess verify conform mutate`

### Surface

```console
ess verify conform mutate --path SPEC --target billing|oracle-fixture|interpreted \
    [--class CLASS]... [--report-out FILE] [--format text|json|yaml]
```

- `--path` defaults to `.`, as it does for `run` (`main.rs:531`). `--target` is the existing
  `ReferenceTarget`, with no new value.
- `--class` repeats and takes the kebab names below. When it is absent, every class runs. It is a
  clap `ValueEnum`, so a misspelled class is a usage error (exit 2, clap's own).
- `--report-out` writes the `ess-mutation-report/1` document (below). `--format json` prints the
  same bytes. `yaml` renders them through the existing `render`.
- There is no `--suite`, `--suite-input`, `--suite-format` or `--scenarios`. The audit synthesizes
  a fresh ordinary suite for the baseline and for every mutant. A committed suite would have to be
  mutated as a suite rather than as a specification. Coverage suites (`/5` and above) carry parent
  chains and inventories that a mutant would have to re-derive, and the audit needs neither. The
  reason there is no `--scenarios` is given above.

**Exit status:**

| Exit | When |
|---|---|
| 0 | The baseline passed, at least one mutant ran, and every mutant that ran was killed. |
| 1 | The specification did not load (the existing `resolved` path, `main.rs:2822-2824`), **or** at least one mutant survived. |
| 3 | `ESS-MUTATE-001` (the baseline did not pass) or `ESS-MUTATE-003` (no site), **or** no mutant survived and at least one was inconclusive. |

A stillborn mutant does not change the exit status. It says something about the mutation operator
and validation, not about the suite, and the report lists it.

### Where a mutant is applied: the parsed documents

A mutant is a changed copy of the `Vec<(Source, RawSpecFile)>` that `load.rs` parses. It goes
through `Specification::assemble` and `ess_compiler::compile` again, exactly as the original did.

- **Not the IR.** `EssIr`'s fields are private and it is built only from `pub(crate)` parts
  (`ir.rs:1719-1778`). Changing that would mean an IR mutation API in the compiler. Such an API
  could produce IR that no validated source can produce, and the compiler's "total lookup"
  handles (AGENTS.md, Determinism) would stop being a guarantee.
- **Not `Specification`.** Its fields are private as well (`spec.rs:116-139`).
- **The raw documents give validation for free.** A mutant the model does not admit (for example,
  negated guards that overlap over a finite domain, `command.rs:2027`) is refused by the check an
  author would meet. It is recorded as *stillborn* with that check's own code, so no hand-written
  admissibility rule sits beside the validator.
- **CLI change.** `load.rs` gains `pub(crate) fn raw_specification(path) -> Result<RawLoaded>`,
  which returns the parsed `(Source, RawSpecFile)` list, the `SourceMap` and `files_read`, or the
  parse problems. `specification()` is rewritten to call it, so there is one parse path.
  `RawSpecFile` is `Clone` (`spec.rs:43`), so a mutant is a clone with one edit.

### The classes

Nine classes form a closed enum `MutantClass` with an `ALL` constant, which is the shape `Fault`
has (`faulty.rs:166`). A mutant's id is `<class>/<site>`. Mutants are enumerated and run in byte
order of id. Where the site rule says "first", it means byte order of the name.

| Issue class | First-cut class (`--class`) | Site: one mutant per… | The change | Expected killer (the measurement decides) |
|---|---|---|---|---|
| drop a `from` state | `from-drop` | (transition, state in its `from`) where `from` holds at least two states **and** a command that `moves:` along it declares a `wrong_state` outcome | remove the state from `from` | the refusal scenario `…/state/<S>/refuses/<command>` the mutant now obliges (`synthesize.rs:1654-1700`), because the target still moves |
| change a transition's `to` | `transition-to` | transition, in an entity with at least two states | `to` := the first declared state that is not `to` | the transition scenario (`scenario.rs:552-557`) reading the arrival state |
| move a guard boundary | `guard-boundary` | `Compare` leaf with `<`, `<=`, `>` or `>=` in an outcome's `when`, in pre-order | swap strictness: `>=`↔`>`, `<=`↔`<` | the outcome scenario whose witness sits on the literal (`witness.rs:32-35`) |
| drop a `sets` entry | `sets-retarget` | `sets` entry whose source is `input.<f>`, where the command has another input field with an identical written type | source := the first such other field, in declaration order | a view scenario reading the set field (`ir.rs:813-819`) |
| invert or weaken a guard | `guard-negate` | outcome with a `when` that is not `Always` | `when` := `Not(when)` | the outcome scenario and the `otherwise` scenario |
| (same) | `guard-connective` | `All`/`Any` node with at least two children in an outcome's `when`, in pre-order | `All`↔`Any` | the outcome scenario |
| wrong error | `error-swap` | outcome with an `error:` | `error` := the first declared error in the command's domain that is not the current one | the `ExpectError` step of the outcome or refusal scenario |
| wrong event | `emit-drop` | (outcome, event in its `emits`) | remove the event from `emits`, and remove its `payload:` entry if there is one | the absent-event assertion (`synthesize.rs:1377-1383`), because the target still emits it |
| wrong order | `order-flip` | (view, key in `order_by`) | flip that key's direction | the ordering scenario. Where synthesis refuses to witness ordering (`OrderUnwitnessed`, `ESS-SYNTH-014`, `synthesize.rs:376`, `:546`), the mutant survives, and that survival is the finding. |

Why the site rules are shaped this way:

- **`from-drop` needs a `wrong_state` branch.** Without one, the mutant leaves the dropped state
  unspecified. That is a weakening, and a correct target satisfies it. It also needs at least two
  states: an empty `from` makes the move unreachable, which is a different mutant (deferred with
  "add a `from` state").
- **`transition-to` takes one alternative, not every other state.** Whatever the new state is, the
  scenario that observes the arrival state is the same one. One alternative is enough to show
  whether that state is pinned, and it keeps the count linear in the number of transitions.
- **`sets-retarget` replaces "drop".** Dropping is weakening (see above). Moving the source to a
  sibling field of the same type is the smallest *altering* version of the same fault: it names a
  wrong value where the issue's mutant named no value. Witness rule 2 gives same-typed text fields
  distinct values (`witness.rs:28-30`), so the swap can be told apart. Where no same-typed sibling
  exists, there is no mutant. A literal would have to be invented, which is the thing witness rule
  3 refuses to do.
- **Only `when` guards are mutated.** Subject guards, view filters, invariants, binding mappings and
  payload values are deferred. Each has its own synthesis family, and the first cut stays with the
  issue's classes.
- **`error-swap` stays in the command's domain.** An error from another domain would be refused by
  resolution far more often than it would test anything. When that happens, the mutant is
  stillborn rather than hidden.

### Running one mutant, and its verdict

For the specification and then for each mutant, the audit synthesizes a suite with
`ess_conformance::synthesize(&ir).suite` and `select_fresh_format()`, admits it, and runs it with
`Runner::for_suite(suite).run_admitted(&admitted, &new_target())` on a **fresh** target. This is the
path `main.rs:2838-2845` takes. A fresh target per run means no mutant sees another mutant's state.
The runner is deterministic by construction (§37, `lib.rs:82-88`), and so is the
enumeration order, so two audits of one tree produce identical bytes.

1. **Baseline.** The unmutated suite must end with `ConformanceStatus::Passed`. Anything else
   refuses the audit with `ESS-MUTATE-001` and lists every scenario that did not pass. Without a
   green baseline, a scenario that fails under a mutant has not been shown to fail *because of*
   the mutant.
2. **Stillborn.** `assemble` or `compile` refuses the mutant. The verdict is `stillborn`, with
   `ESS-MUTATE-002` and the first refusal's own code and message.
3. **Killed.** At least one scenario ends `Status::Failed`. The killers are exactly those scenario
   ids, sorted.
4. **Survived.** Every scenario ends `Status::Passed`.
5. **Inconclusive.** No scenario failed, and at least one ended `Error` or `Unsupported`. Nothing
   contradicted the mutant, and nobody found out. `report.rs:46-59` draws the same line.

The report also records each mutant's `scenarios` count and `refusals` count, and the baseline's
counts, so a reader can see a mutant that survived because synthesis refused the scenario that
would have killed it.

### Codes (`MUTATE` family, `crates/verify/ess-conformance/src/mutate.rs`)

| Code | When | Where it appears |
|---|---|---|
| `ESS-MUTATE-001` | The baseline suite did not pass against the target | stderr `refusal[ESS-MUTATE-001]: …` with every non-passing scenario id; exit 3; no report is written |
| `ESS-MUTATE-002` | A mutant was refused by `assemble` or `compile` | on that mutant's report entry, with the refusal's own code |
| `ESS-MUTATE-003` | The selected classes found no site in the specification | stderr; exit 3; no report is written. A vacuous audit that exits 0 would be a green exit that ran nothing. |

The numbers are derived from the variant, as `RefusalCause::code` does (`synthesize.rs:525-548`).
A test asserts that every variant's code appears in `website/docs/reference/formats.md`, so a new
code cannot ship unnamed.

### `ess-mutation-report/1`

This is a new document family, so it starts at `/1`. It is written only by `--report-out` and
`--format json|yaml`. Keys are sorted, with two-space JSON and a trailing LF, as the count reports
use. It has no timestamp, which keeps the bytes a function of the tree and the target.

```json
{
  "baseline": {"refusals": 3, "scenarios": 27},
  "counts": {"inconclusive": 0, "killed": 31, "mutants": 36, "stillborn": 1, "survived": 4},
  "format": "ess-mutation-report/1",
  "implementation": "billing-reference",
  "mutants": [
    {
      "change": "`to: Paid` becomes `to: Cancelled`",
      "class": "transition-to",
      "id": "transition-to/billing.invoice.Invoice.settle",
      "killers": ["billing.invoice.Invoice/transition/settle/by/billing.invoice.PayInvoice/settled"],
      "refusals": 3,
      "scenarios": 27,
      "verdict": "killed"
    },
    {
      "change": "…",
      "class": "error-swap",
      "id": "error-swap/…",
      "stillborn": {"code": "ESS-MUTATE-002", "cause": "ESS-COMMAND-…", "message": "…"},
      "verdict": "stillborn"
    }
  ],
  "spec_digest": "<64 hex>",
  "specification": "billing 1.0.0"
}
```

The counts in that example are illustrative, not a prediction. `killers` is present only on
`killed`, and `stillborn` only on `stillborn`. The text rendering prints one summary line, then
survivors, inconclusive, stillborn and killed, in that order, one line each. A killed line names
at most three killers and the total count.

There is no reader in the first cut. The document is output for a person or for a diff, and
nothing in this repository admits it. `formats.md` says so on its row.

### The mutants Part 1 must kill

These are defects the implementation could have, and each has a deciding test (listed later).

| Defect | What it would report | Caught by |
|---|---|---|
| runs the original suite for every mutant | nothing killed | P1-1: the billing `transition-to` mutants are killed by their transition scenario ids |
| counts a baseline failure as a kill | inflated kills against a wrong target | P1-4: a `Faulty` Billing baseline is refused with `ESS-MUTATE-001` |
| treats a stillborn mutant as killed or survived | counts that misstate the suite | P1-6: a crafted `to: Nowhere` is `stillborn` with `ESS-MUTATE-002` |
| collapses `Unsupported` into `Passed` | false survivors | P1-7: the classification table over synthetic `ExecutedRun`s |
| ships a weakening class | survivors that are certain in advance | P1-8: `MutantClass::ALL` is exactly the nine names |
| exits 0 with no mutants | a green exit that ran nothing | P1-9: `--class order-flip` on a fixture without `order_by` gives `ESS-MUTATE-003`, exit 3 |
| enumerates in hash order | reports that differ between runs | P1-5: two audits, identical bytes |

## Part 2 — the explorer

### Surface

TypeScript, from the emitted package:

```ts
import { explore, assertExplored } from 'essconform';

const result = await explore(() => newTarget(), { seeds: 200, steps: 60 });
assertExplored(result);                         // hard failure, including excluded outcomes
assertExplored(result, { allowExcluded: true }); // an explicit, reviewable opt-out
```

Go, from the emitted package:

```go
result, err := essconform.Explore(func() essconform.Target { return newTarget() },
    essconform.ExploreOptions{Seeds: 200, Steps: 60})
if err != nil { t.Fatal(err) }
essconform.AssertExplored(t, result, essconform.AssertOptions{})
```

- **Options.** `seeds` is the number of sequences, seeded `1…seeds` (default 200). `steps` is the
  number of commands per sequence (default 60). `seed`, when set, runs exactly that one sequence
  and overrides `seeds`: a failure names its seed, and this is how it is replayed. There are no
  environment-variable overrides; a deeper CI run passes different options.
- **Defaults are 200 × 60.** That is the draft's number, below the issue's 300 × 80 (about 1.3 s).
  Adopter targets are slower than an in-memory rule table.
- **`explore` throws, and `Explore` returns an error, only when the model cannot be used**: an
  absent or unreadable `ir.json`, or a digest mismatch (below). A disagreement is a `failure` in
  the result, not an exception, so a caller can inspect it.
- **The result, the same in both languages** (the Go field names are the capitalised forms):
  `sequences`, `steps`, `executed` (the number of commands actually run), `reached`, `unreached`,
  `excluded` (a list of `{subject, reason}`), `excludedOutcomes`, `undetermined`, `ambiguous`,
  `failure?` (`{seed, message, trace, originalLength, shrinkComplete}`). Every list is sorted.
- **`assertExplored` fails when** there is a `failure`, or `unreached` is nonempty, or
  `excludedOutcomes` is nonempty and `allowExcluded` is not set. That is the brief's hard failure.
  An outcome of an excluded command is never silently dropped: the default fails on it, and
  accepting it is a line of code a reviewer sees. `undetermined` and `ambiguous` are printed and do
  not fail (see below for why).

### What is emitted

New functions, so the existing signatures and their roughly 30 callers stay as they are:

- `ts::emit_with_model(suite, ir)` and `ts::emit_input_with_model(input, ir)`: the existing files,
  plus `essconform/src/explore.ts` (checked in as `src/ts/explore.ts` and copied out, like every
  other source), plus `essconform/ir.json`, with `src/index.ts` carrying one more line,
  `export * from './explore.js';`. `tsconfig.json` already includes `src/**/*.ts` (`ts/mod.rs:224`).
- `go::emit_with_model(suite, ir)` and `go::emit_input_with_model(input, ir)`: the existing files,
  plus `essconform/explore.go` (checked in as `src/go/explore.go`, carrying its own
  `//go:embed ir.json`), plus `essconform/ir.json`.
- The CLI calls the `_with_model` variants at `main.rs:2944`, `:2951` and `coverage.rs:65`, `:75`.
  `ir` is already in scope at all four. The explorer is always emitted for `--target
  typescript|go`, with no flag. It costs an adopter who never calls it nothing, and a flag would be
  a second package shape to test.
- The README gains a "Random command sequences" section in the `_with_model` variants only.

**`ir.json` is the compact IR, and it is bound to the suite.** `ess-compiler` gains
`EssIr::to_compact_json(&self) -> String` (the `serde_json::to_vec` bytes, as UTF-8), and
`source_digest` is rewritten to hash exactly that string, so the two cannot drift. `ir.json` is
that string plus one LF. Before anything else, the explorer checks that SHA-256 of the file without
its final LF equals `suite.json`'s `provenance.spec_digest` (`scenario.rs:361`, `ess-gen/src/
provenance.rs:154-156`). A mismatch is refused: "`ir.json` and `suite.json` come from different
specifications; regenerate the package". The compiled IR is unversioned and has no reader
(`formats.md:109`). This check is what lets the explorer depend on its field names: the only
`ir.json` it will read is one emitted by the same binary that emitted it. It is **compact**, not the
pretty form `ess specify compile` writes, because the compact bytes are the ones the digest is
over. Re-serialising parsed JSON in TypeScript would not reproduce serde's bytes for large integers.
TS reads the file with the runtime's `strictJSON` (`runtime.ts:530`), which keeps integers exact.
Go decodes it with `UseNumber`.

### Randomness

- **mulberry32, 32-bit state = seed.** It is the draft's generator, ported bit for bit to Go
  (uint32 arithmetic in place of `Math.imul`/`>>> 0`). `next()` is the 32-bit output divided by
  2³², which is exact in binary64 in both languages. `int(lo, hi) = lo + floor(next() * (hi-lo+1))`.
  `pick(xs) = xs[floor(next() * len)]`. `chance(p) = next() < p`.
- **Draw order is part of the contract**, so a seed names one sequence in both languages:
  1. Pick a command from the included commands, sorted by name.
  2. Draw each input field in declaration order (below).
  3. If no step can be formed (the supplied instance has no record yet), count an attempt and
     draw nothing more. Each sequence gets at most `steps × 20` attempts.
- **Every enumeration is sorted by byte order of name.** Go randomises map iteration, and TS
  object order is insertion order. Neither is allowed to leak into a draw.

**Inputs, per declared type:**

| Type | Value |
|---|---|
| `Integer` (or a newtype of it) | with `chance(0.7)`, `pick` from the pool, where the pool is the sorted distinct `{L-1, L, L+1}` for every integer literal `L` in this command's `when` guards, together with `{-1, 0, 1, 2, 3, 100}`. Otherwise `int(-10, 10000)`. |
| `Boolean` | `chance(0.5)` |
| `String` (or a newtype of it) | `pick` from the sorted distinct text literals of this command's guards, together with `{"", "a", "b"}` |
| `Uuid` (or a newtype of it) that is not an entity identity | `00000000-0000-4000-8000-` followed by `int(0, 999999)` zero-padded to 12 digits |
| a newtype that is some entity's identity type | reuse an existing record's id with `chance(0.8)`, and always when the field is the command's supplied instance field; otherwise a fresh value as for its base type |
| `enum` | `pick` from its variants in declared order |
| `struct` whose every field is in this table | each field, in declaration order |

The pool takes guard literals one either side for the reason witness rule 3 gives
(`witness.rs:32-35`): those are the values where the branches split. The draft's fixed pool left
boundaries to chance.

### The reference model

The model is a map from entity name to a map from identity to record. A record holds the
lifecycle `state`, plus the fields the model has determined. A field no outcome has set is
**absent from the record**, which is the model saying it does not know the value.

**The subset.** A command is included when all of these hold:

- every outcome's condition is `when`, `otherwise` or `wrong_state` (`ir.rs:537-588`);
- every effect is `creates` with instance `observed`, or `moves`/`updates` with instance `supplied`
  (`ir.rs:690-728`);
- no outcome `replays`;
- every input field's type is in the input table above, and no input type declares invariants;
- some actor `may` invoke it. The first such actor, by byte order of name, is used.

Otherwise the command is excluded, with the first reason in that order. A view is included when it
has no `params`, no `group_by`, and its `source` is an entity. A command with a `response` is
**included**, and the response is not compared: the model does not determine a response's values.
The draft excluded every such command. A `sets` or payload value of `generated` or
`response_field` makes that field absent (undetermined) in the model. `cleared` removes it. Only
`input_field` and `literal` give the model a value (`ir.rs:834-883`). The draft excluded the whole
command in all three of those cases.

**Which outcome a step takes.** Guards are evaluated with the runtime's own evaluator: `fromNode`
over the IR predicate node, and facts bound with `bindFact` from the input, which flattens nested
fields. This replaces the draft's regex, which could not read `amount.amount > 0`, the first guard
of the normative example. Then:

1. **A supplied subject in a state that no move of this command starts from** takes the command's
   `wrong_state` outcome, and does so before any guard. This is how `RawOutcome` defines the branch:
   "taken because the subject is in a state no move starts from" (`command.rs:3025-3032`). If the
   command declares no `wrong_state` branch, the model cannot say what happens, and the draw is
   *ambiguous* (item 4).
2. **Exactly one `when` holds**: that outcome.
3. **None holds**: the `otherwise` outcome. If there is none, the draw is ambiguous.
4. **More than one holds, or the selected outcome moves from a state the subject is not in**
   (although another move of the command starts from it): the draw is **ambiguous**. It is not
   executed. It is counted in `ambiguous` as `command` with the outcome names, and the attempt is
   redrawn. The draft took the first match in declaration order. That is a choice the
   specification does not make: overlap is refused only over finite domains (`command.rs:
   1985-2035`), and ESS does not choose where the specification is silent
   ([the linker never chooses](linker-never-chooses.md)). An ambiguous draw is a statement about
   the specification. It is printed, and it does not fail the run.

A guard that evaluates to Unknown over a generated input (possible only through a path the input
table cannot fill) excludes the command, with the guard in the reason. Unknown is not False.

**Effects.** `creates`: the new identity is read from the target's own direct event
(`instance.event`, `instance.field`). The model cannot mint identities. The failure cases are an
absent id (the target emitted no event field) and an id already present (identity reuse). The new
record's state is the lifecycle's `initial`. `moves`: set `state` to `transition.to`. `updates`:
no state change. After the effect, apply `sets`. A `wrong_state` outcome, and any outcome without a
subject, has no effect.

**Events.** Expected direct events are the outcome's `emits`, in order. Each payload field with a
determined source has an expected value. Binding-driven and later-observed events are not
modelled.

### Checked after every step

In this order. The first disagreement ends the sequence.

1. `outcome`, then `error` (both `""` when absent).
2. Direct event names, in order. Then each determined payload field, compared with the runtime's
   `equal`.
3. **Views.** For each included view: `read_your_writes` is read once, with `atLeast` set to the
   last `CommandResult.consistency`. `eventual` is polled with `atLeast: ""` up to the harness's 8
   attempts (`runtime.ts:2080`, `runtime.go:1378`), until the rows agree or the budget is spent,
   which is how an `eventually` step already waits. "Agree" means all of the following:
   - the row count and the identity set equal the model's records that pass the view's `filter`;
   - every projected field the record determines is `equal`;
   - `ranked(view, order_by, rows)` holds.

   A `filter` that is Unknown for some record (it reads a field no command set) skips that view
   for this step, and adds `"<view> filter reads <field>, which no command set"` to
   `undetermined`. This is the #112 case the issue describes. The target cannot be wrong about a
   row the model cannot place.
4. **Invariants**, over every model record. A record that is False for an entity invariant is a
   failure prefixed `specification:`. The guards allowed a sequence the invariants forbid, and the
   target is not the one at fault. An invariant that is Unknown adds to `undetermined` in the same
   way as a filter.

The runtime's `Harness` mints correlations (`runtime.ts:2088`), so ids follow the runner's own
scheme. `ErrUnsupported` from a command or a view excludes that subject for the rest of the run,
with the target's reason. Any other thrown error is a failure.

### Shrinking

Remove one step at a time, last to first. Keep a removal when the shorter trace still fails on a
fresh target. Repeat until a full pass removes nothing, or until **1,000 replays** have run
(`shrinkComplete: false` in that case, so the report does not pretend the trace is minimal). An
input field that names an existing record is stored as (entity, creation index), not as the id, so
a shorter trace re-resolves it. A step whose index no longer exists is dropped from that replay.
The reported `trace` is the shortest failing trace, with one line per step:
`<command> <input>`. The input is rendered with Go's `json.Marshal` bytes: `goMarshal`
(`runtime.ts:599`) in TypeScript and `json.Marshal` in Go. Both sort keys, so the same trace is the
same text in both languages. `originalLength` records the length before shrinking.

### Reached, unreached, excluded

- `reached` is `command/outcome` for every outcome a step took, across all sequences.
- `unreached` is every declared outcome of an **included** command that is not in `reached`.
- `excludedOutcomes` is every declared outcome of an excluded command.

### The mutants Part 2 must kill

The four engine mutants from the issue, in a toy target over a fixture specification, in both
languages:

| Mutant (`ESS_EXPLORE_MUTANT`) | What it breaks | Expected failure |
|---|---|---|
| `view-cap-5` | a view returns at most 5 rows | a view row count |
| `wrong-state-applies-sets` | a `wrong_state` answer still applies `sets` | a view field value after a refused step |
| `fourth-create-reuses-first` | the 4th create returns the first identity | identity reuse |
| `third-return-refused` | the 3rd return to a state is refused | an outcome mismatch, with a shrunk trace |

And defects the explorer itself could have:

| Defect | Caught by |
|---|---|
| RNG or draw order differs between TS and Go | P2-3: identical `trace` text and `executed` count for every mutant, in both lanes |
| first-match outcome choice | P2-6: two targets that answer the two overlapping branches both pass, and `ambiguous` names the command |
| excluded outcomes left out of the hard failure | P2-4: a target that answers `ErrUnsupported` for one command fails `assertExplored`; with `allowExcluded` it passes |
| `ir.json` not bound to the suite | P2-5: one edited byte in `ir.json` is refused |
| shrinker reports a trace that does not fail | P2-2: `trace.length ≤ originalLength`, and the replayed shrunk trace fails with the same message class |
| views compared before `sets` applied, or eventual views not polled | P2-1: the correct target passes, and reaches every declared outcome of the fixture |

## Formats and numbers

| Number | Taken | Why |
|---|---|---|
| source `ess/11` | **no** | No source construct is added. |
| suite `ess-conformance/18` | **no** | Mutant suites are ordinary suites in whatever version `select_fresh_format` picks. The explorer reads no suite field beyond `provenance.spec_digest`. |
| diff `ess-diff/8` | **no** | — |
| `ess-mutation-report/1` | **yes, new family** | A new persisted document. A new family starts at `/1`, so there is no collision to renumber. |
| code family `ESS-MUTATE-001…003` | **yes, new family** | There is no existing `MUTATE` family (measured: `grep -rhoE 'ESS-[A-Z]+-[0-9]{3}' crates`). |
| compiled IR | **unchanged, still unversioned** | `ir.json` is the existing compact serialisation, which `source_digest` already hashes. No field, name or envelope changes. Its new consumer is emitted by the same binary and checks the digest. |

## Every projection site

| Site | Part | Render or refuse |
|---|---|---|
| `ess verify conform mutate --help` (clap) | 1 | render: synopsis, the nine classes, the exit table |
| `website/docs/reference/cli.md:303-314` (`ess verify` table) | 1 | render: one row for `mutate` |
| `website/docs/guides/verify-conformance.md` | 1, 2 | render: a section "Audit the suite with specification mutants", which states that a survivor is not answered by an authored scenario, and a section "Explore random command sequences" covering the options, `assertExplored` and `allowExcluded` |
| `website/docs/reference/formats.md` | 1, 2 | render: an `ess-mutation-report/1` row (writer only, no reader, no timestamp), the `ESS-MUTATE-*` codes, and on the compiled-IR row (`:109`) the emitted `ir.json` (compact, digest-bound) |
| `crates/edge/ess-xtask/src/support.rs` `observe_conformance` (`:327-400`), which generates the support page | 1 | render: one "Mutation audit" row, with the `--target` choices read from `mutate --help` as `run`'s are (`:334`) |
| emitted TS and Go `README.md` (`ts/mod.rs:266`, `go/mod.rs:108`) | 2 | render: the explorer section, in the `_with_model` variants |
| `ess verify conform web` player | 1, 2 | refuse: nothing is added. The player replays, and neither part produces a scenario. |
| `ess verify impact` / `ess verify diff` | 1, 2 | refuse: nothing is added. No model construct changes. |
| `CHANGELOG.md` | 1, 2 | the implementation units propose `[Unreleased]` text; the coordinator writes it |
| the `ess` agent plugin (`beyond10x/agentplugins`) | 1 | not in this repository. A new verb breaks none of its checks, and the plugin can adopt it later. |

## Deciding checks for the two implementation units

**Part 1**: `crates/verify/ess-conformance/tests/mutation_audit.rs`, plus
`crates/edge/ess-cli/tests/mutate_cli.rs`.

- **P1-1.** The billing audit against `Billing`. The mutant count per class is pinned. Every survivor
  is pinned with a written reason from a closed set: `WeakWitness`, `Equivalent`, or
  `SynthesisGap { story }`. This follows `Caught::Nothing` in `faulty.rs`, and a change to the list
  needs a reason. Every `transition-to` mutant is killed by its own transition scenario id.
  A `SynthesisGap` survivor is recorded and filed as a story. It is not fixed in the unit.
- **P1-2.** The same for `oracle-fixture` against `Oracle`.
- **P1-3.** For every killed mutant, the killers are a subset of that mutant's suite ids, and each
  one ended `Failed`.
- **P1-4.** `Faulty` Billing with one fault is refused with `ESS-MUTATE-001`, and the listed ids are
  exactly the fault's failing scenarios from `tests/faults.rs`. `interpreted` is refused with
  `ESS-MUTATE-001` as well.
- **P1-5.** Two audits of billing produce byte-identical `ess-mutation-report/1`.
- **P1-6.** A crafted `transition-to` with `to: Nowhere`, pushed through the public `apply` and the
  same compile path, is `stillborn` with `ESS-MUTATE-002` and the compiler's own code.
- **P1-7.** The verdict classification over synthetic scenario-status sets: all passed gives
  survived; any failed gives killed; none failed with any error or unsupported gives inconclusive.
- **P1-8.** `MutantClass::ALL` is exactly `from-drop, transition-to, guard-boundary, sets-retarget,
  guard-negate, guard-connective, error-swap, emit-drop, order-flip`.
- **P1-9.** A class with no site gives `ESS-MUTATE-003` and exit 3.
- **P1-10.** A fixture `tests/fixtures/mutation-survivor.yaml`, where a `sets` field has a
  same-typed sibling and no view reads it, has exactly one survivor: that `sets-retarget`.
- **CLI.** Exit 0 (`--class transition-to` on billing), exit 1 (the P1-10 fixture), exit 3 (P1-9),
  the `--format json` bytes equal `--report-out`, and every code is named in `formats.md`.

**Part 2**: `crates/verify/ess-conformance/tests/explore.rs`, which drives a TS lane (compiled
with `tsc --noCheck` and run with `node --test`, as `tests/typescript_runtime.rs` does) and a Go
lane (`go test`, as `tests/aggregate_views_mutants.rs:443-484` does). The fixture is
`tests/fixtures/explore.yaml`: one entity with `Open`/`Held`/`Closed`, a create that sets an
integer, a hold (`Open→Held`, `wrong_state`), a close (`from: [Open, Held]`, `wrong_state`), an
update guarded `items >= 0` with an `otherwise` refusal, one `read_your_writes` view and one
`eventual` view ordered `items desc`, the invariant `items >= 0`, and a `Grade` command with two
overlapping integer guards. The targets are `tests/fixtures/explore-target.mjs` and
`tests/fixtures/explore_target.go`, each switched by `ESS_EXPLORE_MUTANT`. Both lanes assert on the
runner's named per-test lines, not only on the exit status. A missing `node` or `go` panics, as the
existing lanes do, rather than skipping.

- **P2-1.** The correct target passes. `reached` equals the fixture's pinned list of declared
  outcomes. `executed` is pinned.
- **P2-2.** Each of the four mutants fails with its message class. The shrunk trace is pinned, is
  no longer than `originalLength`, and fails again when run with `seed`.
- **P2-3.** P2-1 and P2-2 produce identical `trace` text and `executed` counts in TS and Go.
- **P2-4.** `ErrUnsupported` for `Close`: `assertExplored` fails and names `Close`'s outcomes; with
  `allowExcluded` it passes. `seeds: 1, steps: 1` fails with unreached outcomes.
- **P2-5.** One changed byte in `ir.json` is refused, in both languages.
- **P2-6.** Two correct targets, one answering the higher `Grade` branch and one the lower, both
  pass, and `ambiguous` names `Grade`.
- **P2-7.** A Rust test pins the first five mulberry32 outputs for seed 1 as constants, and both
  lanes print and compare them.
- **P2-8.** `ir.json` equals `EssIr::to_compact_json()` plus LF, and `source_digest` is SHA-256
  over it (a Rust unit test in `ess-compiler`).

## What each implementation unit checks before building

Each of these is **inferred** and is confirmed with one measurement before it is relied on.

**Part 1:**

- `Specification::assemble` accepts a cloned, edited `RawSpecFile` list with the same `Source`s,
  and diagnostics still locate. Build one `transition-to` mutant of billing and compile it.
- `Billing` and `Oracle` do not reject a suite whose `spec_digest` differs from their own
  specification. Run one mutant suite and look for `Error`.
- `error-swap` inside billing's domains finds a second error. If it does not, the class has no
  billing site, and P1-1 pins zero for it.

**Part 2:**

- The IR JSON keys the explorer reads: `commands`, `outcomes[].condition.kind`, `subject.effect`,
  `subject.instance.from`/`field`/`event`, `sets[].target`/`value.kind`, `payload[].event`/`fields`,
  `entities[].identity`, `lifecycle.initial`/`states`/`transitions`, `views[].source`/`params`/
  `group_by`/`filter`/`order_by`/`consistency`/`fields`, `actors[].may`, `types[].body`. Several are
  confirmed at `ir.rs:537-883` and `:1303-1310`. Read the fixture's `ir.json` once and fix every
  name that differs.
- How a view field maps to an entity field: by name, or through a projection? Read the IR of one
  view with a renamed field, if the model admits one. If it does, compare only the fields whose
  name equals an entity field, and say so in the README.
- The `wrong_state` precedence in step 1 matches what `reach_in_state` (`synthesize.rs:1580`)
  arranges. Run billing's `PayInvoice` refusal scenario and read which input it sends in which
  state. If synthesis treats an input guard as taking precedence, the model follows synthesis, and
  this page is corrected in the unit's report.
- `write_owned_files` accepts new file names inside the existing `conformance-typescript` and
  `conformance-go` families (`main.rs:2958-2964`). Emit into a directory that holds an older
  package.

## Deferred

| Item | Why not now | What it needs |
|---|---|---|
| Replaying mutant suites in the adopter's language (`mutate --target typescript\|go --out DIR`, which emits every mutant suite and a manifest, with `runMutants`/`RunMutants`) | Go has no `RunWith` (`runtime.go:1556`); two runtimes and a new manifest format make a unit of their own | Go `RunWith` over a supplied document; a manifest family; the verdict table above, ported |
| The dual direction: the unchanged suite, authored scenarios included, against a mutant implementation | needs an executing interpreter; `interpret.rs` derives nothing | a Rust reference model (Part 2's, ported), selected as a target |
| Mutating subject guards, view filters, invariants, binding mappings, payload values and literal `sets`; "add a `from` state"; an empty `from` | each has its own synthesis family; the first cut is the issue's classes | one class each, with its killer family named |
| An accepted-survivors file for equivalent mutants | needs an identity story for a mutant across edits of the specification | a stable site id that survives renumbering |
| Coverage suites (`/5` and above) and `--component` for `mutate` | parent chains and scope would have to be re-derived per mutant | `coverage_build` per mutant |
| Explorer inputs: `Decimal`, lists, `Optional`, unions, maps, timestamps, and types with invariants | exact-number generation and invariant-respecting draws | a generator per type, the same in TS and Go |
| Explorer conditions: subject fields, subject state, state changes, external outcomes, replays, `preserves` | each needs arranged state or an injected cause | the arranging steps synthesis already has |
| Views with `params` or `group_by`; binding-driven events; a persisted exploration report; an explorer in Rust against the reference targets | out of the issue's measured scope | — |

## What was rejected

- **Mutating the IR.** It needs a compiler API that can build IR no source produces, and it gives
  up validation. See "Where a mutant is applied".
- **Generating weakening mutants and marking them "expected survivors".** A survivor known in
  advance adds noise that looks like signal.
- **Running authored scenarios once per mutant.** They cannot change a verdict (see "What a
  surviving specification mutant means").
- **The draft's first-match outcome choice, regex guard reader, first-key-only order check,
  unfiltered view comparison, and exclusion of every command with a response or a
  non-input `sets` value.** Each is replaced above, and each replacement gives its reason where it
  is decided.
- **Emitting the IR in its pretty form (the bytes `ess specify compile` writes).** The digest is over
  the compact bytes, and a check against the pretty form would need a re-serialisation that TS
  cannot reproduce for large integers.
- **An `--explore` flag on `synthesize`.** It would add a second package shape to test, and it would
  save nothing for an adopter who never calls the explorer.
