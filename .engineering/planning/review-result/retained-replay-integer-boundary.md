---
format: aep.planning-md/2
id: review-result:retained-replay-integer-boundary
kind: review-result
status: active
title: Retained result Integer observation boundary review
relations:
- reviews: story:retained-command-result-replay
revision: 1
---
unit: ESS retained-command-result replay, working-tree design and Integer boundary reviewed on 2026-09-22
verdict: CONFIRMED
cases: executed 0→0, red 0
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 report body
needs-coordinator: retain original findings and correction disposition; verify actual adapter acceptance in the subsequent source review

No repository diff was authored. This was a bounded read-only design review, not an implementation adversary run: no test was added or executed, and no build, AEP, provider or worktree action was performed. The original review remains preserved as `integer-boundary-review.md`; this public-safe record does not replace it. Source line coordinates below identify the text inspected during that original review, before the reported corrections.

## Original findings

| ID | Location | Category | Severity | Verdict | Origin | Finding |
|---|---|---|---|---|---|---|
| F1 | `crates/verify/ess-conformance/src/go/replay.go:185` | contract-drift | blocker | CONFIRMED | introduced | The coordinator's lexical Integer hypothesis rejected 1.0, 1e0 and negative zero in replay although Rust admitted those values and the existing Number serializer emitted small integers as 1.0. |
| F2 | `docs/design/retained-command-results.md:95` | boundary | warning | CONFIRMED | introduced | The new typed-result equality contract omitted an explicit lossless adapter obligation, allowing generic JSON-to-Node admission to be mistaken for preservation of the actual handler's Integer. |

F1 concerns the temporary replay-only filter implementing the coordinator's hypothesis, not a pre-existing Go numeric defect. Its regex accepted only integer-shaped tokens and explicitly excluded `-0`; the relevant distinction was mathematical i64 admission, not decimal/exponent spelling. What reaches it: replay result admission in Go, including `1.0` produced by the existing Rust Number serializer. The Rust implementor's retained `integer-parity-probe.log` measured admission of `1`, `1.0`, `1e0`, `-0` and `-0.0`; its isolated probe completed one case, zero failures, exit 0. That execution belongs to the implementor and is not counted as a test executed by this review. The serializer branch at `crates/specify/ess-primitives/src/facts.rs:143–158` confirms that small integers retain their legacy f64 spelling.

F2 concerns the new contract's unspoken observation boundary, not a demand to migrate global Number semantics. `crates/specify/ess-primitives/src/node.rs:35–70` converts a serde_json::Value using exact integer accessors where available, otherwise via f64. The same retained probe measured these three admitted values:

| Raw JSON token | Integer held by Rust Node |
|---|---:|
| `9007199254740993` | 9007199254740993 |
| `9007199254740993.0` | 9007199254740994 |
| `9.007199254740993e15` | 9007199254740992 |

Thus equal mathematical input spellings can already differ before replay comparison. Conversely, the measured decimal-form value and the exact integer token `9007199254740994` would become equal Rust nodes despite different mathematical transport values; that comparison is a deduction, not a separately executed case. Go's bounded exact-rational observation in `crates/verify/ess-conformance/src/go/response.go:296–326` does not share that rounding.

What reaches F2: the public Node deserialization path can be selected by an adapter; once rounding has occurred, `src/replay.rs:208–242` cannot recover the original value. No shipping JSON target using that path was established by this bounded review. `src/target.rs:456–489` exposes a typed result map and derives Serialize, not Deserialize; the target interface does not require JSON transport. This reachability limit is why F2 is a warning about the contract/adapter obligation, not a confirmed production adapter defect or a global decoder blocker. The inherited raw JSON behavior is background evidence and is not counted as another introduced finding.

## Resolution recommended by this review

A demonstrably lossless typed target boundary satisfies the adopted exact-i64 equality rule. Native adapters can construct the observed number directly from the actual handler's i64/int64. A JSON adapter must exact-decode declared Integer values before constructing Node, or report the observation Unsupported. A declaration that an adapter is typed cannot certify a lossy conversion it actually uses.

Retain current global Number bytes, recursive Decimal/Binary64 replay refusal and exact typed equality. Do not normalize Go through binary64 or reject decimal/exponent spellings merely to imitate a hypothesized Rust token distinction. Separate typed-result parity vectors from raw JSON parser probes; the latter do not prove cross-runner transport parity.

Required future acceptance covers the actual adapter with adjacent integers beyond 2^53, both i64 endpoints and nested positions; equal retries pass and a changed adjacent result fails. A mutation routing that adapter through binary64 must fail. Exercise any serialization round trip the adapter actually uses. These requirements were proposed, not executed, by this review.

## Correction disposition

The coordinator subsequently amended the design and related documentation to adopt the lossless native boundary, exact JSON decoding or Unsupported, and truthful evidence labels. The new obligation and adapter acceptance are present in `docs/design/retained-command-results.md:105–120`, inspected while preparing this record. The Go implementor reported removing the temporary lexical/-0 filter and preserving the old hypothesis only in scratch.

This closes the design decision identified by F1/F2. It does not erase the original findings or certify implementation closure: the filter correction, actual adapters, mutations and combined Rust/Go acceptance await the subsequent source review. No post-fix no-findings verdict is substituted here.

Owners: the coordinator owns the incorrect initial lexical hypothesis and the missing explicit adapter obligation in the new contract, and owns the adopted correction. The Go implementor acted on that hypothesis and owns implementing and testing its removal; the Rust and Go implementors own truthful evidence labels and typed comparison. Each actual target adapter owner owns lossless observation of its handler result. The inherited Node parsing behavior and legacy Number serialization are not assigned to either implementor as a new defect requiring global migration.

Scope limits: this record contains agent review findings, not verifier evidence or approval. Only the assigned sibling report body was written; no repository files changed. Public paths are repository-relative, and the coordinator retains the private scratch location separately.

```findings
- file: crates/verify/ess-conformance/src/go/replay.go
  line: 185
  category: contract-drift
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: The coordinator's lexical Integer hypothesis rejected 1.0, 1e0 and negative zero in replay although Rust admitted those values and the existing Number serializer emitted small integers as 1.0.
- file: docs/design/retained-command-results.md
  line: 95
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: The new typed-result equality contract omitted an explicit lossless adapter obligation, allowing generic JSON-to-Node admission to be mistaken for preservation of the actual handler's Integer.
```
