---
format: aep.planning-md/1
id: review-result:adversary-wave25-unit3-pass-1
kind: review-result
status: active
title: Adversary pass 1 against a component declaring its settings
relations:
- reviews: story:component-declares-its-settings
revision: 1
---
# Adversary pass 1 against `story:component-declares-its-settings`

Worktree `wt-e0cb562e0319`, uncommitted over `wave/ess-wave-25` at `e5a97603`. Verdict
**NEEDS-CHANGE**. Cases 2926 → 2930, 4 red, all in two new test files. No pre-existing case changed
state. The unit's tracked diff is byte-identical before and after the pass.

## Findings

| # | Where | Verdict / origin / severity | Measured | Reaches it |
|---|---|---|---|---|
| F1 | `ess-deployment/src/runtime.rs:870` | NEEDS-CHANGE / introduced / **blocker** | `("state-root", Optional)` where the unit's own rule requires `Required`. `adversary_component_settings.rs:275`, exit 101 | `ess specify runtime compile`. The state is the unit's **own accepted document**: its case `an_unstated_required_is_not_refused_either_way` asserts a setting typed `StateRoot` with no `required:` is accepted, and `RawComponentSetting.required` is `Option<bool>`, `default: null` in the published schema |
| F2 | `ess-deployment/src/runtime.rs:832` | INFEASIBLE / introduced / warning | a refusal naming a hand-authored slot against a runtime document with no `config:` and no `secrets:` block at all. `adversary_component_settings.rs:320`, exit 101 | **nothing found.** The adversary constructed two workloads selecting one container role. Nothing refuses that — `compile_runtime:638` refuses a *component* realized by more than one workload and says nothing about a shared container role — but no example, fixture or document in this tree builds one |
| F3 | `ess-domain/src/component.rs:1345` | NEEDS-CHANGE / introduced / **blocker** | `Specification::assemble` returns `Ok` for a setting typed by an entity and for one typed by a name nothing declares. `adversary_component_settings_seam.rs:103` and `:129`, exit 101 | `Specification::validate` is `pub` and documented at `spec.rs:264` as "Checks every reference in the specification"; a setting's `type:` is a reference. The repository's own consumer catalogue names two shipped profiles that stop here: `authored-assembly` and `authored-validation` (`consumer_coverage/profiles.json:60-80`). No in-repo **binary** reaches it — every `src/` consumer of `ess-domain` also depends on `ess-compiler`, and `ess validate` routes through `compile → compile_locating`, where both refusals do fire |
| F4 | `schemas/generated/ess.schema.json` | CONFIRMED / pre-existing / note | `RawComponentSetting.name` publishes `CliName::PATTERN`; `RawCommandLineSurface.binary` and `RawCommandGroup.name` publish nothing | any schema-aware editor. Both bare fields are `pub … : String` at `e5a97603` and this diff does not touch them. No red case written on purpose: making a pre-existing gap red routes a blocker-looking failure at a defect this unit did not cause |

## F1 stated as the contradiction rather than as a preference

`ComponentSpec::validate_settings` refuses `required: false` over a non-`Optional` type, and its own
rustdoc gives the reason: *"`Optional<…>` is the model's only way of saying that a value may be
absent"*.

Delete the `required: false` line and the identical document is **accepted**, and derives
`ConfigKind::Optional` — the same slot the refused document would have produced. The refusal does
not prevent the state it names; it is evaded by writing one line fewer.

The consequence is downstream and concrete: `environment.rs:395` requires an environment binding for
`ConfigKind::Required` slots and not for `Optional` ones, so a value the model says must be present
ships unbound with no diagnostic.

Named fix, not applied: derive `kind` from `setting.type_ref.is_optional()` when `required` is
`None`, or refuse silence. The unit chose neither, and the two are not equivalent.

## F3: information is not the blocker, ownership is

`Specification::validate` already builds the type registry (`spec.rs:270`) and the `EntityCatalogue`
(`spec.rs:315`) in the same function body, so both catalogues `validate_setting_types` needs are in
scope. The fix is one `errors.extend(crate::component::validate_setting_types(self));` in `spec.rs`
— a file this wave assigned to unit 2, which is why neither the unit nor the adversary applied it.

The unit named this seam in its own rustdoc and left a patch in scratch. The pass promotes it from
note to NEEDS-CHANGE because the promise it breaks is a `pub` doc comment and two catalogued
consumer profiles, not a hypothetical caller.

## Attacked and could not break

- **The injectivity claim is a proof, not a sample.** The map is length-preserving and injective per
  character with pairwise-disjoint images (`[A-Z]`, `[0-9]`, `{_}`), so cross-length collision is
  impossible and the three-character enumeration cannot be extended into a counterexample. The
  enumeration is exhaustive over lengths 1–3 of the charset: it generates all strings over
  `a-z 0-9 -` and filters by `CliName::new`, so one-character names, trailing hyphens, doubled
  hyphens and leading digits are all covered.
- **`CliName::PATTERN` against `CliName::new`**, differentially checked over every string up to
  length 4 over `a b z 0 9 - A _ .` — 3,000+ inputs, **0 disagreements**. The published pattern and
  the parser accept exactly the same language.
- **`Identifier::new(setting.name).expect(…)`** at `runtime.rs:860` is sound: `Identifier`'s charset
  is a strict superset of `CliName`'s. Not a panic path.
- **The typed-diagnostics census** is true of the tree: seven new `ValidationError::at` sites, the
  15 untyped sites untouched, and the head table needed no edit because `setting_site` builds a
  typed `ConstructRef` and writes no document-path string literal.
- **The old-reader test** does not regenerate its expectation, but does not compare captured bytes
  either — it asserts the string `settings` is absent from the current IR. Weaker than its name and
  not a tautology; the pinned `examples/oracle-fixture` digests supply the byte identity the name
  claims. A name overclaim.
- **The story's two errors**, both confirmed against the story body: the Shape example writes
  `state_root` and `slack_bot_token`, which `CliName::PATTERN` forbids, and the field table omits
  `value:`, which the third refusal requires. The unit was right on both, and right to add `value:`.
  These are defects in the story document, not in the tree.

```findings
- file: crates/generate/ess-deployment/src/runtime.rs
  line: 870
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "a setting typed by a type that admits no absence, with required unstated, derives ConfigKind::Optional - the very slot validate_settings refuses when the same document writes required false - so the refusal is evaded by deleting one line and environment.rs:395 never requires the value to be bound."
- file: crates/generate/ess-deployment/src/runtime.rs
  line: 832
  category: concurrency
  severity: warning
  verdict: INFEASIBLE
  origin: introduced
  message: "derive_component_settings reads the container slot lists it also appends to, so a container role selected by two workloads is refused for hand-authoring a slot the runtime document never wrote; nothing refuses a shared container role, but nothing in this tree builds one either."
- file: crates/specify/ess-domain/src/component.rs
  line: 1345
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "Specification::assemble accepts a setting typed by an entity and one typed by an undeclared name, while Specification::validate is documented at spec.rs:264 as checking every reference and the repository catalogues two shipped consumer profiles that stop there."
- file: schemas/generated/ess.schema.json
  line: 1
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: "three Raw fields share the CliName charset and only RawComponentSetting.name publishes CliName::PATTERN; RawCommandLineSurface.binary and RawCommandGroup.name reach the schema as bare strings, unchanged since e5a97603."
```
