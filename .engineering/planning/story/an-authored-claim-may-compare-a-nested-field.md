---
format: aep.planning-md/3
id: story:an-authored-claim-may-compare-a-nested-field
kind: story
status: draft
title: An authored scenario may compare a nested field
summary: Nested-field comparison is refused with ESS-AUTHOR-014/013 (a2a wave 1, G6).
tags:
- adopter-report
- ess-0.70.0
- feature-request
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 4
---
## Need

Reported by the adopter session specifying A2A in beyond10x/a2a, in the a2a wave-1 gaps the
orchestrator relayed on 2026-10-22 (ESS 0.67.0, a2a main 3a18050).

- **G6** (a2a wave-1 gaps list, line 15): an authored scenario cannot compare a nested field (a
  member of a struct in a response, event or view row). Refused with ESS-AUTHOR-014 /
  ESS-AUTHOR-013.

## Where it lives

- `crates/verify/ess-conformance/src/authored.rs:1981` and `:1985`: ESS-AUTHOR-013 is
  `Cause::UndeclaredField`, ESS-AUTHOR-014 is `Cause::MissingField` (numbered by
  `impl Cause => u16` at `:1938`).
- The refusal sites for nested shapes are not traced (candidates in `authored.rs` raise both causes
  from `ShapeError` at `:4805-4810`).

## Fit review

Owed: assessed with `.agents/skills/assessing-external-requests/SKILL.md` before dispatch. Check
first whether a partial (subset) claim on a struct value already expresses it.

## Acceptance (provisional)

- An authored claim may name a nested field by path; it passes when that member equals the literal
  and fails otherwise.
- A path naming no member is refused with ESS-AUTHOR-013 naming the full path.

## Schedule

Gap; tagged `ess-0.69.0` as filed from a2a wave 1.

## Second report: a partial struct in a response literal (harness, 2026-10-30)

Reported by the harness specification session against ESS 0.77.0 (f692aab). A command returns
`stop: String` and `checkpoint: Checkpoint`, a struct of `format`, `checkpoint_id` and `call`. An
authored scenario may leave the top-level `stop` out of its `response:` literal, but a literal that
gives only `checkpoint: {checkpoint_id: cp-1}` is refused:

```text
refusal[ESS-AUTHOR-001]: `gap.loop/authored/defer-partial-checkpoint` in spec/scenarios/defer-partial-checkpoint.yaml
  unreadable: response literal checkpoint: invalid_input
  help: the document is YAML with the keys `type`, `domain`, `scenario` and `summary`; a key it does not know is refused rather than ignored
```

Two findings: (1) a literal must give every member of a nested struct, though the top level of the
same response may leave members out, so a scenario cannot pin one member without writing the whole
struct; (2) the refusal says `invalid_input` for a literal that is not an input, and its help
describes the scenario document's top-level keys, which are correct, instead of naming the missing
members `format` and `call`. The message is formatted in `DirectResponse::validate`
(`crates/verify/ess-conformance/src/direct_response.rs:172-189`), which admits each literal through
`selection::validate_direct_response_value` (`selection.rs:619`).

