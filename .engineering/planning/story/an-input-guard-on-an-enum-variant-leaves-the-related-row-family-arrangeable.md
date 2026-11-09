---
format: aep.planning-md/3
id: story:an-input-guard-on-an-enum-variant-leaves-the-related-row-family-arrangeable
kind: story
status: draft
title: An input guard on an enum input's first variant leaves the related-row family arrangeable
summary: An input guard on the first variant of an enum input blocks the arrange_related_row family with ESS-SYNTH-008 (beyond10x/ess#731)
tags:
- from-consumer-report
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#731
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
- decomposes: epic:zero-open-issues-2026-11
revision: 3
---
## Need

Under the one precedence rule (`docs/design/generic-language-features.md` § G4, lines 448-500) an
input-guarded refusal answers before a missing related row, so the `exists: false` scenario must be
sent an input that no input-guarded refusal takes. Synthesis looks for that input among the plain,
undirected candidates only; where an input refusal takes the plain value of an enum input (its first
variant), no candidate misses it and the `exists: false` branch is refused ESS-SYNTH-008 with a text
about the scenario family's strategy, not about the refusal that took every candidate.

Minimal reproduction (`ess/29`, one file; `spec/marks.yaml` in the triage reproduction):

```yaml
types:
  - name: example.marks.Kind
    kind: enum
    variants: [{name: Satisfied, wire: satisfied}, {name: Unsupported, wire: unsupported}]
commands:
  - name: example.marks.RecordDecision     # creates example.marks.Decision
  - name: example.marks.RecordMark
    input:
      - {name: mark_id, type: example.marks.MarkId}
      - {name: kind, type: example.marks.Kind}
      - {name: decision, type: example.marks.DecisionId}
    outcomes:
      - {name: not-permitted, when: kind == Satisfied, error: example.marks.NotPermitted}
      - {name: exists, existing_instance: true, error: example.marks.Exists}
      - {name: no-decision, when_related: {via: input.decision, exists: false}, error: example.marks.NoDecision}
      - {name: recorded, creates: example.marks.Mark, instance: mark_id, sets: {kind: input.kind, decision: input.decision}, ...}
```

```console
$ ess verify conform synthesize --path spec/marks.yaml --out spec.suite.json
refused: refusal[ESS-SYNTH-008]: outcome example.marks.RecordMark/no-decision has no scenario `example.marks.RecordMark/outcome/no-decision`
  its command's strategy is `arrange_related_row`: the row its `when_related:` guard reads — ... — selects its branch, and this scenario family arranges none
5 scenario(s) (0 authored), 1 refusal(s)
```

With `when: kind == Unsupported` instead, or with `example: Unsupported` on the `kind` input, the
suite has 6 scenarios and no refusal, and `ess verify conform run --target interpreted` passes all 6.

Cause, from the code: `unclaimed`
(`crates/verify/ess-conformance/src/synthesize/related_guard.rs:4472`) tries
`candidates(ir, command, &[], distinction)`, the plain candidates with no goal, and returns
`unarranged()` (line 415, the strategy text) when every one is taken by an input-guarded refusal
(`input_refusal_taking`, line 4389). Its callers `without_row` (line 4307) and the overlap search
(line 4459) reach it only after the guard-directed candidates of `without_row_each` (line 4526),
which are chosen to be *taken* by each input guard, all failed.

## Acceptance

- `ess verify conform synthesize` on `spec/marks.yaml` writes
  `example.marks.RecordMark/outcome/no-decision` with no refusal; its send names a fresh `decision`
  and `kind: Unsupported`, and expects `example.marks.NoDecision`.
- `ess verify conform run --target interpreted` passes every scenario of that suite.
- Where every value of the input is taken by some input-guarded refusal (both `kind == Satisfied`
  and `kind == Unsupported` refused), the `exists: false` branch is refused with ESS-SYNTH-003 whose
  text names the refusal that took the input (`every such input taken first by `<branch>` (<guard>)`,
  the wording `crates/verify/ess-conformance/src/synthesize/subject_fact.rs:3888` already uses),
  not the ESS-SYNTH-008 strategy text.
- A suite that synthesized `exists: false` before keeps its bytes.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- inferred: crates/verify/ess-conformance/tests/missing_related_row_beside_input_refusal.rs
- inferred: CHANGELOG.md
confidence: high

## Fit review

1. **Need.** A command with an input-guarded refusal on an enum value and a `when_related:
   exists: false` branch loses that branch's scenario. Reproduction above, brand-free. The
   requester's expectation: choose an input that passes the earlier guard without an `example:`,
   and name that guard in the refusal.
2. **Class.** Defect. G4 orders input refusals before related rows
   (`docs/design/generic-language-features.md:481-489`), so a witness that misses the refusal exists
   (`kind: Unsupported`), and the swapped-variant control shows synthesis finds it when it happens
   to be the plain value (`repro/run.txt`, case 3).
3. **Already expressible?** `example: Unsupported` on the input (from `ess/11`,
   `website/docs/reference/formats.md:158`) makes the plain candidate miss the refusal (`repro/run.txt`,
   case 2). It is a workaround, not an idiom: `example:` states a sample value, and needing it here
   couples a sample to a generator limit; the next guard on `Unsupported` breaks it again.
4. **Design fit.** No authored surface. The fix hands the negated input-guarded refusals to the
   candidate search in `unclaimed`, as `admits_plain` already refutes sibling refusals for other
   families (`crates/verify/ess-conformance/src/synthesize.rs:9617`), and reuses the existing
   "taken first by" wording for the refusal. It covers every input type the witness search
   directs (enum, integer bounds, text), not only enum variants; the sibling path through
   `drive` for arranging runs calls the same function.
5. **Second adopter.** `example.loans.Apply` refuses `when: amount > 10000` and has
   `when_related: {via: input.customer, exists: false}`; if the plain `amount` exceeds the bound,
   the missing-customer scenario is lost the same way.
6. **Cost.** No format version, keyword or diagnostic code. Suites that refused gain one scenario;
   refusal text changes from ESS-SYNTH-008 to ESS-SYNTH-003 only where no input misses the
   refusals.
7. **Alternatives.** (a) Change nothing and document `example:`: rejected above. (b) Try every enum
   variant in `unclaimed`: covers enums only. (c) Chosen: direct the candidate search with the
   negated input refusals; general across input types, and the refusal names the guard when the
   search finds nothing.

## Decisions

accept as proposed: both expectations of the request (choose an input that passes the earlier guard
without `example:`, and name that guard in the refusal) are adopted unchanged; no authored surface,
no format bump.

## Source

github:beyond10x/ess#731
