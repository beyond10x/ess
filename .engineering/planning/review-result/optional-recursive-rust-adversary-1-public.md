---
format: aep.planning-md/3
id: review-result:optional-recursive-rust-adversary-1-public
kind: review-result
status: active
title: 'Adversary: carriage returns break generated Rust documentation (publication copy)'
relations:
- reviews: story:optional-recursive-rust
revision: 1
---
unit: optional-recursive-rust at 09df87a6123a5ae50d752d2e3223fc19f596c9de
verdict: CONFIRMED
cases: executed 0→3, red 1
origin: introduced 0 / pre-existing 0 / undecided 1
wrote-outside-worktree: 176 files under assigned scratch; complete inventory below
needs-coordinator: route carriage-return failure to implementor; retain tests/report

Publication copy: personal filesystem prefixes are redacted as <home> by the coordinator to satisfy the publication privacy gate. All findings, commands, outcomes and counts are unchanged. Exact private original SHA-256: acbadbd03fca18123980e147d794ac906f30bf09d2315aeef44bc5d557831951. The original immutable record remains in the privately archived implementation worktree; this is a new publication record, not an edit to it.

1. `git --no-pager diff --stat`

```text
(empty)
```

The assigned review tree remains clean at the candidate commit. Only assigned scratch files were written. Own lease `ekr-synthesis-adversary-scope-retention-20261003` was acquired and released.

2. Cases written before execution

Standalone Rust test runner:

`<home>/.cache/ekr-knowledge-prereq-20261003/adversary/attacks.rs`

- `recursive_compositions_compile_and_execute`: **green**. Fresh generation and compilation in crate/workspace layouts; recursive values nested through newtypes, unions, lists, maps and optional values; authored `Box` name; wire-label preservation; binding accessors; nested structured event constructors.
- `continued_prose_with_carriage_return_stays_compilable`: **red**. Valid YAML summary `"Paragraph one.\nParagraph two.\rParagraph three."` passes synthesis, but emitted Rust fails compilation.
- `optional_self_does_not_admit_union_cycle`: **green**. Optional-self support does not admit a second cycle through a union; refusal identifies source types and cyclic fields and produces no generated crate.

Two harness assumptions were corrected before the suite: accessor depth is limited to three segments, and machine-readable refusal source addresses name declarations rather than YAML filenames. Those preliminary failures are not product findings; their original logs remain retained.

Isolated red command:

```text
<home>/.cache/ekr-knowledge-prereq-20261003/adversary/attacks --exact continued_prose_with_carriage_return_stays_compilable --nocapture
```

Isolated red output, captured before the suite:

```text
running 1 test

thread 'continued_prose_with_carriage_return_stays_compilable' (1290917) panicked at <home>/.cache/ekr-knowledge-prereq-20261003/adversary/attacks.rs:31:5:
fresh generated crate failed: exit status: 101
     Locking 1 package to latest compatible version
   Compiling demo v1.0.0 (<home>/.cache/ekr-knowledge-prereq-20261003/adversary/carriage-prose/crate/generated)
error: bare CR not allowed in doc-comment
 --> <home>/.cache/ekr-knowledge-prereq-20261003/adversary/carriage-prose/crate/generated/src/lib.rs:9:19
  |
9 | //! Paragraph two.␍Paragraph three.
  |                   ^

warning: ignoring -C extra-filename flag due to -o flag

error: bare CR not allowed in doc-comment
 --> <home>/.cache/ekr-knowledge-prereq-20261003/adversary/carriage-prose/crate/generated/src/core.rs:9:19
  |
9 | //! Paragraph two.␍Paragraph three.
  |                   ^

warning: `demo` (lib) generated 1 warning
error: could not compile `demo` (lib) due to 2 previous errors; 1 warning emitted

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test continued_prose_with_carriage_return_stays_compilable ... FAILED

failures:

failures:
    continued_prose_with_carriage_return_stays_compilable

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out; finished in 2.29s
```

3. Suite run after the cases existed

```text
<home>/.cache/ekr-knowledge-prereq-20261003/adversary/attacks --nocapture --test-threads=1
```