Triage on main (f692aab, 0.77.0), 2026-10-31: re-run, identical refusal. Not fixed. This answers the
fit-review question above ("check first whether a partial (subset) claim on a struct value already
expresses it"): it does not, at a nested level.

## Where it lives (wave D325)

Lines on `main` f692aab (0.77.0), read by the scoper on 2026-10-31.

- `crates/verify/ess-conformance/src/authored.rs:3207-3231`: an act's `response:` goes to
  `direct_response::Observation::of`; any `Err` becomes `Cause::Unreadable` (ESS-AUTHOR-001) at
  `:3229`; its help (`:2192-2195`) describes the document's top-level keys. `:975-995`: `Surface` has
  no response variant.
- `src/direct_response.rs:51-53` ("every named value is compared completely"), `:172-189`
  (admission), `:202-261` (`compare`, each named field whole).
- `src/selection.rs:619-635` drops the error's path (`map_err(|error| error.code)`); `:786` a missing
  nested member becomes `invalid_input`.
- `src/input.rs:249-299` lets members be left out only at the top level under `Partial` (`:277`);
  `:1713-1744` a missing nested member is `ShapeError::MissingField` whatever the completeness: the
  cause of the original 013/014 refusals on event payloads and view rows.
- `src/leaf_payloads.rs:1-31,50-62`: suite/26 already carries dotted leaf keys (`lead.number`) in
  event payloads and view rows, read by the interpreted runner (`runner.rs:3508-3535`) and Go
  (`go/runtime.go:2810-2850`). Go `prerequisites.go:304-324` and TypeScript
  `ts/direct_response.ts:144-149` admit `expected` names only as top-level fields.
- `website/docs/reference/formats.md:364`: `response:` literals are "checked against their complete
  declared types".

## Fit review

1. **Need.** A scenario cannot pin one member of a struct inside a response, event payload, error or
   view row without writing the whole struct. Minimal reproduction: a command returns `stop` and
   `checkpoint: Checkpoint{format, checkpoint_id, call}`; `response: {checkpoint: {checkpoint_id:
   cp-1}}` is refused ESS-AUTHOR-001 `invalid_input`.
2. **Class.** Gap (nested completeness is documented) plus a defect (code 001, `invalid_input` for a
   literal that is not an input, an unrelated help, and a dropped path that `selection.rs:634`
   already computes).
3. **Already expressible?** Only by writing the whole struct. `satisfies:` on rows is an "every row"
   claim, not `contains` (inferred). The suite already carries dotted leaf keys; only the authored
   layer refuses them.
4. **Fit.** A dotted path is how ESS spells a nested read in guards (`input.a.b`) and suite/26 keys.
   The rule belongs to all four claim surfaces (response, payload, error, row); input, `setup:` and
   `caller:` are sent whole, so a path there is refused by name.
5. **Second adopter.** `Pay` returns `charged: Money{currency, amount, fx}` with `fx` set by the
   implementation; the scenario claims only `charged.currency`.
6. **Cost.** `ess-scenario/6` (precedent: `caller:`, `authored.rs:2970-2977`) and, for responses only,
   the suite pair `ess-conformance/60`/`61` with Go and TypeScript admission (precedent: suite/48-49
   added `undeclared_fields` to the same observation). No new diagnostic code.
7. **Considered.** (A) a partial nested map compares only what it gives: rejected, it changes the
   meaning of existing documents (a nested map leaving out an `Optional` member asserts its absence
   today, `undeclared_fields.rs:105`); (B) dotted path keys: chosen; (C) only the message: ships
   inside (B).

## Decisions

**accept, redesigned** (wave D325, the coordinator's call inside the approved item: B keeps every
existing byte's meaning and reuses the spelling guards and suite/26 already use).

- Message, any header, no format: a response literal is checked for missing and undeclared members
  before `Observation::of`, with a new response surface; the reporter's literal gives ESS-AUTHOR-014
  naming `format` and `call` at `checkpoint`; the 014 help says a nested value is compared whole and
  names the path form; the path is kept.
- From `type: ess-scenario/6`: a key in `response:`, an event `payload:`, an `error:` field or a row
  `contains`/`excludes`/`at` may be a dotted path through struct members (`Optional` and newtypes
  unwrapped), compared alone. A path naming no member is ESS-AUTHOR-013 with the full path; a path
  through a list, map or union is refused by name; below `/6` a dotted key keeps 013 and its help
  names `/6`; paths are refused by name on input, `setup:` and `caller:`.
- Suite: event and row paths compile to suite/26 dotted keys (a path ending at a struct is written as
  its leaves). Response paths go in `expected` under `ess-conformance/60` (`/61` with coverage);
  Rust, Go and TypeScript run them and older readers refuse them by version. Error-field paths use
  `/60` too unless every released reader already reads dotted error keys.
- CHANGELOG `[Unreleased]`: Added (`ess-scenario/6` nested-member claims, suite `/60`, `/61`) and
  Fixed (the ESS-AUTHOR-014 message for a partial nested response literal).

## Acceptance (D325, replaces the provisional one)

- `crates/verify/ess-conformance/tests/direct_returns.rs`:
  `a_partial_nested_response_literal_names_its_missing_members` (before 001 `invalid_input`; after
  014 naming the missing members at their path, under `ess-scenario/4`);
  `a_response_literal_may_compare_one_nested_member_by_its_path` (`/6` `{item.label: nested}` compiles
  to `expected["item.label"]` at `/60`, passes when another member differs, fails when it differs);
  `a_response_path_is_refused_below_its_formats` (`/5`: 013 naming `/6`; a path naming no member:
  013 with the path; the suite relabelled `/58`: refused by version before target effects).
- `crates/verify/ess-conformance/tests/leaf_payloads.rs`:
  `an_authored_payload_error_and_row_claim_may_name_a_nested_member_by_path`.
- The TypeScript and Go runtime tests: a response leaf path compares one member and is refused below
  `/60`, with the same verdicts as Rust.
- `crates/edge/ess-cli/tests/authored_scenarios.rs`:
  `a_nested_response_member_claim_validates_and_synthesizes` (the brand-free stop/checkpoint
  specification: validate exit 0 and 0 refusals; the partial-map variant exit 1 with 014 naming
  `format` and `call`).
