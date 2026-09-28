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
| docdrift | `generated-suite-docs-say-what-the-runner-does` | #186 | `ess-n-docdrift` | `impl/generated-docs-say-what-runs` | `~/.cache/b10x-target/ess-n-docdrift` | `~/.cache/ess-wave-c1/docdrift` | implementing |
| reader | `a-reader-side-conformance-admits-reader-widening` | #191 | `ess-n-reader` | `impl/reader-side-conformance` | `~/.cache/b10x-target/ess-n-reader` | `~/.cache/ess-wave-c1/reader` | implementing |

The full gate that was running on this branch (`fmt-check` 0, `clippy` 0, `test` in progress) was stopped at 99% disk use (13G free); it runs once, on the whole shared branch, after every unit has merged. Its build dir was deleted (disk back to 64G).

Coordinator decisions for #191 are in the story body (per-entry `reader: true` in `ess-composition/3`; JSON only by structures that accept any JSON, so `Map<String, String>` stays drift, against the issue's example).
