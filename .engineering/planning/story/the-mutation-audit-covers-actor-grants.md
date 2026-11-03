---
format: aep.planning-md/3
id: story:the-mutation-audit-covers-actor-grants
kind: story
status: implemented
title: The mutation audit covers actor grants
summary: The mutate baseline drops the grant/denied scenarios and no class mutates may:, so a wrong grant table is invisible to the audit (#710)
tags:
- ess-batch-7
- ess-next
refs:
- provider: github
  reference: beyond10x/ess#710
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
  path: crates/verify/ess-conformance/src/synthesize/grant.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/mutation_component.rs
revision: 16
transitions:
- {from: "draft", to: "proposed", at: "2026-11-03T05:57:42Z", actor: "human:timo", revision: 13}
- {from: "proposed", to: "active", at: "2026-11-03T05:57:42Z", actor: "human:timo", revision: 14}
- {from: "active", to: "implemented", at: "2026-11-03T10:46:58Z", actor: "human:timo", revision: 16, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
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

- A grant mutation class exists:
  - grant-drop removes one command from one actor's `may:`;
  - grant-add adds one command an actor may not invoke, where a command the component accepts is
    missing from that actor's `may:`.
- The baseline keeps the `grant/denied` scenarios synthesize writes.
- Over the toy spec of #710, `mutate --target interpreted` kills the grant mutants. The baseline
  count equals the `synthesize` scenario count.
- A run with `--class` that excludes the grant classes keeps today's baseline and mutant ids. The
  manifest format moves to a new version only if it must name the new class.

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

beyond10x/ess#710, a mutation-audit gap reported 2026-10-30 by a downstream session on ESS 0.77.0.
Re-checked with `ess 0.77.0` on a toy spec:
- `synthesize` writes 2 scenarios, including `<command>/grant/denied`.
- The `mutate --emit` baseline holds 1 scenario, and the output does not mention the drop.
- None of the twelve classes edits an actor's `may:`.

The drop is deliberate (the comment above `synthesized` in `mutate.rs`, citing #265): grant scenarios
could never kill a mutant, because no class alters a grant. So the audit cannot detect a grant table
that is too wide or too narrow.

## Scope

Derived 2026-11-03 by `aep:story-scoper` for wave W2 on b89faaa41. Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).

- **Primary surface:** the mutation audit, `crates/verify/ess-conformance/src/mutate.rs` — cited
- **Files:** `mutate.rs:96-159` (the twelve classes and `MutantClass::ALL`, none of them grant); `mutate.rs:5649-5650` (`synthesized_from` drops `ScenarioId::Grant` from the baseline); `crates/edge/ess-cli/src/main.rs:1080-1113` (the `MutateClass` clap mirror and its exhaustive `From` impl) — cited
- **Symbols:** `MutantClass`, `Mutation`, `apply`, `selection`, `synthesized_from`, `ScenarioId::Grant` — cited
- **Also likely:** `crates/verify/ess-conformance/src/synthesize.rs:12283` (a `with_dropped_write`-style hook, so a grant mutant's suite is synthesized against the baseline's grant table) and `crates/verify/ess-conformance/src/synthesize/grant.rs` — inferred
- **Also likely:** `crates/verify/ess-conformance/tests/mutation_component.rs`, `crates/edge/ess-cli/tests/mutate_cli.rs` — inferred
- **Documents:** `website/docs/guides/verify/mutation-audit.md:23` ("The twelve classes"), `website/docs/reference/formats.md:410-416` — cited
- **Confidence:** medium — the defect sites are known; how a grant mutant is killed is inferred (a mutant's own suite is synthesized from the mutant's model, `mutate.rs:5792-5796`)
- **Would collide with:** any unit on `mutate.rs` (class enum, `Mutation`, `apply`, `selection`, manifest and report constants); `story:the-mutation-audit-reports-the-sites-it-could-not-mutate` is in the same unit (MUT1) — inferred
- **Safety fact:** `ScenarioId::Grant` is removed from the baseline only at `mutate.rs:5650`; a grant mutant's refusal is observable because `interpret/command.rs:15-22` refuses by `may:` — cited
- **Byte pins outside the module:** `crates/generate/ess-synth/tests/adversary_e_u2_bytes.rs:66`, `adversary_e_u6_bytes.rs:89` use `MutantClass::ALL`; `mutate_cli.rs:258` and `main.rs:6083` pin the class list; `mutation_emit_swap.rs:941` asserts `ALL.last() == EmitSwap` — cited
- **Not established:** the #710 toy spec is not in the tree; whether the report format moves (the story names only the manifest); the stale "report/1 evidence" comment at `mutate.rs:5620-5624`.
- **Size:** large, about 500-700 hand-written lines; `ess-mutation-manifest/7` (newest `/6`, `mutate.rs:4997`), probably `ess-mutation-report/8`; no suite major (`grant::ORDINARY = 26`) — inferred
- **Targeted run:** ess-conformance `--test mutation_audit`, `mutation_component`, `mutation_arms`, `mutation_emit_swap`, `mutation_unkillable`, `mutation_known_failures`, `mutation_external`, `adversary_mutate_pass1`, `adversary_mutate_pass2`, `adversary_caller_digest_pass1`, `adversary_244b_window_free_bytes`, `adversary_known_failures_pass1` (plus `grant_denied`, `adversary_265_pass1`, `adversary_265_pass2` if `synthesize/grant.rs` changes); ess-synth `--test adversary_e_u2_bytes`, `adversary_e_u6_bytes`; ess-cli `--test mutate_cli` — inferred

## Delivered

W2 (D408), 2026-11-03: closes beyond10x/ess#710. Classes `grant-drop` and `grant-add`; where either is selected the baseline and every mutant's suite keep their `<command>/grant/denied` scenarios and a grant-drop refusal is sent as the actor that lost the grant; a selection without a grant class keeps its bytes. Unit ad383e41a5 (finished by the wave lead after the implementor stalled); merged a48d077e02. Review round 1 (ee16051c22): a grant-add where an attributed actor holds the command is sent as the gaining actor (`with_added_grant`) or listed as `attributed_holder`, and every other `may:` entry no class mutates is listed (`system_actor`, `command_not_served`); merged 8dec7706dd. Full gate stage B: the where-this-stands mutation row lists both classes, b95e07bc02.
