---
format: aep.planning-md/3
id: story:a-guard-compares-fields-of-two-named-rows
kind: story
status: active
title: A guard predicate compares fields of two rows the input names
tags:
- adopter-report
- deferred
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#564
relations:
- decomposes: epic:generic-row-queries
- serves: vision:O2
- informed_by: story:navigation-reaches-any-depth-in-view-filters-and-guards
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:navigation-reaches-any-depth-in-guards
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
scope:
- confidence: inferred
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/command.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/command/precedence.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/expression.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/system.rs
- confidence: inferred
  path: crates/specify/ess-primitives/src/predicate.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret/execute.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: cited
  path: docs/design/generic-language-features.md
- confidence: inferred
  path: docs/design/language-grids.md
- confidence: inferred
  path: website/docs/reference/predicates.md
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:34:09Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:34:10Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

A rule relates two rows the input names to each other, not each to a constant: a wire may join two
ports only where their directions differ (downstream finding 11, attached to this issue); two
incidents may be merged only within one environment. Today a guard reads one named row at a time
(`when_related: {via, predicate}`, or `{all: [...]}` with one predicate per row), so the comparison
is one branch per value of a finite enum and cannot be stated for an open type such as a `String`.

Minimal reproduction (ess 0.92.0, `format: ess/36`; `repro/g-two-rows`,
`repro/g2-two-rows-input-path` of the triage of beyond10x/ess#564; `first` and `second` are
`demo.ops.IncidentId`, `Incident.environment: String`):

```yaml
      - name: other-environment
        when: "input.first.environment != input.second.environment"
        error: demo.ops.WrongState
```

```text
$ ess specify validate --path g2-two-rows-input-path
error[ESS-COMMAND-003]: `first.environment` cannot select `environment` from `demo.ops.IncidentId` (resolved prefix type `Uuid`)
error[ESS-COMMAND-003]: `second.environment` cannot select `environment` from `demo.ops.IncidentId` (resolved prefix type `Uuid`)
error[ESS-COMMAND-004]: `other-environment` answers at step 1 of the precedence order and `no-first`, declared above it, at step 4; ...
exit 1
$ ess specify validate --path g-two-rows     # demo.ops.Incident[input.first].environment != ...
error[ESS-SPEC-012]: cannot parse predicate "...": left-hand side must be a fact path: ... contains disallowed character '['
exit 1
```

## Acceptance

At `ess/37`:

- `ess specify validate --path g2-two-rows-input-path` exits 0: an input whose type is one entity's
  identity navigates to that entity's row (`input.first.environment`), in `when:` and in every
  command position grid R's related-row column opens;
- a `when:` that reads a navigated row answers at step 4 of the precedence rule, after the
  `exists: false` branches over the rows it reads, so the `no-first` / `no-second` /
  `other-environment` order validates; declared above them it is refused as ESS-COMMAND-004 with the
  step-4 order; a navigated read with no `exists: false` branch for its row is
  `non_exhaustive_branches` naming the row (unless the reference is enforced, `ess/30`);
- an input typed as an identity type two entities share is refused as `type_mismatch`, its hint
  naming `E[input.<id>]` as the explicit form;
- `ess verify conform synthesize` witnesses `other-environment` with two arranged incidents in
  different environments and `compared` with two in the same one; `ess verify conform run
  --target interpreted` passes both and a target comparing one row with itself fails;
- below that format the refusals above stay, the hint of ESS-COMMAND-003 naming the format;
- generated Rust and Go keep the command an obligation naming the path; Entity Runtime refuses it
  as it refuses every `when_related:`.

## Scope

- inferred: crates/specify/ess-domain/src/expression.rs (an identity-typed input as a navigation root)
- inferred: crates/specify/ess-domain/src/command.rs (`when:` reads, around the ESS-COMMAND-003 path check)
- inferred: crates/specify/ess-domain/src/command/precedence.rs (a `when:` reading rows placed at step 4)
- inferred: crates/specify/ess-primitives/src/predicate.rs (a navigated operand)
- inferred: crates/specify/ess-compiler/src/ir.rs
- inferred: crates/verify/ess-conformance/src/interpret/execute.rs
- inferred: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- inferred: crates/specify/ess-domain/src/system.rs
- cited: docs/design/generic-language-features.md (§ G2 navigation; a decided subsection for navigation in guard predicates before code)
- inferred: docs/design/language-grids.md (grid R row `when:`, related-row column)
- inferred: website/docs/reference/predicates.md
- inferred: CHANGELOG.md
confidence: low

## Fit review

1. **Need.** A refusal decided by comparing a field of one named row with the same field of
   another. Reproduction above (run.txt § G). Requester's shape (downstream finding 11,
   2026-10-19): `E[input.a].direction == E[input.b].direction`.
2. **Class.** Gap for open types; convenience for a finite enum (one branch per value with
   `when_related: {all: [...]}`, `generic-language-features.md:248-252`).
3. **Already expressible?** For an enum, yes, one branch per value; for a `String`, `Integer` or
   identity field, no. The per-value idiom grows with the enum and moves no figure out of the
   specification, but does not exist for open types, so policy 1 does not decline it.
4. **Fit.** G2 decides navigation as the construct: "`subject.incident.state`,
   `input.agent_id.label`, `E[input.id]` for a row by identity"
   (`generic-language-features.md:236-237`), and grid R's target opens the related-row column in
   every command position (`docs/design/language-grids.md` Target). The first cut kept "navigation
   in guard predicates" out (`generic-language-features.md:248-250`): a cut, not a refusal, so no
   person must decide it. The G4 rule places the branch by what it reads (step 4), which also
   settles the order shown in the reproduction. Spelling: `input.<id>.<field>`, G2's own example,
   needs no new lexical form; `E[…]` stays for an identity type two entities share. Targets:
   interpreter and synthesis; Rust and Go obligations, named; Entity Runtime refusal kept.
5. **Second adopter.** A transfer only between two accounts of one currency; a merge only of two
   tickets on one queue.
6. **Cost.** `ess/37`; no new key; one placement rule in the precedence plan (a
   `when:` reading rows is step 4); no new diagnostic code.
7. **Alternatives.** (a) Keep it deferred: the design already decides navigation; deferral has no
   open question to wait on. (b) `E[input.a].x` everywhere: a new lexical form where G2's
   `input.a.x` suffices. (c) Chosen: navigation through identity-typed inputs in `when:`, placed at
   step 4, `E[…]` only where the type is ambiguous.

## Decisions

**Accept, redesigned.** The deferral is lifted: G2 decides navigation in guard predicates; only
the first cut left it out. The requester's `E[input.a].x` becomes `input.a.x` (G2's
`input.agent_id.label`), and a `when:` reading a navigated row answers at step 4. From G2's next
source format, after `story:navigation-reaches-any-depth-in-guards`.

## Source

github:beyond10x/ess#564 (downstream finding 11)

