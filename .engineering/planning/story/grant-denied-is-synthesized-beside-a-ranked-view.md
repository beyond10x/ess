---
format: aep.planning-md/3
id: story:grant-denied-is-synthesized-beside-a-ranked-view
kind: story
status: active
title: grant/denied is synthesized beside a ranked view
summary: 'Synthesize withholds grant/denied for a command on an existing row when a view of its entity has order_by: the grant template reads the scenario''s last send (#712)'
tags:
- ess-batch-5
- ess-next
refs:
- provider: github
  reference: beyond10x/ess#712
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/grant.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/grant_denied.rs
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-02T20:48:55Z", actor: "human:timo", revision: 8}
- {from: "proposed", to: "active", at: "2026-11-02T20:48:55Z", actor: "human:timo", revision: 9}
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

- Over both toy specs of #712, synthesize writes the moving command's `grant/denied` scenario, with or
  without `order_by`. The scenario passes the interpreted target and fails a target that skips the
  grant check.
- The grant template is the last send of the command under audit. Later sends that a view check
  appends are dropped from it, as `denied_from` already does.
- Suites that already had their `grant/denied` scenarios keep their bytes. The bytes-table rows that
  gain a scenario are named in the unit and re-pinned on purpose.

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

beyond10x/ess#712, a defect found 2026-10-30 while reproducing #710, on ESS 0.77.0. On a toy spec with
a creating command, a command that moves an existing row, and an actor granted nothing:

- With a view of the entity ranked by `order_by`, synthesize writes 6 scenarios. It notes that no
  scenario sends the moving command into a branch that accepts it, so its `grant/denied` is not
  synthesized, although its `outcome/...` scenario does exactly that.
- Without the `order_by` line, it writes 7 scenarios, `grant/denied` included.

Cause: `refused_from_template` in `synthesize/grant.rs` takes the scenario's last send of any
command. The `ranked` view check appends a second creating send after the command's own, so the
template is rejected and `denied` and `denied_since` push `GrantDeniedUnwitnessed`. This is not the
deliberate baseline drop of #710.

## Scope

Derived 2026-11-02 by `aep:story-scoper` for wave W1 on b81deac78. Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).

- **Primary surface:** `crates/verify/ess-conformance`, grant synthesis in `src/synthesize/grant.rs` — cited (the story names `refused_from_template`).
- **Files:** `crates/verify/ess-conformance/src/synthesize/grant.rs:543-546` (`accepted_send` takes the last send of the scenario and rejects it when that send is another command) — cited.
- **Symbols:** `refused_from_template` (grant.rs:530), `denied_from` (grant.rs:594), `denied` (grant.rs:47), `denied_since` (grant.rs:109) — cited.
- **Also likely:** `crates/verify/ess-conformance/tests/grant_denied.rs` for the two toy specs — inferred.
- **Documents:** none; a `CHANGELOG.md` line — inferred.
- **Generated:** `tests/fixtures/synthesis-suite-bytes-table.tsv` re-pinned for each suite that gains a `grant/denied` (coordinator-merged shared file) — inferred.
- **Confidence:** medium — the cause and the rejecting line are cited; the fix point (`accepted_send`, not `denied_from`) and byte stability of existing suites are inferred.
- **Would collide with:** any unit touching grant-template selection in `grant.rs` (`refused_from_template`, reached from `denied_since` via `synthesize/caller.rs:1000`), and any unit re-pinning `synthesis-suite-bytes-table.tsv` — cited.
- **Safety fact:** `denied_from` drops every step after the template's send except unchanged-view claims (grant.rs:617-634) — cited.
- **Unproven:** widening the template pool leaves the template an existing `grant/denied` uses unchanged (sort at grant.rs:579-582) — inferred.
- **Size:** small, about 15 lines in grant.rs and 60-100 lines of tests; no suite format change (`COVERAGE_SUITE_FORMAT = "ess-conformance/5"`, coverage.rs:15) — inferred.
- **ess-conformance targets:** `grant_denied`, `adversary_j_pass1`, `caller_arranged_rows`, `synthesis_suite_bytes_table` — inferred.
