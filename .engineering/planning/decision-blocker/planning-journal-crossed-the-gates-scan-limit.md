---
format: aep.planning-md/2
id: decision-blocker:planning-journal-crossed-the-gates-scan-limit
kind: decision-blocker
status: cleared
title: The planning journal crossed the gates scan limit, and nothing can be pushed
relations:
- blocks: initiative:ess-evolution
revision: 4
---
## What is blocked

Every push to `github.com/beyond10x/ess`, including the 0.24.0 release. 64 commits are held
locally, and `b10x-gates` refuses to sign a receipt for any candidate that contains commit
`21fd733e` or anything after it.

## The measurement

The limit is **per changed blob**, and it is roughly **8 MiB**. Measured by pushing one synthetic
commit per size against `origin/main`, each changing `.engineering/planning/journal.jsonl` and
nothing else, and reading `b10x-gates check`'s own answer:

| journal bytes | `b10x-gates check` |
|---|---|
| 1,582,109 | signed receipt retained |
| 5,405,088 | signed receipt retained |
| 7,451,677 | signed receipt retained |
| 9,435,326 | candidate exceeds scan limit |
| 11,385,026 | candidate exceeds scan limit |
| 23,251,803 | candidate exceeds scan limit |
| 25,194,363 | candidate exceeds scan limit |

So the threshold sits between 7,451,677 and 9,435,326 — 8 MiB is 8,388,608 and falls inside it. The
gate's binding string reads `full-changed-blobs`, and this is what that means: the scan reads the
whole blob, so the journal's *size*, not its diff, is what is measured.

The journal is 25,495,825 bytes on local `main`. It is **three times** the limit and has been over it
since roughly 2026-08, when it passed 8 MB. `origin/main` carries a 24,569,668-byte copy, so journal
writes were pushed while it was already far over — those pushes predate this gate's adoption in this
repository.

## What was tried, and what it settles

`release/0.24.0` was pushed in chunks and merged as PR #31, which moved `origin/main` from
`c8023067` to `b90aeab1` — 33 commits, all four required checks green. That proves chunking works
for *accumulation*.

It does not help here. Re-measured after that merge, with the candidate reduced to the single
commit `21fd733e`, the answer is still `candidate exceeds scan limit`. A synthetic commit whose only
change is the journal blob at 25,194,363 bytes is refused the same way. **No ordering, chunk size or
pull request changes this**: one blob is over the line, and every commit that writes to the planning
store rewrites that blob.

## Why it cannot be worked around here

- **The commit cannot be split.** It is atomic, and chunked pushes already carried everything
  before it: `release/0.24.0` is pushed to `3d5ec38e` and no further.
- **The journal cannot shrink.** `aep-backend-markdown` states that rewriting a line *"is exactly
  what append-only forbids"*, and `aep plan artifact validate` reads a rewrite as forgery.
- **The limit is not in this repository.** `AGENTS.md` § Commits: *"Private policy and local signing
  keys stay outside public source."* It is read from `policy.json` through `B10X_GATES_POLICY`, and
  raising it is an administrator's decision.
- **Bypassing is not a workaround.** The coordinated hooks guard plain `git push` too, and going
  around a security scanner to ship a release is not a trade this decision gets to make silently.

## It gets worse on its own

The journal only grows, and it grows on every planning write — the four commits in this batch added
roughly 900 KB between them. Whatever the limit is, the distance to it shrinks with every wave, and
the failure is total rather than partial: one blob over the line stops every push in the repository.

## What needs deciding

One of:

1. Raise the limit in the private policy, and accept that a 25 MB blob is scanned on every push.
2. Exempt `.engineering/planning/journal.jsonl` from full-blob scanning, on the argument that it is
   append-only and every line is already `aep`-validated — and state what that gives up.
3. Give the journal a size bound that does not rewrite it: rotate to `journal.<n>.jsonl` at a
   threshold, with the reader concatenating. This is a change to `aep`'s storage backend, not to
   this repository, and it does not rewrite a single recorded line.

Until one is taken, ESS cannot publish a release, and the 0.24.0 tag stays local.

## The cause, measured again, and what cleared it

The limit is neither 8 MiB nor per blob. `b10x-gates` sets `MAX_TOTAL` to 256 MiB in
`src/git.rs:12`, and `candidate()` sums the changed blobs of **every commit from the repository's
adoption baseline to the candidate head**. With the baseline at `24d2fe71`, the 46 commits already
on `origin/main` weigh 260,089,116 bytes of changed blobs, 211,438,385 of them the journal rewritten
nine times. Headroom: 8,346,340 bytes, which is the gap the table above found between 7,451,677 and
9,435,326.

So the budget is spent permanently, not per push: each planning commit costs another 25 MB of it
until the baseline moves, and the held batch (86 commits, 24 journal rewrites, 669,375,456 bytes)
could not fit in any order or chunking.

Tried after this record was cut, each measured, none the answer:

| attempt | result |
|---|---|
| publish the source with the store pinned to the published version | the `host_paths` and `internal_names` lanes fail: the batch's redaction lives in the store; PRs #32 and #33 closed |
| rotate the journal into parts | `aep` reads `journal.jsonl` only; `history` silently loses entries while `validate` says `valid` |
| scrub the home-directory prefix to `~` | 67,474 occurrences cleared, 607,266 bytes saved, 2.4%; worth doing, not the unblock |
| compress the journal with gzip | 3,394,888 bytes, the gate signs it; retracted as PR #34 before merge and reverted, because a gate that cannot read a finding has not cleared it |
| delete the journal | the gate signs it; it throws the record away |

Cleared by two changes, neither in this repository's source:

| change | measured |
|---|---|
| ESS baseline in the private policy advanced `24d2fe71` → `b90aeab1`, the public tip PR #31 merged; the previous policy file kept beside it | budget restored to 268,435,456 |
| the held tree landed as one bot commit on `origin/main`, tree unchanged from the 41-commit record branch | `b10x-gates check`: `common checks passed`, 1 commit, 317 units, 0 findings |

What stays true: the budget refills only when the baseline moves, and at 25 MB per planning commit
ten of them exhaust it again. The durable fix is in the tools, not here: a per-commit limit in
`gates` so the sum stops growing with history, or a smaller journal in `aep` (rotation with a reader
that concatenates the parts, or bodies recorded by digest). Neither is filed in this store; they
belong to the repositories they change.
