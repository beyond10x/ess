---
format: aep.planning-md/3
id: story:predicate-reference-page
kind: story
status: implemented
title: Every predicate form ESS accepts is on one reference page
refs:
- provider: github
  reference: beyond10x/ess#92
relations:
- serves: vision:O2
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-09-25T21:39:16Z", actor: "human:timo", revision: 2, imported: true}
- {from: "proposed", to: "active", at: "2026-09-25T21:39:43Z", actor: "human:timo", revision: 3, imported: true}
- {from: "active", to: "implemented", at: "2026-10-02T09:41:02Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1,"verification":1}}, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
---
## Outcome

An adopter can look up every predicate form ESS accepts on one reference page, including where
synthesis cannot witness a form.

## Why

GitHub issue beyond10x/ess#92. The grammar is documented only in the Rust doc comment of
`crates/specify/ess-primitives/src/predicate.rs:15-66`; no page under `website/docs` lists
structured `all/any/not`, the map operators, quantifiers or `.count`. The agentplugins ESS skill
says structured forms are refused, which is false.

## Acceptance

- `website/docs/reference/predicates.md` (linked from the reference index/sidebar) lists: compact
  forms (comparison, bare path, `defined(p)`, `not …`), structured `all/any/not/none`, map
  operators (`eq, ne, lt, lte, gt, gte, any_of|in|one_of, none_of|not_in, exists|defined, truthy`,
  and the string operators of #95), quantifiers, `.count`, where each is accepted (`when`,
  `invariants`, view `filter`, `when_subject`), and what synthesis can and cannot witness after
  #93/#94.
- Every example on the page is checked by a test that runs it through validate (and synthesize
  where the page says it synthesizes), so the page cannot drift from the grammar.
- The agentplugins skill correction is recorded as a follow-up for that repository (issues are
  disabled there); this story does not edit it.

## Reconciliation 2026-10-02

ESS page/test delivery is released: commit4ff39cf22 and later corrections are contained in0.51.0. The reference covers compact/structured/map forms, quantifiers/count/string forms, accepted sites and synthesis limits. ess-cli/tests/predicate_reference_page.rs:221 actually invokes validation/synthesis; :315/:335/:363/:469/:508 checks navigation, examples and grammar coverage. Retained a2-scratch/r2-before.log detects stale examples after93/94; r2-after.log has two suites of5passing cases. This is actual drift-control evidence, not a claim about original missing-page chronology.

The cross-repository follow-up is also implemented, not missing: beyond10x/agentplugins commit b38f8d9cd13f149251282cc653ff029f1f7f4e5b explicitly cites ess#92 expectation2 and corrects the ESS syntax reference to list structured predicates. Exact remote main3910ec52e6344d6f77c40848959c78168bdc235b was read via GitHub: plugins/ess/skills/specifying/references/syntax.md:330-355 documents compact and structured forms, links this reference, lists all/any/none/not, maps, quantifiers and count, and distinguishes validation from witness support. This supersedes the initial local-checkout observation at stale agentplugins2f82cbc.

The required follow-up is recorded here with its actual delivery commit: https://github.com/beyond10x/agentplugins/commit/b38f8d9cd13f149251282cc653ff029f1f7f4e5b. No new agentplugins task or source edit is needed for this acceptance.
