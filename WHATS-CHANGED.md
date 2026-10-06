# What changed

What each ESS release is worth to somebody using it: what became possible, how much it matters, and where to read the rest. `CHANGELOG.md` is the complete record at the level the change was made; this is the short one.

Generated from `changes/*.yaml` by `cargo xtask whats-changed`. Edit a fragment, not this file. A release with no entry added nothing an adopter would act on.

| Release | Change | Kind | Impact |
|---|---|---|---|
| [0.54.0](#counted-event-claims-generate---check-and-validation-completeness) | Counted event claims, generate --check and validation completeness | capability | notable |
| [0.54.0](#source-format-ess23-re-keyed-records-bulk-deletes-and-enum-attributes) | Source format ess/23, re-keyed records, bulk deletes and enum attributes | capability | significant |
| [0.53.0](#experimental-protocol-specifications-and-instant-ordered-timestamps) | Experimental protocol specifications and instant-ordered timestamps | capability | notable |
| [0.53.0](#source-format-ess22-row-sets-and-calendar-windows) | Source format ess/22, row sets and calendar windows | capability | significant |
| [0.52.0](#event-publishers-recursive-rust-contracts-and-consumer-fixes) | Event publishers, recursive Rust contracts and consumer fixes | capability | notable |
| [0.51.0](#committed-generated-output-regenerates-in-another-checkout) | Committed generated output regenerates in another checkout | capability | notable |
| [0.51.0](#generated-rust-servers-select-a-branch-by-whether-the-record-exists) | Generated Rust servers select a branch by whether the record exists | capability | notable |
| [0.50.0](#synthesis-witnesses-an-aggregate-over-rows-a-whenrelated-guarded-command-creates) | Synthesis witnesses an aggregate over rows a when_related-guarded command creates | capability | notable |
| [0.49.0](#generated-servers-enforce-actor-grants) | Generated servers enforce actor grants | breaking | significant |
| [0.49.0](#a-whenrelated-guard-reads-the-related-rows-lifecycle-state) | A when_related guard reads the related row's lifecycle state | capability | notable |
| [0.49.0](#synthesis-witnesses-related-copies-owner-linked-guards-and-overlapping-branches) | Synthesis witnesses related copies, owner-linked guards and overlapping branches | capability | notable |
| [0.48.0](#an-outside-runners-results-become-a-conformance-report) | An outside runner's results become a conformance report | capability | notable |
| [0.48.0](#a-served-501-says-whether-the-effect-was-committed) | A served 501 says whether the effect was committed | capability | notable |
| [0.48.0](#ess-ui-test-runs-ui-tests-by-node-path) | ess ui test runs UI tests by node path | capability | notable |
| [0.47.0](#ess-ui1-ui-documents-load-check-and-render) | ess-ui/1 UI documents load, check and render | capability | notable |
| [0.47.0](#generated-servers-answer-with-what-a-command-published) | Generated servers answer with what a command published | capability | notable |
| [0.46.1](#ess-verify-diff-classifies-error-payload-sources-and-three-outcome-flags) | ess verify diff classifies error payload sources and three outcome flags | capability | notable |
| [0.46.0](#the-rust-target-generates-what-the-specification-fully-determines) | The Rust target generates what the specification fully determines | capability | significant |
| [0.45.0](#the-go-web-and-clap-synthesis-targets-represent-json) | The Go, web and clap synthesis targets represent Json | capability | significant |
| [0.44.0](#the-rust-synthesis-target-represents-json-and-the-public-docs-are-rebuilt) | The Rust synthesis target represents Json, and the public docs are rebuilt | capability | significant |
| [0.43.0](#instance-references-inside-structured-values-one-refusal-precedence-and-unstated-external-answers-refused) | Instance references inside structured values, one refusal precedence and unstated external answers refused | capability | significant |
| [0.42.0](#link-field-guards-overlap-precedence-counter-limits-and-unkillable-mutants-scored-apart) | Link-field guards, overlap precedence, counter limits and unkillable mutants scored apart | capability | significant |
| [0.41.0](#state-scoped-refusals-guards-over-a-related-row-delivery-context-and-a-synthesis-and-mutation-defect-batch) | State-scoped refusals, guards over a related row, delivery context, and a synthesis and mutation defect batch | capability | significant |
| [0.40.0](#generated-go-and-typescript-runtimes-run-every-suite-and-reader-side-composition-checks) | Generated Go and TypeScript runtimes run every suite, and reader-side composition checks | capability | significant |
| [0.39.0](#concurrent-histories-checked-against-the-model-and-direct-library-returns) | Concurrent histories checked against the model, and direct library returns | capability | significant |
| [0.38.0](#set-effects-caller-values-existence-selected-outcomes-bounded-retries-and-paged-views) | Set effects, caller values, existence-selected outcomes, bounded retries and paged views | capability | significant |
| [0.37.0](#outcome-shapes-subject-guards-over-the-input-new-value-types-and-a-pinned-toolchain) | Outcome shapes, subject guards over the input, new value types and a pinned toolchain | capability | significant |
| [0.36.0](#payload-and-sets-values-read-the-subject-increment-fall-back-and-nest) | Payload and sets values read the subject, increment, fall back and nest | capability | significant |
| [0.35.0](#typed-fixture-values-resolved-before-a-scenario-starts) | Typed fixture values resolved before a scenario starts | capability | significant |
| [0.34.0](#string-operators-subject-guards-aggregate-views-and-outcome-groups-and-a-mutation-audit) | String operators, subject guards, aggregate views and outcome groups, and a mutation audit | capability | significant |
| [0.33.0](#a-scan-records-only-that-a-secret-value-exists-and-a-bound-workload-can-acknowledge-foreign-containers) | A scan records only that a Secret value exists, and a bound workload can acknowledge foreign containers | capability | significant |
| [0.32.0](#openapi-30-imports-and-a-bound-workloads-unbound-sidecars-are-violations) | OpenAPI 3.0 imports, and a bound workload's unbound sidecars are violations | capability | significant |
| [0.31.0](#a-service-contract-lowers-to-entity-runtime-definitions) | A service contract lowers to Entity Runtime definitions | capability | significant |
| [0.30.0](#ess-ships-its-own-agent-plugin-and-the-binary-prints-its-skills) | ESS ships its own agent plugin, and the binary prints its skills | capability | significant |
| [0.29.0](#verify-retries-against-their-original-command-results) | Verify retries against their original command results | capability | significant |
| [0.28.0](#conformance-observes-history-and-independently-arranged-failures) | Conformance observes history and independently arranged failures | capability | significant |
| [0.27.0](#an-enum-variant-carries-its-own-wire-spelling) | An enum variant carries its own wire spelling | capability | significant |
| [0.26.0](#a-branch-may-say-the-field-it-owns-holds-nothing) | A branch may say the field it owns holds nothing | capability | significant |
| [0.25.0](#an-author-names-the-identifier-a-code-emitter-spells-a-declaration-as) | An author names the identifier a code emitter spells a declaration as | capability | significant |
| [0.24.0](#a-component-declares-its-settings-and-the-runtime-slots-are-derived-from-them) | A component declares its settings, and the runtime slots are derived from them | capability | significant |
| [0.23.0](#a-branch-can-turn-on-the-state-a-subject-is-already-in) | A branch can turn on the state a subject is already in | capability | significant |
| [0.22.0](#a-binding-can-declare-one-delivery-attempt-and-no-redelivery) | A binding can declare one delivery attempt and no redelivery | capability | significant |
| [0.21.0](#a-conformance-suite-records-what-it-covers-and-what-it-left-out) | A conformance suite records what it covers, and what it left out | capability | significant |
| [0.20.0](#a-specification-can-declare-a-finite-floating-point-field) | A specification can declare a finite floating-point field | capability | significant |
| [0.19.0](#checked-schema-imports-and-structural-data-realizations) | Checked schema imports and structural data realizations | capability | significant |
| [0.18.0](#an-authored-scenario-can-claim-that-a-consumer-halted-an-ordered-scan) | An authored scenario can claim that a consumer halted an ordered scan | capability | significant |
| [0.18.0](#a-specifications-scenarios-can-be-played-in-a-browser) | A specification's scenarios can be played in a browser | capability | significant |
| [0.16.0](#a-specification-carries-the-scenarios-an-author-wrote-not-only-the-ones-it-obliges) | A specification carries the scenarios an author wrote, not only the ones it obliges | capability | significant |
| [0.16.0](#an-authored-scenario-can-claim-a-length-of-time-and-a-target-has-to-answer-for-it) | An authored scenario can claim a length of time, and a target has to answer for it | capability | significant |
| [0.14.0](#a-component-can-declare-a-command-line-surface-and-ess-synthesizes-its-parser) | A component can declare a command-line surface, and ESS synthesizes its parser | capability | significant |
| [0.1.0](#ess-becomes-a-standalone-executable-specification-toolchain) | ESS becomes a standalone executable specification toolchain | migration | significant |

## 0.54.0 — 2026-10-06

### Counted event claims, generate --check and validation completeness

capability · notable impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.54.0)

Suites `ess-conformance/44` and `/45` match repeated event claims as a set in the Rust, Go and TypeScript runners. `ess generate --check` reports drift without writing, `ess specify validate` reports completeness, `ess specify formats` lists every format, and a CLI binding takes a trailing argument list. A membership operand naming a parameter is refused.

### Source format ess/23, re-keyed records, bulk deletes and enum attributes

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.54.0)

`ess/23` lets an update re-key its record, a value read the state held before the outcome, `deletes:` remove every row a filter selects, an `affects:` entry write one record per list element and an enum declare typed variant attributes. Synthesis witnesses state predicates, struct-identity selectors and row sets beside an addressed record.

## 0.53.0 — 2026-10-05

### Experimental protocol specifications and instant-ordered timestamps

capability · notable impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.53.0)

Experimental `ess-protospec/1` models communicating peers, bounded channels, timers and safety properties; `ess verify protocol` simulates, replays and explores it. Conformance runners in Rust, Go and TypeScript now order RFC 3339 instants by instant, not by text.

### Source format ess/22, row sets and calendar windows

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.53.0)

`ess/22` adds fact operands, constant offsets, instant comparison, UTF-8 byte length, distinct list members, typed text operands, row sets with filtered related reads, calendar windows at a fixed offset, binding payload conditions and conditional aggregate measures, each checked by synthesized conformance scenarios in the Rust, Go and TypeScript runners.

## 0.52.0 — 2026-10-03

### Event publishers, recursive Rust contracts and consumer fixes

capability · notable impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.52.0)

ESS adds declared event transports and typed Rust/Go publishers with native timestamps, positional names and constant constructors. Optional recursive Rust types, generated Go behavior, typed view parameters, UI filters and integer bounds complete the update. Breaking interfaces are documented in the release notes.

## 0.51.0 — 2026-10-01

### Committed generated output regenerates in another checkout

capability · notable impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.51.0)

Generated output committed with its `.ess-output` regenerates in a clone, a second worktree or CI when its owned files still have their recorded bytes, and an unchanged regeneration leaves `state.json` byte-identical; a state carried without matching files refuses, lists them and prints the steps to re-enroll with `ess generate output adopt`.

### Generated Rust servers select a branch by whether the record exists

capability · notable impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.51.0)

A generated Rust server looks the input identity up in the storage port before dispatch, so an `existing_instance:` refusal beside a creation and a creating `unknown_instance:` branch beside an update are generated instead of owed; a command whose lookup would not match its creations stays an obligation, and Go, Web and Clap still refuse both forms by name.

## 0.50.0 — 2026-10-01

### Synthesis witnesses an aggregate over rows a when_related-guarded command creates

capability · notable impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.50.0)

An aggregate view whose creating command has a `when_related` guard gets its `<view>/aggregate` scenario instead of ESS-SYNTH-017: each row's related row is arranged, owner-linked guard inputs name an arranged owner, and a group key read from a related row's owner link holds one owner per value.

## 0.49.0 — 2026-10-01

### Generated servers enforce actor grants

breaking · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.49.0)

In a specification that declares actors, generated Rust and Go servers take the authenticated caller and answer `403 not granted` before a command runs for a caller the specification does not grant it; served suites witness the refusal. Existing callers of `dispatch`, `handle` and `serve` must pass the caller.

### A when_related guard reads the related row's lifecycle state

capability · notable impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.49.0)

`ess/20` lets a `when_related` predicate read the related row's held lifecycle state as `state`, and synthesis arranges a related row of an entity already being arranged one level deep; earlier formats refuse the path and keep their suites.

### Synthesis witnesses related copies, owner-linked guards and overlapping branches

capability · notable impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.49.0)

Synthesis now witnesses aggregate keys and values copied from a related row, `when_related` over an `owns` via field, a fresh caller-supplied identity under swapped callers, and a `when:` beside a `when_subject:`; `ess verify diff` leaves no residual for a one-sided declaration.

## 0.48.0 — 2026-09-30

### An outside runner's results become a conformance report

capability · notable impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.48.0)

`ess verify conform report` turns a runner's per-scenario results into an `ess-conformance-report/2` whose producer profile says the results were supplied and ESS executed nothing; aep 0.66.0 records it as evidence.

### A served 501 says whether the effect was committed

capability · notable impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.48.0)

Every served `501` carries `committed`: false for an unmet obligation, true when the command took effect and delivering what it published failed. Rust shells gain `Refused::Undelivered`.

### ess ui test runs UI tests by node path

capability · notable impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.48.0)

`ess ui test` runs `ess-ui-test/1` tests, which select nodes by their document path, headless against the terminal renderer, and writes a Playwright spec for the generated React project from the same file.

## 0.47.0 — 2026-09-30

### ess-ui/1 UI documents load, check and render

capability · notable impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.47.0)

`ess-ui/1` describes an application's pages, state and live channels without naming a renderer. `ess ui check` reports findings by node path, `ess ui run --tui` runs a document in the terminal, `ess generate ui --target react` writes a React project, and `ess ui docs` renders the format reference from its schema.

### Generated servers answer with what a command published

capability · notable impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.47.0)

The generated Rust and Go servers list the events a command published in its answer, keep request headers, and name and encode every system event. Delivery is tracked per binding, so an undeliverable event no longer blocks later requests, and 501 is declared for a committed command whose delivery failed.

## 0.46.1 — 2026-09-30

### ess verify diff classifies error payload sources and three outcome flags

capability · notable impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.46.1)

`ess verify diff` reports an outcome's error `payload:` sources declared, dropped or replaced, and a moved `accepts: nothing`, `returns:` or caller-decided refusal, as their own changes instead of one `unclassified-changed`. The new kinds need `ess-diff/12`; a delta without them keeps its format and bytes.

## 0.46.0 — 2026-09-30

### The Rust target generates what the specification fully determines

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.46.0)

`ess generate synthesize --target rust` generates the command behaviours, view queries and invariant checks the specification fully determines, over storage and context ports you provide. Actor grants arrive as data, the server gains a transport-free `handle`, `--layout crate` writes one crate, and `ess/19` gives an error's fields their sources.

## 0.45.0 — 2026-09-29

### The Go, web and clap synthesis targets represent Json

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.45.0)

`ess generate synthesize --target go`, `web` and `clap` carry `Json` as `rust` already does: object members keep their order and numbers their spelling, and a clap flag takes one JSON document. No code target refuses a model for using `Json` any more, and a model without it synthesizes the same bytes as before.

## 0.44.0 — 2026-09-29

### The Rust synthesis target represents Json, and the public docs are rebuilt

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.44.0)

`ess generate synthesize --target rust` carries `Json` as the types crate's `json::Value`, unchanged on the wire. The public docs gain generated CLI and diagnostics references, Start here tutorials a test runs against the built binary, task-sized guides and a what-changed page.

## 0.43.0 — 2026-09-29

### Instance references inside structured values, one refusal precedence and unstated external answers refused

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.43.0)

`{$instance}` works inside lists, maps, structs and unions (suites `ess-conformance/32` and `/33`); acts claiming an unstated external answer are refused `ESS-AUTHOR-037`. An input refusal may sit beside held-state branches under one precedence order the explorer follows; synthesis meets nested input invariants and witnesses absent fields and stored maps.

## 0.42.0 — 2026-09-29

### Link-field guards, overlap precedence, counter limits and unkillable mutants scored apart

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.42.0)

Synthesis witnesses a guard comparing a link field with an input and a stored counter at its limit; the first declared of overlapping accepting branches answers; a relation may ride on the identity. `ess-diff/11` names newtype prefix changes, and mutation report and manifest `/3` score unkillable mutants apart from survivors.

## 0.41.0 — 2026-09-29

### State-scoped refusals, guards over a related row, delivery context, and a synthesis and mutation defect batch

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.41.0)

`ess/18` adds state-scoped refusals, `state` in a `when_subject` predicate, guards over a related row (`when_related:`) and a binding's delivery context. Synthesis now witnesses map inputs, identity and link view filters, every creating command and the record a refusal needs; mutation scores gained refusals apart from survivors.

## 0.40.0 — 2026-09-28

### Generated Go and TypeScript runtimes run every suite, and reader-side composition checks

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.40.0)

The generated Go and TypeScript runtimes run suites `ess-conformance/22` to `/27` with the Rust runner's verdicts, where 0.38.0 refused them. `ess-composition/3` adds `reader: true`, admitting the widenings a consumer may read. Duplicate names are refused once, and the schema publishes name charsets.

## 0.39.0 — 2026-09-28

### Concurrent histories checked against the model, and direct library returns

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.39.0)

`ess verify conform check-history` checks an `ess-history/1` run of several clients for linearizability against the interpreted model and holds each view to its declared consistency; the Go and TypeScript explorers record concurrent histories and inject declared faults. Source format `ess/17` adds `returns: true`, checked as a typed direct response.

## 0.38.0 — 2026-09-28

### Set effects, caller values, existence-selected outcomes, bounded retries and paged views

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.38.0)

Source format `ess/16` adds literal fallbacks, `defined()` over optional aggregates, bodiless requests, related-record values, `now` in guards, create-or-update outcomes, caller attributes, bounded retries, view paging and outcomes over every record a filter selects. `ess-composition/2` checks consumer types.

## 0.37.0 — 2026-09-27

### Outcome shapes, subject guards over the input, new value types and a pinned toolchain

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.37.0)

Source format `ess/15` adds outcome shapes (`deletes:`, `into:`, `unknown_instance:`, `accepts: nothing`, `preconditions:`), `input.` in subject guards, case-insensitive text, optional aggregates, `prefix:`, `Json` and field `presence:`. Synthesis kills more mutants; `ess` can run a pinned release.

## 0.36.0 — 2026-09-27

### Payload and sets values read the subject, increment, fall back and nest

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.36.0)

Source format `ess/14` adds `{subject: <field>}`, `{increment: <n>}`, `{input: <field>, else: {generated: true}}`, nested struct mappings and `{generated: true}` in `sets:`. Decimal literals are admitted in every format; `subject.<field>` as text is refused. Older readers refuse `ess/14`.

## 0.35.0 — 2026-09-26

### Typed fixture values resolved before a scenario starts

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.35.0)

Source format `ess/13` adds `fixture_inputs:` and `ess-scenario/3` adds `fixtures:` with `{$fixture: name}` references, compiled to `ess-conformance/18`/`19`. Rust, Go and TypeScript resolve and type-check the values from an independent provider before the scenario starts. Older readers refuse the new formats; fixture-free models and suites keep their bytes.

## 0.34.0 — 2026-09-26

### String operators, subject guards, aggregate views and outcome groups, and a mutation audit

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.34.0)

Source formats `ess/8`–`ess/12` add `starts_with`/`ends_with`/`contains`, `when_subject: {predicate: …}`, aggregate views with `group_by:`, `alphabet:` and input `example:`, and `outcome_groups:`. `ess verify conform mutate` audits a suite with specification mutants. Older readers refuse each new format; models that use none keep their bytes.

## 0.33.0 — 2026-09-26

### A scan records only that a Secret value exists, and a bound workload can acknowledge foreign containers

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.33.0)

A Kubernetes scan writes `infra-observation/3` and `infra-ir/3`, recording each Secret key as `{"present": true}` instead of a digest that confirms a guessed value; `infra-drift/3` no longer reports a rotated Secret. `ess-observed-bindings/2` acknowledges a mesh proxy or vendor agent by name and reason. Breaking: older readers refuse each new version.

## 0.32.0 — 2026-09-25

### OpenAPI 3.0 imports, and a bound workload's unbound sidecars are violations

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.32.0)

`ess infra import openapi` reads OpenAPI 3.0 by rewriting each schema to its 3.1 form, refusing a construct with no faithful one. `ess verify bindings` reports OBS-BIND-008 for a container or native sidecar in a bound workload that no binding names. Synthesize orders `Timestamp` values by the instant they name.

## 0.31.0 — 2026-09-25

### A service contract lowers to Entity Runtime definitions

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.31.0)

ess-service-contract extracts an admitted component's service contract, and ess-entity-runtime lowers it into validated Entity Runtime 0.23.0 definitions and typed host binding obligations, refusing what it cannot express by name. Breaking: `ess skill` and the in-repository plugin are removed; the plugin ships from beyond10x/agentplugins.

## 0.30.0 — 2026-09-23

### ESS ships its own agent plugin, and the binary prints its skills

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.30.0)

The ESS agent plugin (ess@ess) now ships from this repository at the binary's version, with skills for writing, retrofitting and conformance-testing a specification. `ess skill` prints the same skills from the binary, and the release gate refuses a plugin version that differs from the workspace.

## 0.29.0 — 2026-09-22

### Verify retries against their original command results

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.29.0)

Source ess/7 declares silent retries of original command results and finite state refusals. Conformance suites 12/13 compare actual retained responses and complete subjects in Rust and Go; unsupported runners refuse. Native APIs preserve typed results, and diff/6 records the new semantics.

## 0.28.0 — 2026-09-21

### Conformance observes history and independently arranged failures

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.28.0)

Source format ess/6 models input eligibility for external failures and enum subject history. Suite formats 10/11 compare actual before-and-after subject values and assert silent success. Rust, Go and TypeScript runners reject changed, missing or duplicate subjects; older formats refuse the new vocabulary.

## 0.27.0 — 2026-09-20

### An enum variant carries its own wire spelling

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.27.0)

A variant is authored as a bare name or as a mapping that also carries `wire`, `display`, `summary` and `code`, so a variant whose wire form is not derivable from its name can be declared at all. Specification format `ess/5`; delta format `ess-diff/5` adds three `TypeChange` cases, so a moved spelling no longer returns an empty delta.

## 0.26.0 — 2026-09-17

### A branch may say the field it owns holds nothing

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.26.0)

`sets: {field: {cleared: true}}` states absence. Before it, a branch either left the field alone, wrote a literal that synthesis dropped, or wrote an empty string, which is a different reading. The field's type must be `Optional<...>` and the source must be an entity; an event payload is refused because an undetermined field is not the outcome's to set.

## 0.25.0 — 2026-09-16

### An author names the identifier a code emitter spells a declaration as

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.25.0)

`naming: { code: ... }` settles a collision the emitter cannot: a target without dotted type names flattens `X.State` and an authored view `XState` to one identifier. Only code emitters read it, so wire names, document schemas and qualified names are untouched and setting one cannot break a deployed consumer. The source format stays `ess/4`.

## 0.24.0 — 2026-09-12

### A component declares its settings, and the runtime slots are derived from them

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.24.0)

`settings:` states each of a component's configuration inputs once, typed from the specification's own `types:`. `ess specify runtime compile` derives the `ess-runtime/1` config and secret slots from it, and refuses a hand-authored slot or two settings binding one environment variable twice. Absence is stated with `Optional<...>` and nothing else.

## 0.23.0 — 2026-09-11

### A branch can turn on the state a subject is already in

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.23.0)

`when_subject_state` in `ess/3` combines a declared held lifecycle state with input guards, so equal input applied to two different existing states is distinguished. `ess/4` declares error wire names without merging semantic identities and fills emitted event payloads from typed command response fields. New deltas use `ess-diff/3` and `ess-diff/4`.

## 0.22.0 — 2026-09-10

### A binding can declare one delivery attempt and no redelivery

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.22.0)

`delivery: at_most_once` states that a binding is tried once. Loss stays possible and no idempotency obligation is imposed on the handler. OpenAPI requires `Idempotency-Key` only for `at_least_once` commands, conformance records `BindingGap::DeliverySingleAttempt` instead of synthesizing a redelivery, and the format stays `ess/1`.

## 0.21.0 — 2026-09-09

### A conformance suite records what it covers, and what it left out

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.21.0)

Opt-in `ess-conformance/5` records selected generated and authored coverage, omitted scenarios, source identities and every refusal occurrence. Report 2 then distinguishes complete passing conformance from empty, unknown or incomplete coverage — a pass over nothing and a pass over everything stop reading the same.

## 0.20.0 — 2026-09-06

### A specification can declare a finite floating-point field

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.20.0)

Authored `ess/2` adds `Binary64` fields, distinct from integer and decimal values. Reference, Rust and Go preserve signed zero, subnormals and nearest-even rounding; two typed operands use IEEE equality. Synthesis and conformance refuse unsupported `Binary64` before publication rather than emitting a target that would disagree.

## 0.19.0 — 2026-09-05

### Checked schema imports and structural data realizations

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.19.0)

Import complete JSON Schema roots with source identity; generate structural Go, Rust and TypeScript libraries; and run source-pinned normalization recipes. Rust normalization has a library API; Go/TypeScript support is pending. Rust/Web synthesis returns checked failures, and sliced provenance names its digest profile.

## 0.18.0 — 2026-09-04

### An authored scenario can claim that a consumer halted an ordered scan

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.18.0)

`halts_after: <n>` lets an authored scenario require a consumer to stop an ordered scan after n rows. The target reports rows produced and whether the consumer ended the read. Targets without incremental reads report `unsupported`. This adds a claim to `ess-conformance/4`.

### A specification's scenarios can be played in a browser

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.18.0)

`ess verify conform web` emits a browser page that walks authored scenarios, showing actors, state, selected views, and declared consequences. It replays model-determined behavior without executing an implementation, so authors can inspect a specification before filling its obligations.

## 0.16.0 — 2026-09-04

### A specification carries the scenarios an author wrote, not only the ones it obliges

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.16.0)

Authors can declare modelled instances, command timelines, and expectations in `ess-scenario/1`. These scenarios compile into `ess-conformance/2` with identities distinct from synthesized suites. Twenty-seven refusals check their names against the model. Included in the published 0.16.0 release.

### An authored scenario can claim a length of time, and a target has to answer for it

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.16.0)

Authored scenarios gain `mark:` and `elapsed:` with `not_before`, `within`, and `quiet` bounds. The suite states a duration and its anchor; the target reports an observed or advanced clock. Targets without clock support report `unsupported`. The claims use `ess-conformance/3`.

## 0.14.0 — 2026-09-04

### A component can declare a command-line surface, and ESS synthesizes its parser

capability · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.14.0)

`reached_by: command_line` and a `cli:` block state that a component's callers are people at a terminal and where each command sits in the tree. `ess generate synthesize --target clap` emits the parser, the handler obligations and the completion scripts from that declaration.

## 0.1.0 — 2026-09-01

### ESS becomes a standalone executable specification toolchain

migration · significant impact · [release notes](https://github.com/beyond10x/ess/releases/tag/0.1.0)

ESS 0.1.0 extracts system modeling, schema contracts, generators, and conformance into an independent repository with no AEP dependency.
