---
format: aep.planning-md/3
id: story:feature-request-233
kind: story
status: draft
title: 'Value expressions: dotted input paths in sets:/payload:, field arithmetic, sibling-field comparison, byte length'
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#233
relations:
- serves: vision:O2
revision: 4
---
## Outcome

Value expressions: dotted input paths in sets:/payload:, field arithmetic, sibling-field comparison, byte length (beyond10x/ess#233).

## Status

Feature request, triaged 2026-09-29 as outside the 0.41 and 0.42 defect batches. Not scheduled; acceptance is written when it is.

## Reconciliation

Backlog reconciliation (coordinator, 2026-10-01).

- Item 5 (a trusted-time operand against a stored deadline) is the same need as #244's elapsed-time guard and moves there. Items 1–4 stay here.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md` (fit review 2026-10-01; repros under `~/.cache/ess-gaps/fit2/`, run on ess 0.44.0 unless stated).

# Fit review: feature-request-233 (value expressions)

Tested with `ess 0.44.0`; rules re-read in the 0.48.0 read tree (`gaps-270`, `e3bc9a2ff`). Repros are `repro-233/*.yaml`, one variant of `base.yaml` each.

1. **Need**, as five facts plus one comment:
   - (1) A stored field is set from a member of a struct input. `1-dotted-sets.yaml`: `sets: {generation_id: input.opening.generation_id}` -> `ESS-COMMAND-001`. `when_subject` reads such a path (`predicates.md:752`).
   - (2) A field is a constant offset from another. `2a-arith-invariant.yaml`: `upper == lower + 5` -> `ESS-ENTITY-002`, read as the text `lower + 5`. `2b`: `upper - lower <= 4000` -> parse error.
   - (3) One stored field is ordered against a sibling. `3a`: `lower <= upper` -> `ESS-ENTITY-002`. `3c` (the comment's pool): `leased >= capacity` -> `ESS-COMMAND-002`.
   - (4) A UTF-8 byte-length limit. `4-byte-count.yaml`: `.bytes` -> `ESS-ENTITY-003`, and `.count` counts scalar values (`docs/design/string-alphabet-and-length.md:199-215`). Plus a control-character exclusion.
   - (5) A stored deadline against the current time. `5-now-subject.yaml`: `expires_at <= now` in `when_subject` -> `ESS-COMMAND-002` "admitted only in a command outcome's `when:`".
   - All results are from 0.44.0. The tree is unchanged: `command.rs:2529-2545`, `docs/design/current-time-guards.md:41,151`.
   - *Requester's syntax:* infix arithmetic over Integer and Timestamp, bare sibling names, a byte predicate, `now` in `when_subject`.
2. **Class.**
   - (1) gap: the flattening workaround is refused by Entity Runtime for Optional members (per the issue; not reproduced).
   - (2) gap.
   - (3) gap for stored counters. The struct idiom validates (`3b-struct-invariant.yaml`, `3d-sibling-guard-struct.yaml`, the latter synthesizing 5 scenarios with 0 refusals). It forces the two fields into one struct, and `{increment:}` targets a whole required numeric field (`value-expressions.md:59-65`), so `leased` cannot be both counted and compared that way.
   - (4) byte length is a gap. The control-character exclusion is a convenience: `4b-control-idiom.yaml` (`none:` of `contains`) validates and synthesizes on 0.44.0.
   - (5) gap. It was deliberately left out (`current-time-guards.md:151`), not refused on principle.
3. **Already expressible?** Only (3) via the struct and (4b) via `none`/`contains`, shown above. Nothing else exists.
   - `value-expressions.md:318` excludes arithmetic.
   - `predicates.md:189-193` makes a bare RHS a literal.
4. **Fit of the requester's design: partial.**
   - General infix arithmetic (`upper - lower <= 4000`, field minus field) adds an expression grammar no evaluator lane has. Every relation asked for is linear with one constant, and `a - b <= k` is `a <= b + k`.
   - The `now ± offset` operand grammar already exists (`current-time-guards.md:27-36`) and is the shape to generalise.
   - Bare sibling names need the same reading rule as #225.
   - `now` in `when_subject` must also reach `when_related` (sibling construct).
   - A byte count is a sibling of `.count`, not a new predicate.
   - So, redesigned as family F (Q7).
5. **Second adopter.**
   - Already one in the issue comment: a lease pool, `leased <= capacity`.
   - Others: a coupon `valid_until > valid_from`; a column limited to 255 UTF-8 bytes; a lock `held_until <= now` releasable.
6. **Cost.**
   - Everything goes in under the ess/20 bundle (`spec-versions.md:59`, unreleased).
   - IR keeps predicate text, so A1/A2 change meaning without changing IR shape. That needs the format gate.
   - A3/A2 in command guards: no suite format, since they are decided at synthesis.
   - In invariants and view filters, A2 and C reach `satisfies` expectations, so Rust, Go and TypeScript evaluators plus a suite major are needed, as for E7 (`value-expressions.md:199-202`).
   - Entity Runtime refuses `now` (R-62, `current-time-guards.md:139-145`) and value expressions. It refuses A2/A3 by name.
   - New diagnostics: none beyond `type_mismatch`/`unsupported_format_version`.
7. **Considered. Family F, one design note for this cluster** (alternatives: change nothing, the requester's infix arithmetic, per-issue spellings):
   - **A1 operands.** On the right of any comparison, a bare word naming a root of the place (field, input field, quantifier binder) is that fact. Today it is refused (`expression.rs:1003-1016`), so no valid document changes meaning; the binder case is a silent literal today (`repro-237/1e-nested-binder.yaml`). `input.<path>` is admitted in `when:` too.
     Covers #225, #233 (3) and the comment.
   - **A2 offset.** Right side `<fact|now> ± <constant>`, one offset, Integer (whole number) and Timestamp (`s/m/h`, no days, as `now`). No field minus field and no left-side arithmetic.
     Covers #233 (2) and the #244 elapsed rule (`promoted_at <= now - 24h`).
   - **A3 now.** The current-time operand is admitted in `when_subject` and `when_related` predicates, still not in invariants, filters or selections. The stored instant is arranged from a creator's input as a `now_offset` (`current-time-guards.md:86-92`). An implementation-stamped instant (`{generated: true}`) is refused by name: no clock seam.
     Covers #233 (5) and #244.
   - **A4 paths.** `input.<dotted path>` in `sets:`, `payload:` and `else:`, the path grammar `when_subject` already reads. Covers #233 (1).
   - **B row sets.** `when_related: {entity, where, exists|count|forall}` (see `237-fit.md`). Covers #228, #237 (2) and the #229 follow-up.
   - **C collection and text.** `distinct` over a list (#237 (1)) and `.utf8_bytes` beside `.count` (#233 (4)). The control-character case uses the idiom.
   - The requester's free arithmetic was refused; the bare-name and `now` asks were taken.

## Decisions

- **accept, redesigned (proposed):** Split along family F, all under the ess/20 bundle:
  - (1) -> A4 dotted input paths in values.
  - (2) -> A2, one `± constant` offset on the right side. General infix arithmetic and field-minus-field are refused, since `a - b <= k` is written `a <= b + k`.
  - (3) and the comment -> A1, a bare right-hand-side root.
  - (5) -> A3, `now` in `when_subject` and `when_related`.
  - (4) -> C `.utf8_bytes`.
  - The control-character exclusion is **declined with the idiom** (`none:` of `contains`, `repro-233/4b-control-idiom.yaml` validates and synthesizes on 0.44.0).

  Overlaps: #225 duplicates (3); #244 item 1 duplicates (5) and needs (2). Not already fixed (tree `e3bc9a2ff`: `command.rs:2529-2545`, `current-time-guards.md:151`).

- Coordinator (2026-10-01): adopted as family F below. Format: ess/21, because ess/20 ships alone in 0.49.0 and family F lands as one bump. Family F (one design across #225, #228, #233, #237, #244): A1 a bare right-hand root names a field, input or binder; A2 one `± constant` offset (Integer or Timestamp); A3 `now` in `when_subject`/`when_related`; A4 `input.<dotted path>` in values; B `when_related: {entity, where, exists | count | forall}` over row sets; C `distinct` over lists and `.utf8_bytes`.
