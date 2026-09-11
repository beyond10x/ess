---
format: aep.planning-md/1
id: review-result:response-payload-adversary-pass1
kind: review-result
status: active
title: Review actual typed command response observations
relations:
- reviews: story:typed-response-outcome-payloads
revision: 1
---
unit: typed-response-outcome-payloads — adversary pass 1
verdict: red
cases executed: 1 new regression; 0 passed, 1 failed; valid control passed inside the case
origin: n/a
wrote-outside-worktree: private gap13/review1 evidence only
needs-coordinator: yes — record immutable finding before production correction
<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 e460757a567fede1ec69e495a30f092695106089183d2f7cb1fcd01b9ef7016f, retained as local-evidence:runtime-gaps/publication-replay/snapshots/e460757a567fede1ec69e495a30f092695106089183d2f7cb1fcd01b9ef7016f.md. Source creation recorded at 2026-09-11T05:52:24Z. Private labels and local paths are projected to descriptive aliases. This is a disclosed immutable-review publication projection, not a new review. Original review body SHA256 5b87cdbfee25f52ce8e180e4eed0198d205a9fde90f06c307536d6ed3b0038f5, retained privately as local-evidence:runtime-gaps/publication-replay/snapshots/5b87cdbfee25f52ce8e180e4eed0198d205a9fde90f06c307536d6ed3b0038f5-body.md. Verdict, finding identifiers, outcomes and measured counts are retained; label/path redaction may change their literal spelling.
<!-- public-import-provenance:end -->

## Exact candidate

Reviewed source checkpoint: base 1fd6ba62c497c1c0fd4354745e2f60e26859c235; implementation.patch SHA256 e632ac5218803079257d0c345953720eeaf121916535e50253b653d8e48d2b09; candidate.sha256 manifest digest f22c1e156252d103ab04dea8cffa1dcc775a65e4d75a4211f9e458a46f57c5e7. During review the coordinator committed the unchanged implementation as 47d4241057014c370be09f0ca751ad085b22035a. Every manifest entry verified unchanged after the deciding run. Temporary coordinated format routing remains present for source4/suite8/9/diff4 admission.

## R1 — Closed tagged union response admits undeclared members

Severity: high; confirmed executable false acceptance in the new response observation contract. Rust `crates/verify/ess-conformance/src/selection.rs`, `validate_value_inner`, `Declaration::Union` branch, and Go `crates/verify/ess-conformance/src/go/runtime.go`, `selectionObservation.validateValue`, `case "union"`, validate only the declared tag and selected content. The new response-mode struct path rejects unknown members, but its union path does not.

The regression compiles a valid source4 model whose actual command response and emitted event both contain a declared tagged union. The valid value `{item:{kind:"text",value:"ok"}}` passes. Adding `unexpected:true` to both independently supplied arguments also returns `Ok(())`; it must fail typed value admission even when response and event agree. This contradicts the closed union schema emitted by `crates/generate/ess-gen/src/types.rs::variant`, which sets `additional: Some(Additional::Refused)`. Undeclared descendants are also never visited by the Rust value byte/depth accounting; that consequence is static, not a second executed attack. Go has the same missing member check, although its command-result snapshot separately bounds serialized bytes.

Smallest correction: in response mode, require union member names to be exactly the declared tag and the existing `union_content_key(tag)` (allow content absence only where the existing optional branch permits it). Apply equivalent Rust and Go checks without altering legacy selection behavior. Preserve `tag: value` / `content` spelling and optional content semantics. Add the corresponding Go mutation to the existing response runner witness. No production changes made by this reviewer.

## Deciding evidence

New file: `crates/verify/ess-conformance/tests/adversary_response_union.rs`.

Command: `cargo test --offline --locked -p ess-conformance --test adversary_response_union`, using the assigned gap13 warm cache, Rust 1.98.1, two jobs, no incremental/debug data, sccache/lld. Exit 101; compile 0.30 seconds; one test executed in 0.00 seconds. Log: `union-red.log`. No existing successful suite was rerun. Compiler slot released immediately afterward.

## Bounded review coverage

Read source declaration/admission, response observation closure and type checks, Rust runner last-invocation routing, Go result snapshot and exact integer comparison, native response-bearing outcomes and match helpers, schema response projection, semantic diff cases and version selectors. Reviewed implementor evidence for null/missing/map mutations, exact command/outcome binding, response-source literal compatibility, browser refusal, and native Rust/Go execution; those are inherited evidence, not tests executed by this reviewer. Actual response is supplied by the implementation and compared to its direct event; expected fixture values are not used to manufacture it. Optional response absence is explicitly handled. New envelope versions remain coordinated. No independent claim of exhaustive correctness is made after this deciding red finding.

## Handoff

Only the new regression file was added in the managed candidate; no implementation/Git/AEP changes. Root owns immutable review recording and subsequent correction dispatch. The reviewer releases only lease `ess-gap13-review1-20260911`; implementor/root leases are untouched.

```findings
- file: crates/verify/ess-conformance/src/selection.rs
  line: 481
  category: contract-drift
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: Response-mode tagged unions admit undeclared members even though the generated typed schema is closed; equivalent Go validation shares the defect.
```