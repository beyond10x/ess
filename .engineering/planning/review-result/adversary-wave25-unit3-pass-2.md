---
format: aep.planning-md/2
id: review-result:adversary-wave25-unit3-pass-2
kind: review-result
status: active
title: Adversary pass 2 against a component declaring its settings
relations:
- reviews: story:component-declares-its-settings
revision: 1
---
# Adversary pass 2 against `story:component-declares-its-settings`

Worktree `wt-e0cb562e0319`, uncommitted over `wave/ess-wave-25` at `e5a97603`. Verdict
**NEEDS-CHANGE**. Cases 2929 → 2933, 4 red, all four `introduced`. Three untracked test files added;
the unit's tracked diff is byte-identical before and after.

**Recorded late.** The coordinator reported this pass and did not write the record. Unit 3's
correction round 2 went looking for it, found it absent from the worktree, the primary checkout and
`aep plan artifact list`, and said so — then built against the findings as quoted in its brief and
confirmed every mechanism claim by running the four cases red before changing anything. The record
is reconstructed here from the pass's report.

One caveat the pass stated once and applies to all four findings: `settings:` is brand new, so no
document in this repository declares one. "What reaches it" names the shape the first author writes
and, where possible, an in-repo precedent for that shape.

## Findings

| # | Where | Verdict / origin / severity | Measured | Reaches it |
|---|---|---|---|---|
| F1 | `ess-deployment/src/runtime.rs:877` | NEEDS-CHANGE / introduced / **blocker** | `adversary2_component_settings.rs:255`, exit 101. `secret: true` over `Optional<oracle.order.Email>` is accepted, `is_required()` is `false`, and the derived slot serialises to `{"environment":"SLACK_BOT_TOKEN","key":"slack-bot-token","name":"slack-bot-token"}` — no field for absence | `environment.rs:403-410` requires **every** secret slot to be bound, unconditionally. There is no `ConfigKind` equivalent on `SecretSlot`. An optional credential is the story's own example domain — `slack-bot-token` is the name in its Shape block |
| F2 | `ess-deployment/src/runtime.rs:882` | NEEDS-CHANGE / introduced / warning | `adversary2_component_settings.rs:293`, exit 101. Two components of one workload declaring `log-level` are refused with `DuplicateIdentifier server container server has a duplicate configuration slot or environment variable` — naming neither setting nor component | The fixture's own workload: `ONE_WORKLOAD` (`deployment.rs:1520`) realizes `order-service` **and** `dispatch-service` in one container role `server`. This is the defect pass 1's F2 named — a refusal about a slot the runtime document never wrote — reached through `validate_container` instead, because the correction moved derivation **above** it |
| F3 | `ess-compiler/src/ir.rs:840` | CONFIRMED / introduced / warning | `adversary2_component_settings.rs:117`, exit 101. Two documents with the same `is_required()` produce `source_digest` `3fd99f4d…` and `a88b012c…` | `EssIr::source_digest` is the `semantic_digest` every realization, runtime and build document pins. **Counter-argument stated by the pass itself:** it is a *source* digest, and a `summary:` moves it too. What differs is that every other field of `ResolvedComponentSetting` is resolved — `type_ref` is a `TypeHandle` — and `required` alone is the raw `Option<bool>`, read by nothing but `is_required()` and the serialiser, while the unit engineered digest stability deliberately for the empty case |
| F4 | `ess-domain/src/types.rs:231` as consumed by `component.rs:554` | NEEDS-CHANGE / introduced / warning | `adversary2_component_settings.rs:102`, exit 101. `is_required()` answers `true` for a setting typed by `newtype of: Optional<String>`. Probed further: `required: false` over it is refused with *"typed `connectors.config.MaybeRoot`, which admits no absence"* — false about the type — and the hint offers `Optional<connectors.config.MaybeRoot>`, i.e. `Optional<Optional<String>>` | `TypeRef::is_optional` is `matches!(self, Self::Optional(_))`: syntactic, blind to a name. This repository already declares three newtypes over `Optional<…>` in `ess-compiler/tests/fixtures/adversary_expression.yaml:12,15,28`. The six-cell grid is the grid of the wrapper question only; this is its seventh cell |

## Judgement findings

| # | Where | Verdict | Reason |
|---|---|---|---|
| J1 | `ess-deployment/tests/deployment.rs:1577` | CONFIRMED / introduced / note | **"Verbatim" is true of three of four moved cases and not the fourth.** Pass 1's F1 case panicked with `…refused by \`environment.rs:395\``; the moved case says `…refused by \`environment.rs\``. Assertion, input and expected value unchanged, so nothing is relaxed — a `file:line` citation was dropped. The F2 case is strengthened exactly as claimed, and both seam cases are byte-identical in panic text, measured by running them |
| J2 | `ess-domain/src/component.rs:1366-1376` and `tests/component_settings.rs:370-384` | NEEDS-CHANGE / introduced / warning | **The coordinator's patch works and its prose does not.** Measured on a scratch copy: `spec-rs-placement.patch` and `unignore-settings-seam-cases.patch` both apply clean, the hunk lands inside `Specification::validate`, and `cargo test -p ess-domain --test component_settings` goes **12 passed, 0 failed, 0 ignored** — both seam cases green — with no regression across `ess-domain`, `ess-compiler` and `ess-deployment`. But it touches neither of two statements it falsifies: `validate_setting_types`' "# Where this runs" still says it runs "rather than inside `Specification::validate`", and the mid-file comment still says the two cases below "are `#[ignore]`d" and the patch is "held by the coordinator" |
| J3 | `ess-xtask/src/main.rs:457` | CONFIRMED / pre-existing / note | `xtask schema --check` renders `schema_for!(RawSpecFile)` and compares it byte-for-byte with the committed file — a fixed-point check on the generator, which passes whenever the document agrees with its own source. The only thing in this diff comparing the published schema against *behaviour* is the unit's own name-pattern test, and it covers one field |

## Attacked and could not break

- **The F2 traversal orderings** — a container role selected by no workload, a workload with no
  containers, a workload naming an unknown role, a component realized by no workload, a component
  realized by two workloads — all five refused by explicit checks at `runtime.rs:608-692`, before or
  independently of the derivation.
- **`validate_setting_types`' declared-type set** is exactly what `Specification::types_with_lifecycles`
  builds. No drift.
- **The single-rule-site claim** holds for Rust: exactly two readers of a setting's `required`, both
  delegating. F3 is the same claim failing for a reader of the serialised IR.
- **The schema regeneration** is purely additive: one definition, one property, 60 lines.
- **`Optional<Optional<T>>` and `List<Optional<T>>`** are refused where the grid predicts.
- **`ConfigKind::Literal`** holds the literal/value invariant on derived slots as on hand-authored ones.
