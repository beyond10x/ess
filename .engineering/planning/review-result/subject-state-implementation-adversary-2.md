---
format: aep.planning-md/1
id: review-result:subject-state-implementation-adversary-2
kind: review-result
status: active
title: Subject-state implementation adversary, round 2
relations:
- reviews: story:subject-state-outcome-guards
revision: 1
---
unit: subject-state-outcome-guards second bounded review at a39bb16c04a4bfdda0302833fc979a17b5968fde
verdict: CONFIRMED
cases: executed 0→1 new selected case, red 1
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: assigned review evidence directory and existing coordinator compiler cache; private inventory retained
needs-coordinator: record before correcting conservative move-source validation
<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 1a92a673bd2ab203cdd1b1328f2ba34bdf51010716d63c9a02077e5eced7311b, retained as local-evidence:runtime-gaps/publication-replay/snapshots/1a92a673bd2ab203cdd1b1328f2ba34bdf51010716d63c9a02077e5eced7311b.md. Source creation recorded at 2026-09-11T04:05:28Z. Private labels and local paths are projected to descriptive aliases. This is a disclosed immutable-review publication projection, not a new review. Original review body SHA256 af909aba8ea9cbdd8772a4849abcde81843a306adee5602134a177774f13f945, retained privately as local-evidence:runtime-gaps/publication-replay/snapshots/af909aba8ea9cbdd8772a4849abcde81843a306adee5602134a177774f13f945-body.md. Verdict, finding identifiers, outcomes and measured counts are retained; label/path redaction may change their literal spelling.
<!-- public-import-provenance:end -->

```text
$ git diff --no-index --stat /dev/null crates/specify/ess-domain/tests/subject_state_open_default_adversary.rs
 .../tests/subject_state_open_default_adversary.rs  | 68 ++++++++++++++++++++++
 1 file changed, 68 insertions(+)
```

That new test is this reviewer's entire checkout delta. Existing coordinator documentation/planning changes remain untouched. No implementation edits, existing-test changes, Git commits, AEP mutations or cleanup occurred. Source module SHA256 before correction: `80f5a82973a8a19e9b03f703f5e497f129214956ba74e5bac90046e6f68487f1`; test SHA256: `977f32f84243cd9b55a2cbe88bba0093b4a1ba969af52cb6f07abdd03c3ab5d3`.

1. One complete source-derived regression was written before execution:

`crates/specify/ess-domain/tests/subject_state_open_default_adversary.rs:56`, `open_input_default_move_must_not_accept_an_invalid_held_state`, first assembles a complete valid ess/3 model with declared Call states Init/Bridged, a bridge transition from Init only, a real Bridge command driving it, and an Enrich command. Enrich has an Init-and-amount-positive guarded update plus an unguarded state-preserving fallback. This control assembles successfully. The only mutation changes Enrich's fallback from update to the bridge move. In held state Bridged, the Init guard is false for every admitted Integer amount, so the fallback selects an impossible transition.

The test requires assembly to refuse that mutated model. It remains red. Its first and only execution compiled successfully and failed at precisely that assertion; there was no malformed-control correction or preceding suite run.

Command, with the assigned integration compiler cache, Rust 1.98.1, two jobs, debug/incremental disabled, sccache and lld:

```text
cargo test --offline --locked -p ess-domain --test subject_state_open_default_adversary open_input_default_move_must_not_accept_an_invalid_held_state -- --exact
```

Complete runner output after compiler/path lines:

```text
running 1 test
test open_input_default_move_must_not_accept_an_invalid_held_state ... FAILED

failures:

---- open_input_default_move_must_not_accept_an_invalid_held_state stdout ----

thread 'open_input_default_move_must_not_accept_an_invalid_held_state' (3305419) panicked at crates/specify/ess-domain/tests/subject_state_open_default_adversary.rs:64:5:
open input plus a genuine default admitted bridge from held state Bridged
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    open_input_default_move_must_not_accept_an_invalid_held_state

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p ess-domain --test subject_state_open_default_adversary`
exit: 101
```

Complete original log is retained at `local-evidence/subject-state/adversary-2/first.log`, SHA256 `5032001b90984479f4011f9c4d6360d608651f96209782d931533a52fa10448d`. The `first-public.log` variant substitutes only exact tree/cache prefixes; raw original bytes are unchanged.

2. No affected or existing tests were run, per the coordinator's explicit instruction for this second bounded pass. The count 0→1 refers only to the new selected test, not a repeated count of the prior implementation suite. The compiler slot was released after its terminal exit.

3. Finding:

| Location | Verdict | Origin | What was measured | What reaches it |
| --- | --- | --- | --- | --- |
| `crates/specify/ess-domain/src/command/subject_state.rs:146` | CONFIRMED | introduced | When finite state/input analysis is unavailable, a genuine default bypasses move-source checks and can admit a transition from an invalid held state. | Public `RawSpecFile::parse` and `Specification::assemble` on the complete source model above; an ordinary Integer guard makes the finite analyzer return None, while a declared default bypasses the finite-case move validation. |

The fallback has valid syntax, declared identity/event/transition references and a genuine default. This is semantic admission of an impossible lifecycle move, not merely incomplete witness synthesis. State-qualified moves are checked separately, but the default only reaches `validate_move` inside the finite-case loop that this return skips. Preserve the passing update control and reject unproved move-source safety when no finite partition is available. Unqualified guarded Moves outcomes share that missing proof seam and should receive the same conservative treatment; no claim of a separately executed second case is made.

Origin is introduced by this subject-state implementation: the module is absent at pre-unit base `ff5ec1ac895ea88b0b452d0213c06f02382270af` (read-only `git show` refusal retained). The first review's external-subject correction is present and this case contains no external branch. This is new ground, with a distinct signature and mechanism, not a repetition of that finding.

4. This review was confined to the open-partition/default/move-source seam and normal specification reachability. It provides no additional runtime, publication or whole-unit correctness claim.

5. Public paths intentionally use aliases under the user's privacy constraint. Full paths and every reviewer output are in `inventory-private.md`. Only the assigned evidence directory, new test, worktree-manager lease metadata, existing compiler cache and shared sccache received writes. The coordinator retains cleanup ownership. The review lease is released at handback.

```findings
- file: crates/specify/ess-domain/src/command/subject_state.rs
  line: 146
  category: acceptance
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: When finite state/input analysis is unavailable, a genuine default bypasses move-source checks and can admit a transition from an invalid held state.
```