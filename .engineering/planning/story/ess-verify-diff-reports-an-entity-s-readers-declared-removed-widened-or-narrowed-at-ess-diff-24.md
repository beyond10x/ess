---
format: aep.planning-md/3
id: story:ess-verify-diff-reports-an-entity-s-readers-declared-removed-widened-or-narrowed-at-ess-diff-24
kind: story
status: draft
title: ess verify diff reports an entity's readers declared, removed, widened or narrowed, at ess-diff/24
tags:
- gap-p2
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:an-entity-names-the-commands-whose-decisions-may-read-it-readers-and-validation-refuses-every-other-read-by
revision: 2
---
## Need

`g16-entity-readers` adds `readers:` to an entity and carries it in the IR. A change to that claim
is a change to what the specification promises. Its reader is anyone who relies on "no other
command reads this", and `ess verify diff` would report the change only through its residual,
`unclassified-changed` (`crates/verify/ess-diff/src/diff.rs:2483`, the residual's comment), or
miss it. The fit-review rule "every target handles it or refuses it by name" covers `ess verify
diff` (`.agents/skills/assessing-external-requests/SKILL.md:57-58`).

`ess-diff/24` is open with no change kind of its own: it is read and written wherever
`ess-diff/23` is, and no delta is written at it until a kind needs it
(`website/docs/reference/spec-versions.md:608`). So the kinds land there, and no diff version
opens.

## Outcome

`EntityChange` gains `ReadersDeclared {after}`, `ReadersRemoved {before}` and
`ReadersChanged {before, after}`, each side the sorted list of names. A changed list is
classified `narrowed` when the later list is a subset of the earlier one, `expanded` when it is a
superset, and `changed` otherwise. A delta carrying one of these is written at `ess-diff/24`, and
a reader of `ess-diff/23` refuses it with `unsupported_format_version`.

## Acceptance

- Red first: `crates/verify/ess-diff/tests/entity_readers.rs` diffs two revisions of the
  `g16-entity-readers` reproduction, without `readers:` and with `readers: []`. It expects one
  `readers-declared` change on `demo.review.Note`, written at `ess-diff/24`. Today the delta holds
  `unclassified-changed` or nothing; the report says which.
- `readers: [a, b]` to `[a]` is `readers-changed` classified narrowed. `[a]` to `[a, b]` is
  expanded. `[a]` to `[b]` is changed. Removing the key is `readers-removed`.
- Compatibility: expanding or removing the claim is breaking for a consumer of the claim, and
  narrowing it or declaring it is compatible. A test pins one row per kind in the compatibility
  table.
- A delta with none of these kinds keeps its earlier format and bytes. The `ess-diff/24` row of
  `spec-versions.md` and `formats.md` names the kinds.

## Fit review

Tree `bcfc51555`. Report item 16, third part. See `g16-entity-readers` § Fit review.

6. **Cost.** Three change kinds in an open diff version, and three compatibility rows. No new
   version. Size S.

## Decisions

**Accept, redesigned** (`P2B-D3` (a)). Open question:

- **D1 (decided: a, 2026-11-24). Is widening breaking?** (a) Widening and removing are breaking: the claim weakens. (b)
  Every change is reported as compatible, because it is the model's own property. **Recommended:
  (a).** A consumer that relied on the claim loses it.

## Scope

- cited: `crates/verify/ess-diff/src/change.rs` (`EntityChange` 2135)
- inferred: `crates/verify/ess-diff/src/diff.rs` (the entity comparison), `compatibility.rs`
  (rows), `delta.rs` (format selection)
- inferred: `crates/verify/ess-diff/tests/entity_readers.rs` (new)
- cited: `website/docs/reference/spec-versions.md` (`ess-diff/24` row), `website/docs/reference/formats.md`
- Size: S (about 150 changed lines).

## Depends on

- `g16-entity-readers`

## Source

A downstream application, 2026-11-24, report item 16.
