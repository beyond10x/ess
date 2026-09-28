# correctness-1

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
| branch | `integrate/correctness-1` |
| base | `46e367ab2` (origin/main, ESS 0.38.0 + #184) |
| worktree | managed `ess-wave-c1` → `~/.local/state/worktree/trees/b10x/ess/ess-wave-c1` |
| build dir | `~/.cache/b10x-target/ess-c1-int` |
| scratch | `~/.cache/ess-wave-c1/int` |

## Units

| unit | story | objective | scope | worktree (managed id) | branch | build dir | scratch | stage |
|---|---|---|---|---|---|---|---|---|
| onename | `one-name-held-by-two-kinds-is-refused-whether-or-not-it-converts` | vision:O2 | cited (high) | `ess-c1-onename` | `impl/one-name-two-kinds` | `~/.cache/b10x-target/ess-c1-onename` | `~/.cache/ess-wave-c1/onename` | proposed |
| charset | `a-field-constrained-by-a-charset-publishes-that-charset` | vision:O2 | cited + inferred (medium) | `ess-c1-charset` | `impl/charset-publishes` | `~/.cache/b10x-target/ess-c1-charset` | `~/.cache/ess-wave-c1/charset` | proposed |
| trailing | `a-wrong-trailing-key-guess-is-reported-as-a-line` | vision:O2 | cited (high) | `ess-c1-trailing` | `impl/trailing-key-guess` | `~/.cache/b10x-target/ess-c1-trailing` | `~/.cache/ess-wave-c1/trailing` | proposed |

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
