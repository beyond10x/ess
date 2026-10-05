---
format: aep.planning-md/3
id: story:feature-request-455
kind: story
status: active
title: Overlapping input guards on an open domain pass validation
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#455
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- depends_on: story:feature-request-454
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/refusal-pair-overlap.yaml
- confidence: inferred
  path: crates/verify/ess-conformance/tests/refusal_pair_overlap.rs
- confidence: cited
  path: docs/design/cross-record-and-stored-field-guards.md
- confidence: cited
  path: docs/design/input-guard-overlap-precedence.md
- confidence: cited
  path: website/docs/guides/specify/fields-and-invariants.md
- confidence: cited
  path: website/docs/guides/specify/guards-and-predicates.md
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T14:53:31Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-10-05T14:53:32Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":2}}}
---
## Outcome
Resolve beyond10x/ess#455: Overlapping input guards on an open domain pass validation.

## Origin
beyond10x/ess#455, filed 2026-10-05; found running a conformance suite against an implementation. The same run as #454: an invalid code with `pause: true`.

## Fit review
1. Need: when two branches' input guards both hold for one input, the author must know which branch answers, and the suite must not hinge on an order nobody chose. Requester's proposal, theirs: validation refuses or warns about input guards that can both hold on an open domain unless precedence is declared, and synthesis never picks a witness two branches claim. Fresh probe `<fit-review scratch>/probe-455/spec.yaml` on installed ess 0.52.0, with refusals `invalid-code: code == ""` and `pause-refused: pause == true` and a creating default. It validates (`validate.out`) and gives 3 scenarios with 0 refusals (`synth.out`). `invalid-code`'s only send is `{code: "", pause: true}`, the overlap, requiring `invalid-code`. `pause-refused` is sent `{code: "code", pause: true}`.
2. Class: convenience, plus a witness-quality gap. Precedence is already declared, and declaration order is that declaration. Two input-guarded refusals answer "the first declared whose guard holds". A refusal answers before an overlapping accepting branch, and two accepting branches answer in declaration order (`docs/design/input-guard-overlap-precedence.md`, table under "The rule"; `docs/design/cross-record-and-stored-field-guards.md:706-735`). The adopter guide says the same (`website/docs/guides/specify/guards-and-predicates.md:74`, `website/docs/guides/specify/fields-and-invariants.md:35-46`). The probe's suite follows that rule, so it is not a defect. The gap: the first refusal's only witness sits in the overlap. A target that never refuses a blank code with `pause: false` is not caught, and a target with the other order fails a scenario whose name says "invalid code" rather than "order". Unknown: whether the reporter's suite required the later branch at an overlap point. That would be a defect, and `precedence_contradictions` (ESS-SYNTH-019, `website/docs/reference/diagnostics.md:201`) should have withdrawn it. A minimal reproduction from the reporter is needed to tell.
3. Existing idiom: (a) put the branch that should win first. (b) Make the guards disjoint, e.g. `pause-refused: {all: [pause == true, code != ""]}`. Where the finite proof can decide (enum/Boolean, no default), validation already refuses an overlap. A text or number guard requires a default (`input-guard-overlap-precedence.md`, after the table).
4. Fit: a validation refusal was considered and rejected in the existing design. It "needs an overlap analysis over text and number guards that does not exist, a format version", and would break specifications that validate today (`input-guard-overlap-precedence.md`, "Why stated precedence and not a refusal"). A warning (`Severity::Warning`, `crates/specify/ess-compiler/src/diagnostic.rs:18-23`) would fire on every intended refusal-before-acceptance overlap, which is the #178 pattern the rule exists for. That makes it noise, not a signal. The witness half fits existing rules 1 and 2 of that design, which apply to refusal/accepting and accepting/accepting pairs. Extend them to refusal/refusal pairs. An earlier refusal's own witness refutes later siblings where some input does. The overlap is sent once more, separately, requiring the first declared. Existing steps only; the `overlap_inputs`/`boundary_inputs`/`sibling_refusals` seam in `crates/verify/ess-conformance/src/synthesize.rs:6017,11934,12175`.
5. Second adopter: a transfer is refused for `amount <= 0` and, separately, for `currency == ""`. `{amount: 0, currency: ""}` should be sent once as a labelled overlap, not be the zero-amount refusal's only witness.
6. Cost: no format, keyword or diagnostic. Suite bytes change for commands with two overlapping input refusals; measure across repository models as the design did (209 specifications). One guide paragraph.
7. Alternatives: (a) change nothing. The probe's suite is correct but under-witnesses. (b) Validation refusal or warning, the requester's. Rejected in the design cited above, and it adds a format or a noisy diagnostic. (c) A declared precedence key. That is a second spelling of declaration order. (d) Chosen: decline (b)/(c) with the idiom, and add the refusal/refusal overlap witness.

