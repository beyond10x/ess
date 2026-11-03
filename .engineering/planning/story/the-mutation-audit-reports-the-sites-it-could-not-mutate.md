---
format: aep.planning-md/3
id: story:the-mutation-audit-reports-the-sites-it-could-not-mutate
kind: story
status: implemented
title: The mutation audit reports the sites it could not mutate
summary: transition-to, emit-drop and sets-drop leave common shapes without a live mutant, and the audit still exits 0 without saying how much it covered (#711)
tags:
- ess-batch-7
- ess-next
refs:
- provider: github
  reference: beyond10x/ess#711
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/mutate_cli.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/mutate.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/mutation_audit.rs
revision: 15
transitions:
- {from: "draft", to: "proposed", at: "2026-11-03T05:57:41Z", actor: "human:timo", revision: 12}
- {from: "proposed", to: "active", at: "2026-11-03T05:57:41Z", actor: "human:timo", revision: 13}
- {from: "active", to: "implemented", at: "2026-11-03T10:46:55Z", actor: "human:timo", revision: 15, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
<!-- Starting point for a `story` artifact, seeded by `aep artifact new story <name>`.
     No frontmatter here on purpose: the `---` block is written by the CLI from the id, kind, status
     and relations you gave it, and a second copy in this file would be the one that went stale.
     Delete the italic guidance as you fill each section. -->

# Story: <name>

## Outcome

*What is true for whom once this has shipped, in one sentence. If it names a component rather than a
person, it is a task — say what changes for someone.*

## Context

*Why this is worth doing now, and what it depends on. Link the epic or specification it comes from
rather than restating it; the `derived_from` relation already carries the edge.*

## Acceptance

- The mutation report and manifest list every site that a selected class excluded or left without a
  live mutant, with its reason, beside the stillborn mutants. The text summary gives a coverage
  count: sites with a scored mutant out of sites considered.
- A write on a creating branch, or to an `Optional` field, that no class mutates gets a live mutant
  where one exists: sets-retarget to a literal, or sets-drop of an `Optional` field to absent. A
  transition-to on a linear lifecycle retargets to a state the model admits, or is listed as
  unavailable.
- Over the toy spec of #711, the report names each such site.
- The default exit status is unchanged (D135). An opt-in flag makes a site without a live mutant
  block exit 0, as an unavailable emit-swap site does.
- Today's killed and survived mutant ids are unchanged.

## Out of Scope

*What a reasonable reader would expect to be included and is not — the boundary that stops this
story quietly becoming an epic.*

## Ambiguities

*Each gap this story found and did not close, classified. `inferable` — the answer is already
written down, so give the `path:line` or the artifact id that settles it.
`requires-stakeholder-input` — nobody here can decide it, so name who does, and raise that entry as
a `decision-blocker` with a `blocks` edge to this story, or it is a sentence somebody improvises
later.*

## Open Questions

*Anything still undecided belongs in `## Ambiguities` above, classified and with its citation or its
decider. Keep this section for a question that is neither — an unowned question is a story that
stalls without anybody noticing.*

## Problem

beyond10x/ess#711, a mutation-audit gap reported 2026-10-30 by a downstream session on ESS 0.77.0.
Re-checked with `ess 0.77.0` on a toy linear lifecycle:
- transition-to: 1 mutant, 1 stillborn (ESS-ENTITY-011).
- emit-drop: 3 mutants, 3 stillborn (ESS-COMMAND-007).
- sets-drop: ESS-MUTATE-003, with `creates_row` and `optional_field` notes on stderr only.
- The full audit exits 0 with 6 of 12 mutants stillborn.

The operators behave as documented. The gap is what the audit reports. Stillborn mutants and
excluded `sets` sites do not count against the result, and the excluded sites are absent from the
report and manifest. An emit-swap site with no alternative, by contrast, is `unavailable` and blocks
exit 0.

## Scope

Derived 2026-11-03 by `aep:story-scoper` for wave W2 on b89faaa41. Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).

- **Primary surface:** `crates/verify/ess-conformance/src/mutate.rs` (the mutation-audit module) — cited: classes at `mutate.rs:101-142`, transition sites at `:1598`, sets-drop at `:1160`
- **Files:** `crates/edge/ess-cli/src/main.rs:958-1004` (the `mutate` flags) and `:4717-4732` (exit status) — cited
- **Symbols:** `sets_drop_site` (:1160), `unmutated_sets` (:1185), `transition_sites` (:1598), `UnavailableSite` (:1360), `MutationReport::unaudited_sites` (:3581), `REPORT_FORMAT_7` (:92), `MANIFEST_FORMAT_6` (:4997) — cited
- **Also likely:** `crates/verify/ess-conformance/src/synthesize.rs` — inferred: a literal retarget may need synthesis to send a value that separates the literal from the row's (sibling-retarget handling at `:12426-12476`)
- **Also likely:** `crates/verify/ess-conformance/tests/mutation_audit.rs`, `crates/edge/ess-cli/tests/mutate_cli.rs` — inferred: both pin today's transition-to stillborn verdicts and report bytes
- **Documents:** `website/docs/reference/formats.md`, `website/docs/reference/spec-versions.md` — inferred: a new report or manifest version is written in both
- **Confidence:** high for the surface; medium for the synthesis and test files
- **Would collide with:** any unit touching the audit module's enumeration, its report/manifest members or `UnavailableSite`; `mutate`'s flags or exit mapping in `ess-cli` `main.rs`; sibling-retarget synthesis in `synthesize.rs` — inferred
- **Safety fact:** new sites only add mutant ids and remove none, because `transition_sites` and `outcome_sites` only push (`mutate.rs:1598-1617, 1692-1701`) — inferred
- **Already in the tree (partly):** sets-drop reasons computed once per entry (`mutate.rs:1070-1084, 1185-1216`), printed only as stderr notes (`main.rs:4517-4523`); emit-swap unavailable sites already block exit 0 (`mutate.rs:1355-1432`, `main.rs:4725-4728`). Absent: a persisted list of excluded sites, a transition-to alternative search, sets-retarget to a literal, the opt-in exit flag, a coverage count — cited
- **Not established:** sets-drop of an `Optional` field to absent is excluded today because no synthesized row states an absence (`mutate.rs:1218-1222`); transition-to on a linear lifecycle probably reduces to "listed as unavailable" (`ESS-ENTITY-011`, `resolve.rs:148`); the literal per type; what "sites considered" counts; the flag name; whether the listing is written only when non-empty (D135, the `/4` pattern at `mutate.rs:4958-4968`); the #711 toy spec is not in the tree.
- **Size:** large, about 700-950 hand-written lines; new report and manifest versions — inferred
- **Targeted run:** ess-conformance `--test mutation_audit`, `mutation_sets_entries`, `mutation_emit_swap`; ess-cli `--test mutate_cli`, `mutate_report_v2` — inferred

## Delivered

W2 (D408), 2026-11-03: closes beyond10x/ess#711. Each site a selected class excluded or left without a live mutant is listed with its reason under `unmutated_sites` in the text, `ess-mutation-report/8` and `ess-mutation-manifest/7`; the summary counts the scored sites; `--fail-on-unmutated` makes such a site block exit 0. Unit ebed16afd6, ess-cli pins re-pinned in ad383e41a5; merged a48d077e02. Review round 1 (ee16051c22, 4b58a5ee95) lists an unobservable sets-retarget literal as `unobserved_field`, prints listed sites beside ESS-MUTATE-003, and lists grants the grant classes skip; merged 8dec7706dd. Known limit (D413): a literal on a field only an invariant names stays an equivalent survivor, tracked as backlog draft story:a-literal-on-a-field-only-an-invariant-names-is-not-a-survivor.