Exit status: **101**.

```text
running 3 tests
test continued_prose_with_carriage_return_stays_compilable ...
thread 'continued_prose_with_carriage_return_stays_compilable' (1340885) panicked at <home>/.cache/ekr-knowledge-prereq-20261003/adversary/attacks.rs:31:5:
fresh generated crate failed: exit status: 101
   Compiling demo v1.0.0 (<home>/.cache/ekr-knowledge-prereq-20261003/adversary/carriage-prose/crate/generated)
error: bare CR not allowed in doc-comment
 --> <home>/.cache/ekr-knowledge-prereq-20261003/adversary/carriage-prose/crate/generated/src/lib.rs:9:19
  |
9 | //! Paragraph two.␍Paragraph three.
  |                   ^

warning: ignoring -C extra-filename flag due to -o flag

error: bare CR not allowed in doc-comment
 --> <home>/.cache/ekr-knowledge-prereq-20261003/adversary/carriage-prose/crate/generated/src/core.rs:9:19
  |
9 | //! Paragraph two.␍Paragraph three.
  |                   ^

warning: `demo` (lib) generated 1 warning
error: could not compile `demo` (lib) due to 2 previous errors; 1 warning emitted

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test optional_self_does_not_admit_union_cycle ... ok
test recursive_compositions_compile_and_execute ... ok

failures:

failures:
    continued_prose_with_carriage_return_stays_compilable

test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 12.01s
```

The `0→3` count describes this new standalone adversary suite. Coordinator-reported existing counts were feasibility 53/adversary 14; I did not rerun those suites or claim their coverage.

4. Finding

| Location | Category | Severity | Verdict | Origin | Finding |
|---|---|---|---|---|---|
| `crates/generate/ess-synth/src/rust/mod.rs:69` | boundary | blocker | CONFIRMED | undecided | The documentation escaper preserves a compiler-admitted escaped carriage return, so successful synthesis emits Rust rejected with “bare CR not allowed in doc-comment.” |

**Measured:** the isolated case at `attacks.rs:157` calls the real synthesis CLI; synthesis succeeds, and fresh generated compilation exits 101 at generated `src/lib.rs:9` and `src/core.rs:9`.

**Reachable:** normal authored YAML free text with an escaped `\r`, through `ess generate synthesize --layout crate`. No implementation mutation, corrupt IR, or bypass was used. This establishes the valid-source compilation defect; it does not establish that current EKR source contains carriage returns.

**Origin:** undecided because the exact base was not executed. The source suggests an older free-text issue, but that is not a measured origin.

**Needed change:** handle Rust-invalid carriage returns in emitted documentation, preserving already-successful single-line output, and add a permanent fresh-compilation regression.

5. Attacked without a break

- Nested recursive newtype/union/collection values execute and round-trip in both layouts.
- Recursive structured constructors and optional accessor traversal agree on boxed representation.
- Explicit enum wire labels survive encoding.
- Additional union recursion remains refused with source-addressed diagnostics.

6. Outside-worktree paths

Everything written is under:

`<home>/.cache/ekr-knowledge-prereq-20261003/adversary`

The complete inventory of **176 exact file paths** is:

`<home>/.cache/ekr-knowledge-prereq-20261003/adversary/written-paths.txt`

Retain that inventory with the report. It includes the Rust runner/binary, individual-case logs, suite log/status, generated fixtures and harnesses under `composition/`, `carriage-prose/`, `union-cycle/`, and the task-owned compiler output under `target/`.

The supplied CLI was read-only. Its measured SHA-256 was:

```text
1d58646f85a327bff6c9ed78f410510acb00d89f975e915e4423d5ea94832d08
```

```findings
- file: crates/generate/ess-synth/src/rust/mod.rs
  line: 69
  category: boundary
  severity: blocker
  verdict: CONFIRMED
  origin: undecided
  message: The documentation escaper preserves a compiler-admitted escaped carriage return, so successful synthesis emits Rust rejected with “bare CR not allowed in doc-comment.”
```