## Decisions
decline, with the idiom — validation stays as it is. The story body is the decline record, with the probe-455 output above. Declaration order is the declared precedence: write the winning branch first, or make the guards disjoint with `all:`/`not`. Accepted alongside it: synthesis gives an earlier input refusal a witness outside any later sibling's guard where one exists, and sends the overlap separately. No format bump.

Depends on #454 (edge recorded), which owns the guide section `## Which branch answers` and the rule that existence and wrong-state scenarios resend a claimed input. This story's base is `docs/design/input-guard-overlap-precedence.md` "What synthesis does", rules 1–2: an accepting branch's witness refutes every sibling input-guarded refusal, and each refusal is sent again at every overlap point with an accepting sibling. It extends those two rules to refusal/refusal pairs (a primary witness of the earlier refusal outside the later one's guard, plus one overlap send requiring the earlier), and adds the paragraph `### Two input refusals whose guards overlap` inside #454's section, plus its own cases in its own test file. #450 depends on this story (edge recorded) and edits `fields-and-invariants.md` below this story's lines 35-46.

## Acceptance
- earlier_refusal_witness_avoids_later_sibling: for the committed fixture `crates/verify/ess-conformance/tests/fixtures/refusal-pair-overlap.yaml` (the fit-review model: `invalid-code` on a blank `code`, declared before a refusal on `pause: true`), `invalid-code`'s primary send has `pause: false`.
- refusal_pair_overlap_sent_separately: the same scenario also sends `{code: "", pause: true}` requiring `invalid-code`.
- swapped_order_target_fails_only_overlap: a target checking `pause` first fails only the overlap send; a target never refusing blank codes fails the primary send.
- inseparable_refusals_keep_overlap_witness: where no input separates the two guards, today's witness and bytes are kept.
- no_overlap_models_bytes_unchanged: suites for models without overlapping input refusals are byte-identical.
- refusal_pair_paragraph_states_declaration_order: inside `## Which branch answers` of `website/docs/guides/specify/guards-and-predicates.md`, the heading `### Two input refusals whose guards overlap` exists. Its text contains "the first declared answers", "make the guards disjoint", "`all:`" and "sent once more". A case in `crates/verify/ess-conformance/tests/refusal_pair_overlap.rs` reads the page and fails on a missing heading or phrase.

## Scope
- docs/design/input-guard-overlap-precedence.md  cited — the rule; "What synthesis does" rules 1–2 (64-82), this story's base, read and not edited; the rejected refusal
- docs/design/cross-record-and-stored-field-guards.md  cited — the precedence order :706-735
- website/docs/guides/specify/guards-and-predicates.md  cited — the `### Two input refusals whose guards overlap` paragraph inside #454's section
- website/docs/guides/specify/fields-and-invariants.md  cited — overlap guidance :35-46
- crates/verify/ess-conformance/src/synthesize.rs  cited — `sibling_refusals` :6017, `boundary_inputs` :11934, `overlap_inputs` :12175
- crates/verify/ess-conformance/tests/refusal_pair_overlap.rs  inferred — new regression test and the paragraph case
- crates/verify/ess-conformance/tests/fixtures/refusal-pair-overlap.yaml  inferred — the fit-review model, committed
