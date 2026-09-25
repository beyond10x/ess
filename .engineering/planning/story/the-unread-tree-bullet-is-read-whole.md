---
format: aep.planning-md/2
id: story:the-unread-tree-bullet-is-read-whole
kind: story
status: draft
title: The unread-tree bullet is read whole
scope:
- confidence: cited
  path: crates/edge/ess-xtask/tests/host_paths.rs
- confidence: cited
  path: crates/edge/ess-xtask/tests/host_paths_lane/mod.rs
revision: 3
---
# The unread-tree bullet is read whole

`crates/edge/ess-xtask/tests/host_paths.rs` documents, tree by tree, which trees the host-path scan
does not read. `documented_unread_trees` parses those bullets. It has now been narrowed twice and
is still narrower than the claim its own documentation makes for it.

| Round | What the parse read | What it missed |
|---|---|---|
| original | the bullet's first backtick token | every number, every comparison, the verdict |
| wave 24 correction 1 | the tree name, the verdict, and two counts before the first colon | the five numbers after the colon, including the one that was false |

Measured at `host_paths_adversary_6.rs:83`, exit 101: the corrected bullet and a bullet carrying the
falsified superlative — whose reasons contradict its own head, `1 file and 2 lines` — both parse to
`[(".engineering/", true, 60, 32806)]`.

`host_paths.rs:1262` claims the widening addressed this: "every number in the bullet was unread
prose. One of those numbers was wrong." The number that was wrong is in the region still unread.

## Also in scope, from the same pass

- `host_paths.rs:1244` — `UNREAD_TREE_SECTION`'s doc says "every bullet between the anchors is read".
  False for any marker but `* `. A `- `-marked bullet is dropped without a word.
  `host_paths_adversary_6.rs:121`, exit 101. Markdown-legal, 6 instances elsewhere under `crates/`.
- `host_paths.rs:1244` — a continuation line whose trimmed text equals the closing anchor closes the
  section where it stands. Later bullets dropped, `claimed.is_empty()` does not fire, run green.
  `host_paths_adversary_6.rs:158`, exit 101.
- `host_paths.rs:1503` — the count comparison is inline in a `#[test]` body, so it is not a function,
  not in `TRANSCRIBED`, and no adversarial target can drive it. A weakened comparison there stays
  green. This is also why the synthetic fixtures asserting `1 files, 1 lines` in
  `host_paths_adversary_5.rs` survive unchallenged.
- `host_paths_adversary_5.rs:224` — `the_journal_holds_the_largest_share_…` is vacuous twice over.
  `unread_bullet`'s window ends at `host_paths.rs:61` and the phrase is at `:71`, so the `return`
  always fires; and the phrase is now a disavowal, so a reaching window would fire on a correct
  document. Delete it or make it measure what it names.

## Acceptance

A bullet whose reasons contradict its head, or whose reasons carry a number the repository refuses,
is refused by the lane. Every claim the section's documentation makes about what it reads is true of
what it reads.

## Scope

- `crates/edge/ess-xtask/tests/host_paths.rs` — `cited`
- `crates/edge/ess-xtask/tests/host_paths_lane/mod.rs` — `cited`
- `crates/edge/ess-xtask/tests/host_paths_adversary_5.rs` — `cited`, untracked
- `crates/edge/ess-xtask/tests/host_paths_adversary_6.rs` — `cited`, untracked, holds the three red cases

Work in progress lives on `impl/planning-store-carries-workstation-paths`, uncommitted.
