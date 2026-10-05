---
format: aep.planning-md/3
id: story:feature-request-434
kind: story
status: draft
title: A measure of how complete a specification is
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#434
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/validate_completeness.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: website/docs/guides/specify/layout-and-validation.md
- confidence: inferred
  path: website/docs/reference/cli.md
revision: 5
---
## Outcome
Resolve beyond10x/ess#434: A measure of how complete a specification is.

## Origin
beyond10x/ess#434, filed 2026-10-05; feedback from adopters writing first specifications (2026-09-25), who build "how finished is this spec?" by hand from `UNMAPPED:` markers and an open-question list.

## Fit review
1. Need: one machine-readable, gateable answer to "what does this specification leave undecided": what synthesis cannot hold, what the model does not answer, and what is left to the implementation. Domain fact, brand-free: a specification of a shipment domain validates, but two commands get no synthesized scenario and one view's order is undecided, and the author cannot gate a merge on "no new gaps" without scraping text. Requester's proposal (theirs): `ess specify status`, or a field in `ess specify validate --json`, that reports open questions, `UNMAPPED` markers, constructs refused by name, declarations without a synthesized scenario, and obligations left to the implementation.
2. Class: convenience for most of it, gap for the gateable part, local policy for the comment markers.
   - Comment markers: `UNMAPPED:` and "open question" are YAML comments that the plugin skills prescribe (`agentplugins/plugins/ess/skills/specifying/SKILL.md:158-174`, `retrofitting/SKILL.md:11,140`). ESS reads no comments by design: the model digest is of the IR, "two source trees that differ only in comments … mean the same system" (`crates/generate/ess-gen/src/provenance.rs:28-32`). The only `UNMAPPED:` that ESS knows is in `ess-ui/1` (`schemas/ui/ess-ui.schema.yaml:74`).
   - Gap: conformance-synthesis refusals, `outside` and `notes` are printed only as text lines (`crates/edge/ess-cli/src/main.rs:4561-4571`). `ess verify conform synthesize --format json` emits only the suite (`provenance`, `scenarios`; probe on 0.52.0 against `examples/billing`), so they cannot be gated without scraping.
3. Existing idiom: most of the measure exists today.
   - Declarations without a scenario: "every construct that gets no scenario appears in `Synthesis::refusals`", with a stable code and subject (`crates/verify/ess-conformance/src/synthesize.rs:16-18,617-640`), printed as `refused:`. Questions the model does not answer are printed as `note:` (`main.rs:4569-4571`).
   - Implementation obligations and refused capabilities: `ess generate synthesize` prints `48 capabilities: 40 generated, 4 obligation(s), 4 refused` (installed 0.52.0, `examples/billing`), and `plan.json` holds one `disposition` per capability (`--format json`, key `plan.json`). These depend on the target.
   - Refusals of the specification: `ess specify validate --format json` `diagnostics[].code`.
   - Precedent for extending validate: it already compiles listed authored scenarios through `ess_conformance` and adds `scenario_refusals` to its JSON only when it has some (`main.rs:2491-2531,4855-4870`).
4. Fit: extend the existing report and add no verb. When the model resolves, `ess specify validate --format json|yaml` gains an additive `completeness` object that `skip_serializing_if` leaves out when empty, as `scenario_refusals` is today:
   - `unscenarioed`: each conformance-synthesis refusal as `{code, subject, scenario}`;
   - `outside`: each scenario held outside the selected component;
   - `unanswered`: each synthesis note;
   - `counts`: one per list.
   Text mode adds one summary line. Exit status does not change, because completeness is advisory. Target-dependent obligations stay in `ess generate synthesize` `plan.json`. Validate names no target, and choosing Rust silently would be a decision made per projection (`synthesize.rs:20-23`). The page says to read them there. Comment markers stay with the plugins. The author agent already reports every `UNMAPPED:` (`specifying/SKILL.md:322`), and counting comments in ESS would make comments meaningful. Codes come from `Refusal::code()`, so gating on them is stable.
5. Second adopter: a team specifying a payroll domain gates CI on "`completeness.counts.unscenarioed` does not grow" while it drafts commands across several pull requests. The need is the same, and nothing in it is specific to one team.
6. Cost: no `ess/23` and no `ess-conformance/N` bump, because the validate report is not a versioned format (no `format` key; `main.rs:4855-4870`). It is an additive JSON field. `validate` now runs conformance synthesis, which is slower on large models (unmeasured). There are no new diagnostics and no new keywords. A docs page records the composite recipe.
7. Alternatives: change nothing and document the three commands. Rejected, because the refusals cannot be gated without scraping text. A new `ess specify status` verb with its own `ess-status/1` format: a new verb and a new format for data that one existing report can carry, so it is rejected. Count `UNMAPPED:` comments: declined, comments are not the model (question 2). The requester's shape is taken as their second option, "a field in validate", and the comment-marker and implementation-obligation parts are redirected.

## Decisions
accept, redesigned. `ess specify validate --format json|yaml` gains an additive `completeness` object: conformance-synthesis refusals by stable code and subject, scenarios outside a component, unanswered-question notes, and counts. It is advisory, and the exit status does not change. Declined with the idiom: `UNMAPPED:` / open-question comments are plugin conventions, and the `ess` plugin's author and retrofitter report them. Implementation obligations stay in `ess generate synthesize` `plan.json` per target. No format bump.

## Acceptance
- validate_json_reports_unscenarioed_constructs: a specification with one construct that synthesis refuses lists it under `completeness.unscenarioed` with the same code `conform synthesize` prints as `refused:`, and exits 0.
- validate_json_reports_unanswered_notes: a note that `conform synthesize` prints appears under `completeness.unanswered`.
- validate_json_reports_outside_scenarios: a specification with a component selected and a scenario held outside it lists that scenario under `completeness.outside`, as `conform synthesize` prints it, and `completeness.counts.outside` is 1.
- completeness_absent_when_nothing_is_owed: a specification with no synthesis refusal, outside scenario or note emits today's JSON unchanged (the field is omitted when empty); `examples/billing`, which prints one actor note, carries it under `unanswered`.
- completeness_counts_match_synthesis: counts equal the `refusal(s)` figure of `conform synthesize` on the same input.
- completeness_does_not_change_exit_status: an invalid specification still exits 1 with no `completeness` object; a valid one with refusals exits 0.
- validate_completeness_guide_section: `website/docs/guides/specify/layout-and-validation.md` has the heading `## How complete is it` after `## Validate early, read the refusals`, naming `completeness`, `unscenarioed`, `outside`, `unanswered`, `plan.json` and `UNMAPPED:`. A case in `crates/edge/ess-cli/tests/validate_completeness.rs` reads the page and fails on a missing heading or name.

## Scope
- crates/edge/ess-cli/src/main.rs  cited — `validate` (2491-2561), `ValidationSummary` (4853-4870), conform-synthesize text lines (4561-4571); #437 depends on this story and lands its `warnings` field after it
- crates/verify/ess-conformance/src/synthesize.rs  cited — `Synthesis::{refusals,outside,notes}`, `Refusal::code`
- crates/edge/ess-cli/tests/validate_completeness.rs  inferred — scenarios above
- website/docs/guides/specify/layout-and-validation.md  inferred — `## How complete is it` section after :115
- website/docs/reference/cli.md  inferred — validate JSON field
