# What changed

What each ESS release is worth to somebody using it: what became possible, how much it matters, and where to read the rest. `CHANGELOG.md` is the complete record at the level the change was made; this is the short one.

Generated from `changes/*.yaml` by `cargo xtask whats-changed`. Edit a fragment, not this file. A release with no entry added nothing an adopter would act on.

| Release | Change | Kind | Impact |
|---|---|---|---|
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
