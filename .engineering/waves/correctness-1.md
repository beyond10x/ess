# correctness-1 (shared branch integrate/ess-next)

First wave of the ESS correctness and expressiveness plan of 2026-09-28. Operator, 2026-09-28:
"figure out what we should do next in ESS in terms of correctness and spec expressiveness", then
`/aep:wave`. Runs beside the concurrent-history work of session `ess-formal-advancement`
(`integrate/concurrent-history`, unpublished at proposal time), so the wave stays out of
`crates/verify/ess-conformance` and `crates/edge/ess-cli`.

Skill: aep 0.16.0 (`aep:implementing`, wave mode). Dispatch types: `aep:story-scoper` (done),
`aep:implementor`, `aep:adversary`. N = 3.

## Commits this approval authorises

- One opening store commit on `integrate/correctness-1`: this page, the re-derived `## Scope`
  sections and typed scope entries, a `serves: vision:O2` edge where missing, and the
  draft → proposed → active moves of the three units.
- One commit per unit (plus correction commits for adversary findings) on its unit branch.
- The merges of unit branches into `integrate/correctness-1`.
- One closing store commit (evidence, `implemented` moves, scope rewrite).
- Publishing `integrate/correctness-1` through the bot, one PR to `main`, and its bot merge once CI
  is green.

Nothing else: no tag, no version bump, no release.

## Integration

