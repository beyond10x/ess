---
format: aep.planning-md/3
id: story:error-swap-mutates-refusals-that-carry-a-payload
kind: story
status: implemented
title: error-swap mutates refusals that carry a payload
summary: error-swap keeps the original error's payload mapping, so every refusal with a payload is stillborn and never audited (#709)
tags:
- ess-batch-7
- ess-next
refs:
- provider: github
  reference: beyond10x/ess#709
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/mutate.rs
revision: 13
transitions:
- {from: "draft", to: "proposed", at: "2026-11-03T05:57:40Z", actor: "human:timo", revision: 9}
- {from: "proposed", to: "active", at: "2026-11-03T05:57:40Z", actor: "human:timo", revision: 10}
- {from: "active", to: "implemented", at: "2026-11-03T10:46:57Z", actor: "human:timo", revision: 13, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
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

- error-swap on an outcome with a `payload:` writes a mutant the model admits.
  - The payload is re-keyed to the swapped-in error where its fields can be filled the same way.
  - Otherwise the payload is dropped, where the swapped-in error has no fields.
  - Otherwise the site is listed as unavailable with a named reason, not as stillborn.
- Over the toy spec of #709, both payload-carrying refusals get a live mutant, and the synthesized
  suite kills it.
- An `unknown_instance` refusal gets a mutant that keeps it free of effects.
- Payload-less sites keep their mutant ids and verdicts.

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

beyond10x/ess#709, a mutation-audit gap reported 2026-10-30 by a downstream session on ESS 0.77.0.
Re-checked with `ess 0.77.0` on a toy spec: `mutate --target interpreted --class error-swap` gives 3
mutants, 1 killed and 2 stillborn, and exits 0.

error-swap replaces an outcome's `error:` but keeps its `payload:`, which is keyed by the original
error. The model then refuses the mutant: ESS-COMMAND-001 on a guarded or `wrong_state` refusal,
ESS-COMMAND-004 on an `unknown_instance` refusal. A hand-written mutant with the stale payload
removed validates and synthesizes. So every refusal that says what fills its error's fields has no
live error-swap mutant, and only refusals without a payload are audited.

## Scope

Derived 2026-11-03 by `aep:story-scoper` for wave W2 on b89faaa41. Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).

- **Primary surface:** `crates/verify/ess-conformance` (mutation audit, `src/mutate.rs`) — cited: the class string is at `mutate.rs:180`, the variant at `mutate.rs:426`
- **Files:** `crates/verify/ess-conformance/src/mutate.rs:2124-2131` — cited: the apply sets `found.error` and leaves `error_payload` alone; discovery at `mutate.rs:1743` pushes the site with no compile check
- **Symbols:** `Mutation::ErrorSwap`, `Outcome::error_payload` (`crates/specify/ess-domain/src/command.rs:2563`, read only) — cited
- **Also likely:** `crates/edge/ess-cli/src/main.rs` (help text at `:927` names the unavailable reasons), only if the unavailable branch adds a reason — inferred
- **Documents:** `docs/design/mutation-audit-and-model-runner.md` and `website/docs/reference/formats.md`, only if the report version moves — inferred
- **Confidence:** high
- **Would collide with:** any unit touching `mutate.rs` (emit-swap and unavailable-site code, report constants at `mutate.rs:72-92`) and the `mutate` report output in `ess-cli` — inferred
- **Safety fact:** payload-less error-swap sites are unchanged, because re-keying an empty `error_payload` is a no-op and the id stays `error-swap/<command>/<outcome>` — inferred
- **Not established:** the ESS-COMMAND-001/004 refusals on a stale payload come from the story, not a traced run; whether the billing `error-swap` count of 5 (`tests/mutation_audit.rs:330`) moves; the #709 toy spec is unplaced; "fields can be filled the same way" has no definition in the tree; whether `unknown_instance` needs an explicit effect-free check. ess-cli `mutate_cli` and `mutate_report_v2` also pin error-swap output.
- **Size:** medium, about 150 lines plus about 120 test lines; a new unavailable reason needs `ess-mutation-report/8` by the crate's convention (`REPORT_FORMAT_5`-`_7`, `mutate.rs:82-92`) — inferred
- **Targeted run:** ess-conformance `--test mutation_audit`, `mutation_emit_swap`, `mutation_authored`, `refusal_with_effect`, `mutation_unkillable`, plus one new; ess-cli `--test mutate_cli`, `mutate_report_v2` — inferred

## Resolution

Acceptance bullet 1c (drop the payload only where the swapped-in error has no fields, otherwise
list the site) cannot hold beside bullet 2 (both toy refusals get a live mutant): on the #709 toy
every candidate error has different, non-empty fields. Found by the W2 MUT1 adversary pass
(review-result:adversary-w2-mut1-pass-1, F6). Wave lead's decision, 2026-11-03: bullet 2 and the
issue's text stand as the acceptance. `error-swap` re-keys the `payload:` entry where both errors
have the same fields, drops it where they differ, takes the first error that compiles, and lists a
refusal none compiles for as `no_compatible_error_alternative` (W2 unit MUT1, `578fde2652`).

## Delivered

W2 (D408), 2026-11-03: closes beyond10x/ess#709. `error-swap` re-keys a refusal's `payload:` entry to the error swapped in (dropped where the fields differ) and takes the first error that compiles; where none compiles the site is listed as `no_compatible_error_alternative`. Unit 578fde2652; merged a48d077e02. Known limit: a payload-less refusal in a single-error domain is skipped silently, pinned in 07dff602dc and tracked as backlog draft story:an-error-swap-site-without-an-alternative-error-is-listed.
