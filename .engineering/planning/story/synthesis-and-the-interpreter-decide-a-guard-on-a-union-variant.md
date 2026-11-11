---
format: aep.planning-md/3
id: story:synthesis-and-the-interpreter-decide-a-guard-on-a-union-variant
kind: story
status: active
title: Synthesis and the interpreter decide a guard on a union variant
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#637
- provider: github
  reference: beyond10x/ess#764
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:a-guard-reads-a-union-tag-and-a-union-names-its-content-key
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/specify/ess-primitives/src/facts.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret/facts.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/mutate.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/union_variant_guards.rs
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:46:21Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:46:23Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

Once a guard may compare a union with variant labels
(`story:a-guard-reads-a-union-tag-and-a-union-names-its-content-key`), the suite must send one
witness per guarded variant, and the interpreted target must read a union value's variant label as
the fact the guard compares. Today a union is witnessed by its first label only, unless it has a unit
variant (`docs/design/union-unit-variants.md:82`, lane "conformance synthesis"), so a branch guarded
on any other variant would get no scenario.

Minimal model: `example.g.Kind = text | web_crawl(example.g.CrawlConfig)`, tag `type`. Command
`example.g.Add(kind: Kind)` has `fetched` guarded `when: kind in [web_crawl]` and `stored`
otherwise.

## Acceptance

- `ess verify conform synthesize` on the minimal model reports 0 refusals. It writes
  `example.g.Add/outcome/fetched`, sending `{"type": "web_crawl", "value": {"url": …}}`, and
  `example.g.Add/outcome/stored`, sending `{"type": "text"}`.
- The same model with the guard over a stored union field (`when_subject:`) and in a view `filter:`
  synthesizes one arranged row per variant, with each row on the side of the guard its variant puts
  it.
- `ess verify conform run --target interpreted` passes every scenario. A mutant guard with the
  label list flipped (`[text]`) fails `fetched`, and the mutation report lists the union guard among
  the boundary mutations.

## Scope

- inferred: crates/verify/ess-conformance/src/witness.rs
- inferred: crates/verify/ess-conformance/src/synthesize.rs
- inferred: crates/verify/ess-conformance/src/mutate.rs
- inferred: crates/verify/ess-conformance/src/interpret/facts.rs
- inferred: crates/specify/ess-primitives/src/facts.rs
- inferred: crates/verify/ess-conformance/tests/union_variant_guards.rs
confidence: low

## Fit review

1. **Need.** A guard on a union's variant must be witnessed from both sides, or a target that
   ignores the variant passes. Reproduction: the parent story's model. It does not validate today
   (triage run, `guard`, ESS-COMMAND-002).
2. **Class.** Gap, following the parent story.
3. **Already expressible?** Not applicable: the guard does not validate today.
4. **Fit.** Grid O, union column. Reuses the enum witness, one label per side of a membership
   guard, and the per-variant run that unit variants already have (`union-unit-variants.md:82`). It
   adds no scenario id family.
5. **Second adopter.** The payment-method example of the parent story.
6. **Cost.** No source format. No suite format: scenarios carry union values as they do today.
   Suites of models without a union guard keep their bytes.
7. **Alternatives.** (a) Require an authored scenario per variant: rejected, it leaves the guard
   unchecked by default. (b) Witness every variant of every union input: rejected, it multiplies
   scenarios where no guard reads the variant. (c) Witness the labels the guard names, plus one
   outside the list: chosen, as for an enum.

## Decisions

accept as proposed, as the parent story. This story carries synthesis, mutation and the
interpreted target.

## Source

github:beyond10x/ess#637