| | |
|---|---|
| branch | `integrate/ess-next` (was `integrate/correctness-1`) |
| base | `46e367ab2` (origin/main, ESS 0.38.0 + #184) |
| worktree | managed `ess-wave-c1` → `~/.local/state/worktree/trees/b10x/ess/ess-wave-c1` |
| build dir | `~/.cache/b10x-target/ess-c1-int` |
| scratch | `~/.cache/ess-wave-c1/int` |

## Units

| unit | story | objective | scope | worktree (managed id) | branch | build dir | scratch | stage |
|---|---|---|---|---|---|---|---|---|
| onename | `one-name-held-by-two-kinds-is-refused-whether-or-not-it-converts` | vision:O2 | cited (high) | `ess-c1-onename` | `impl/one-name-two-kinds` | `~/.cache/b10x-target/ess-c1-onename` | `~/.cache/ess-wave-c1/onename` | merged (86b5e89c2) |
| charset | `a-field-constrained-by-a-charset-publishes-that-charset` | vision:O2 | cited + inferred (medium) | `ess-c1-charset` | `impl/charset-publishes` | `~/.cache/b10x-target/ess-c1-charset` | `~/.cache/ess-wave-c1/charset` | adversary 2 (head 5519a581c) |
| trailing | `a-wrong-trailing-key-guess-is-reported-as-a-line` | vision:O2 | cited (high) | `ess-c1-trailing` | `impl/trailing-key-guess` | `~/.cache/b10x-target/ess-c1-trailing` | `~/.cache/ess-wave-c1/trailing` | merged (b0c89d384) |

Acceptance for `onename` and `trailing` is an existing `#[ignore]`d test that fails on
`46e367ab2` (scoper runs, 2026-09-28): `masked_declaration_boundaries.rs:119-123,178-181`,
`trailing_key_guess_citations.rs:215`.

## Selection

`aep plan artifact waves --kind story --status draft` (aep 0.63.1), after the scope entries above
were written: `one-name…`, `a-field-constrained…` and `a-wrong-trailing…` are all in wave 1 with no
collision among them. Full output: `~/.cache/ess-wave-c1/waves-draft.txt` (10 waves, 94 collisions,
13 unassessed).

Left out:

| story | why |
|---|---|
| `go-normalization-pattern-semantics` | partially done; the rest needs a bounded ECMA-262 matcher design nobody has made (scope: "matcher choice and limit policy are undecided") — not finishable inside a wave |
| `a-refusal-records-the-document-it-was-read-from` | verb wave 2: collides with `one-name…` (`spec.rs`, `system.rs`, `locator_citations.rs`, inferred) and with `a-wrong-trailing…` (`resolve.rs`, cited) — next wave |
| `a-branch-may-clear-the-field-it-owns` | code shipped in `ce2197efb` (0.26.0); only a docs row remains, routed to the public-docs overhaul (`docs/public-docs-overhaul`) |
| `outcome-decided-by-environment` | collides with `a-field-constrained…` on `command.rs` (inferred) |

## Pre-flight (2026-09-28)

| check | value |
|---|---|
| free disk `/` | 26G (floor 10G); `b10x-target/ess-chc-*` (another session) holds 87G and is growing |
| compiler cache | sccache 15 GiB of 30 GiB; `RUSTC_WRAPPER` unset in the shell — set per unit |
| previous wave trees | none of this wave's; 14 linked worktrees belong to other sessions |
| one measured build | not yet measured; measured on the first unit and recorded here |

## Decisions taken by the coordinator

- Units gate package-scoped (`ess-domain`, `ess-compiler`); the whole gate runs once on the
  integration branch after unit build dirs are deleted, because of disk.
- `CHANGELOG.md` and `schemas/generated/ess.schema.json` regeneration are coordinator-owned at
  integration if two units need them; `charset` alone regenerates the schema.

## Log
- 12:22 trailing: implementor green (ess-compiler executed 208→210), committed 08241ecc6; adversary pass 1 dispatched. Coordinator-owned follow-ups: ~/.cache/ess-wave-c1/trailing/tests.patch (locator_citations.rs doc), design-doc.md, changelog.md.
- 12:29 trailing: adversary pass 1 red (3 findings: 2 introduced, 1 pre-existing), recorded review-result:adversary-c1-trailing-pass-1; back to the same implementor (correction round 1).
- 12:29 charset: implementor green (ess-domain 920→924); coordinator applied its topology.rs patch (RawTopology.workloads keys) + test, schema regenerated (`cargo xtask schema --check`: current, 5/5 published_charsets); committed 4c369054f; adversary pass 1 dispatched.
- 12:29 onename: implementor green (ess-domain 920→926, ess-compiler 208→208); committed 8c34e8a77; adversary pass 1 dispatched. Noted, not in scope: two sound same-kind duplicates are already reported twice (declare + Assembly::claim) — candidate story.
- 12:33 trailing: correction 1 green (ess-compiler 210→215, 4 adversary cases red→green), committed 4602cc32c; 3 outcomes fixed recorded; adversary pass 2 dispatched.
- 12:35 onename: adversary pass 1 red (F1, F2 pre-existing but inside the story's claim — taken into the unit by the coordinator; F3 introduced note), recorded review-result:adversary-c1-onename-pass-1; correction round 1 to the same implementor.
- 12:40 charset: adversary pass 1 red (F1 introduced: xtask schema-metadata pin stale, 10 ess-xtask cases red; F2/F3 story class: Transition.name, Selection/SelectionInput.name; F4 alias not published → filed story:the-published-schema-admits-the-name-aliases-the-parser-reads, outcome no-op); recorded review-result:adversary-c1-charset-pass-1; correction round 1 (assignment widened to reviewed-schema-metadata.json, entity.rs, selection.rs).
- 12:42 onename: correction 1 green (ess-domain 926→936, 9 adversary cases green), committed 8ce4a54c2; 3 outcomes fixed; adversary pass 2 dispatched.
- 12:43 trailing: adversary pass 2 red — ledger: carried 0, new 3, resolved 3 (pass 1: 3 → pass 2: 3); recorded review-result:adversary-c1-trailing-pass-2; correction 2 to the same implementor, verified by the coordinator's diff read (no third attack).
- 12:48 trailing: correction 2 green (ess-compiler 215→220); coordinator read the diff (test-file change is doc-only; no assertion dropped); VERIFIED (acceptance test red on base, green on c1ad14182); merged into integrate/correctness-1 as b0c89d384.
- 12:48 charset: correction 1 green (ess-domain 929→934, ess-xtask consumer_coverage 118→128), committed 5519a581c; 3 outcomes fixed (+1 no-op for F4); adversary pass 2 dispatched.
- 12:50 onename: adversary pass 2 red (N1, N2, N4 introduced; N3 pre-existing note) — pass 1: 3 → pass 2: 4, carried 0; recorded review-result:adversary-c1-onename-pass-2; correction 2 (last), coordinator-verified. N4 routed: restore Collected order, change Assembly::claim order instead.
- 12:55 onename: correction 2 green (ess-domain 936→945); coordinator read the diff (tests additions only; misplaced refusal moved byte-identical); VERIFIED; merged as 86b5e89c2. N3 fixed in-unit, no story filed.

## Shared integration branch (operator, 2026-09-28)

"check if there are other issues on github for ESS - add these to the current wave - do integrate all upcoming fixes under one shared integration branch". `integrate/correctness-1` is renamed `integrate/ess-next`; `integrate/runtime-parity` is merged into it (`2e0960e7f`) and its go and ts units merge here too (page: `runtime-parity.md`). Open issues at that time: #186, #188, #191 — all three are in this branch's work.

| unit | story | issue | worktree (managed id) | branch | build dir | scratch | stage |
|---|---|---|---|---|---|---|---|
| docdrift | `generated-suite-docs-say-what-the-runner-does` | #186 | `ess-n-docdrift` | `impl/generated-docs-say-what-runs` | `~/.cache/b10x-target/ess-n-docdrift` | `~/.cache/ess-wave-c1/docdrift` | merged |
| reader | `a-reader-side-conformance-admits-reader-widening` | #191 | `ess-n-reader` | `impl/reader-side-conformance` | `~/.cache/b10x-target/ess-n-reader` | `~/.cache/ess-wave-c1/reader` | merged (380852f06) |

The full gate that was running on this branch (`fmt-check` 0, `clippy` 0, `test` in progress) was stopped at 99% disk use (13G free); it runs once, on the whole shared branch, after every unit has merged. Its build dir was deleted (disk back to 64G).

Coordinator decisions for #191 are in the story body (per-entry `reader: true` in `ess-composition/3`; JSON only by structures that accept any JSON, so `Map<String, String>` stays drift, against the issue's example).
- 13:50 reader: implementor green (ess-composition 43→57), committed a15c2a9cd; adversary pass 1 dispatched. Coordinator revises decision 5: Map<String, Json> admitted only for an object-shaped producer (struct, or map with String keys); a bare producer Json read as a map stays drift — routed with the adversary findings.
- 13:50 docdrift: implementor green except the 2 Outcome-comment cases that need its runtime-go/ts patches (applied at integration after go/ts merge); committed 86ce9f8cd; adversary pass 1 dispatched. Not done by the unit: website/docs/guides/verify-conformance.md:260-264 run commands lack ESS_REPORT_FORMAT=2 (coordinator at integration).
- 13:57 reader: adversary pass 1 red (9/12; F1 Json-as-map, F2 wire-name collision of extra fields — blockers; F3 /2 reader: null; F4 subset vs closed generated types; F5 docs), recorded review-result:adversary-n-reader-pass-1; story Design 1, 5, 6 revised; correction 1 to the same implementor.
- 13:59 docdrift: adversary pass 1 red (F1 duplicate Running it section, F2 hard-coded version sets break against the merged runtimes — blockers; F3 run instructions for refused versions; F4 guide; F5 help; F6 TS message). Recorded review-result:adversary-n-docdrift-pass-1. Correction 1 waits for the go unit to merge; the coordinator then merges integrate/ess-next into impl/generated-docs-say-what-runs so the fix is made against the merged runtimes.
- 14:06 reader: correction 1 green (ess-composition 69→70, adversary 12/12), committed d6d073d0d; 5 outcomes fixed; adversary pass 2 dispatched.
- 14:10 docdrift: Outcome-comment patches applied on integrate/ess-next (c4ffd7d36); integrate/ess-next merged into impl/generated-docs-say-what-runs (46cfd5023); correction 1 to the same implementor.
- 14:13 reader: adversary pass 2 red on 2 over-refusal notes only (no admission of a rejecting consumer) — pass 1: 5 → pass 2: 2; recorded review-result:adversary-n-reader-pass-2; both deferred to story:reader-conformance-over-refusals (outcome no-op); the two cases pinned to today's refusal before merge.
- 14:14 reader: two over-refusals pinned (ess-composition 70→85), committed 5994d2b00; VERIFIED; merged as 380852f06.
- 14:28 docdrift: correction 1 green (ess-conformance 1374 passed, 0 failed), committed ff62941c5; 6 outcomes fixed; adversary pass 2 dispatched. Behaviour change: emit refuses a suite version the emitted runtime does not admit. Left: ess-cli coverage.rs:160 message still says suite/8 and /9 (not in scope).
- 14:31 integrate/ess-next: stale 'runtimes refuse /22–/27' text fixed in 21 files; origin/main (0.39.0, #190) merged (67e5e19fe), conflicts in formats.md and spec-versions.md resolved keeping both sides; CHANGELOG Unreleased assembled (Added #191, Changed target surface, Fixed #188, #186, correctness-1). This release is 0.40.0.
- 14:38 docdrift: adversary pass 2 green; VERIFIED; merged into integrate/ess-next. All eight units merged; full gate next.
- 15:34 gate on a4b422e7e stopped at 100% disk after fmt-check 0 and clippy 0 (test step incomplete); build dir ess-next-int (56G) deleted. One final gate runs on the release candidate after #192 and wave 5 merge. Primary checkout: 2 stale store edits restored, 3 stashes dropped, 726M pre-migration .engineering/state and untracked .agents/ + one review doc removed — non-integrated content archived under ~/.local/state/worktree/archives/ess/primary-leftovers-20260928/; primary fast-forwarded to main 9966add71.
