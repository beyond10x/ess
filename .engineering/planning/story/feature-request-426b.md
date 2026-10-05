---
format: aep.planning-md/3
id: story:feature-request-426b
kind: story
status: active
title: The refusals a case-record generator hits name the repair
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#426
relations:
- decomposes: story:feature-request-426
- serves: vision:O2
scope:
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/finite.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/subject_fact.rs
- confidence: cited
  path: crates/specify/ess-domain/src/types.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/enum_refusal_shapes.rs
- confidence: cited
  path: docs/design/typed-literals-and-unknown-instances.md
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T14:53:29Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-10-05T14:53:30Z", actor: "human:timo", revision: 5}
---
## Outcome
Resolve beyond10x/ess#426 (part b): the refusals a case-record generator hit say what to write instead.

## Origin
beyond10x/ess#426, filed 2026-10-05; spike findings 1 and 3 of a downstream case-record generator on ess 0.52.0. Fit review of the whole request is in story 426.

## Fit review
1. Need: three refusals do not name the repair. (i) `variants: [True, False, Unknown]` gives `system.yaml: invalid type: boolean \`true\`, expected a variant name, …` with no key path and no repair (probe `<fit-review scratch>/probe-426/a`). (ii) `sets: {tests_pass: True}` over that enum gives "`true` is not a variant … hint: variants: True, False, Unknown" (probe `g`). The variant `True` is listed and the fix, quoting, is not named. (iii) Four three-valued `when_subject` fields in a guarded pair give "coverage is open, unsupported, or exceeds 64 joint assignments" without saying which; here it is 81 > 64 (probe `f`). The requester proposed no syntax.
2. Class: a gap in diagnostics. None of the three is a semantic defect: YAML 1.2 reads `True` as a boolean before ESS sees it, and the cap is documented (docs/design/cross-record-and-stored-field-guards.md:215). Finding (ii) sits against docs/design/typed-literals-and-unknown-instances.md:24, whose enum column reads "refused: **quote it**". The code gives the quote-it hint only where the lower-case quoted form is admitted (crates/specify/ess-domain/src/command.rs:4380-4414, `quoted.or_else`), so the table overstates the behaviour.
3. Existing idiom: the repairs work (probe `b` quotes; 426a's refusal default). What is missing is a diagnostic that names them. The sibling path already has the repair: `RawPayloadSource` reads YAML scalars "so the rule that types a literal can say what to write instead" (command.rs:1485-1489).
4. Fit: reuse the existing codes and sites; there is no new code.
   - (i) Add `visit_bool` to the `EnumVariant` visitor (types.rs:778-790). It refuses with `quote it`, and the variant list keeps its key path the way the literal refusals do.
   - (ii) Where a boolean over an enum field equals a declared variant ignoring case, the hint becomes `quote it: tests_pass: 'True'`. That is a hint, never an admission, so imports still never guess.
   - (iii) `finite` returns why it declined (open domain, unsupported guard, or N joint assignments over 64) in place of `None`. `subject_fact.rs:520-530` and the input-coverage site (command.rs:3333-3340) render the reason. It is an internal Rust type, so no format consequence (AGENTS.md "Determinism and formats").
5. Second adopter: anyone writing `variants: [Yes, No]`, or `On`/`Off` under YAML 1.1 readers, hits (i). Any domain with five Boolean-like stored flags in one guard hits (iii).
6. Cost: diagnostic message and hint text change, with no new `ValidationCode`, so `ess-diff` and the formats are unchanged. The diagnostics reference and the typed-literals table row are corrected. Tests that pin the old message text update.
7. Alternatives: change nothing, leaving the repair to 426a's note (rejected: the note is not where a refused author looks); admit a YAML boolean as the variant `True` (rejected: ESS cannot tell whether `True`, `true` or `TRUE` was written, which would be a guess); chosen: name the repair.

## Decisions
Accept, redesigned (the requester asked for none of this; it comes from findings 1 and 3). The three diagnostic refinements above, at existing codes and sites. No format bump (ess/22, ess-conformance/43 unchanged).

## Acceptance
- boolean_enum_variant_is_refused_with_quote_repair: `variants: [True, False, Unknown]` is refused at `types.<type>.variants`, hint `quote it`.
- boolean_literal_matching_a_variant_names_the_quoted_variant: `sets: {f: True}` over an enum declaring `"True"` is refused `type_mismatch`, hint `quote it: f: 'True'`.
- boolean_literal_matching_no_variant_keeps_variant_list: `sets: {f: true}` over `[Holds, Fails, Unknown]` keeps `hint: variants: …`.
- finite_cap_decline_names_the_count: four three-valued fields in a guarded pair give `non_exhaustive_branches` naming `81 joint assignments exceed 64`.
- finite_open_domain_decline_names_the_field: `weight_kg > 20` with no default names the open field and not the cap.
- typed_literals_table_matches_behaviour: the enum column of docs/design/typed-literals-and-unknown-instances.md:24 states the conditional rule.

## Scope
- crates/specify/ess-domain/src/types.rs  cited — EnumVariant visitor, lines 778-790
- crates/specify/ess-domain/src/command.rs  cited — scalar_representation 4389-4414; input coverage 3333-3340
- crates/specify/ess-domain/src/command/finite.rs  cited — MAX_ASSIGNMENTS line 10; analyze_with_fields 294
- crates/specify/ess-domain/src/command/subject_fact.rs  cited — cap message 520-530
- docs/design/typed-literals-and-unknown-instances.md  cited — table row line 24
- crates/specify/ess-domain/tests/enum_refusal_shapes.rs  inferred — home of (i)/(ii) scenarios
