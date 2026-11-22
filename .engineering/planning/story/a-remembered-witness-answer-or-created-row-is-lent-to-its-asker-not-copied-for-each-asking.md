---
format: aep.planning-md/3
id: story:a-remembered-witness-answer-or-created-row-is-lent-to-its-asker-not-copied-for-each-asking
kind: story
status: draft
title: A remembered witness answer or created row is lent to its asker, not copied for each asking
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/created_memo.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/witness_memo.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/
revision: 7
---
## Outcome

Answering a witness search or a creating branch's row from memory costs a lookup, not a copy of
the answer. On the largest downstream specification known, the 11.4 % of synthesis cycles now
spent in the memories' own frames, almost all of it deep-copying a stored answer to return it,
falls to under 2 %. The suites are byte for byte the same.

## Need

Found by profile, not reported. Integration head `df7eb37a65`, the downstream specification,
`ess verify conform synthesize --path . --target ir --scenarios scenarios` (
the profile notes, section 2): `perf record -e cycles:u -F 199 --call-graph lbr`, 70,568 samples
(the recorded profile), each sample charged to its innermost frame with a full `ess_*::` path
(two awk scripts of the profile notes):

- `witness_memo::answer`: 7.80 % of all cycles in its own frames. Of these, 6.91 points are
  reached from `witness::candidates`, and the samples are `BTreeMap<String, Node>` clones
  (`clone_subtree`, `to_vec`, `memmove`) and their allocations: `answer` returns
  `answer.clone()` of a `Searched` (`Result<(Vec<BTreeMap<String, Node>>, bool, usize), _>`,
  witness.rs:696) on every hit, witness_memo.rs `answer`. `cut_short` clones the whole answer to
  read one count.
- `created_memo::created_owned` and the clone of its `Entry`: 3.25 %. A hit returns
  `top.rows.get(&key).cloned()` (a whole `Arrangement`), created_memo.rs `created_owned`.
- The other memories (`run_memo`, `model_memo`, `made`): 0.36 %.

Together 11.41 % (about 46 of the 401 CPU-s of the run). The cost grew with the memories'
success: the witness memo keyed by content turns more searches into hits, and every hit pays the
copy. The 0.97.3 profile named the same copy at 2.5 % (an earlier profile note, section 3); no
story holds it (`synthesis-time-on-large-specifications-after-0-97-3` lists other remainders).

## Fit review

Not an adopter request; a defect of cost found by profile. Considered:
- change nothing: 11 % of every large synthesis stays copying;
- share by reference (`Rc`) inside the memory, and let a caller that changes an answer copy
  only what it changes (chosen);
- shrink the copied values (paths as `Arc<[Segment]>`, interned names): broader, every crate
  naming `FactPath` changes, and it does not remove the copy, only cheapens it. Kept for later.

## Design

- `witness_memo::Memo::answers` stores `Rc<Searched>`; `answer` returns `Rc<Searched>`.
  `search` returns it; `cut_short` and `candidates_between`'s flag read through the `Rc`; the
  crate-internal callers of `candidates` that only iterate get a borrowing form
  (`candidates_shared` returning the `Rc`); the public `candidates` keeps its signature and
  copies, for API stability. A memory miss stores the `Rc` and returns a second handle to it.
- `created_memo::Entry::made` becomes `Rc<Result<Arrangement, Unreachable>>`; callers that
  modify the arrangement clone it at that point (`Rc::unwrap_or_clone` or `(*rc).clone()`), the
  others read it.
- The debug guard of `witness_memo` (`checked`) compares the shared answer with a fresh search as
  today.
- No change to what any memory answers, so no change to any byte; no format version; no
  dependency.

## Acceptance

- A work test in `ess-conformance` counts the deep copies of a remembered answer (a test counter
  beside `witness_memo`'s and `created_memo`'s existing counters) on a generic model whose
  creating command is asked many times: the count is 0 for every caller that only reads, and the
  test fails on the current code. Under 60 s.
- The byte pins (`synthesis_suite_bytes_table`, `synthesis_suite_bytes_table_cli_read`,
  `adversary_related_values_all_bytes`, `external_beside_held_guard`) pass unchanged.
- On the downstream specification, measured once (instructions:u, user CPU, suite sha256, LBR
  profile as above): the memories' own frames under 2 % of cycles, instructions:u at least 6 %
  below `df7eb37a65`'s 1,385.3e9, suite sha unchanged (the digest the profile notes record).

## Decisions

- D1. Where callers get a shared answer. A: an internal borrowing function beside the public
  `candidates`, which keeps copying; B: change `candidates` to return `Rc` (breaking for a library
  user of `ess-conformance`). Recommended: A.
- D2. Scope. A: the witness and created-row memories (11.05 of the 11.41 %); B: also `run_memo`,
  `model_memo`, `made` (0.36 %). Recommended: A; B only where a profile shows it.
