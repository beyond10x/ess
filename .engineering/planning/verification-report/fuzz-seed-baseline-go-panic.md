---
format: aep.planning-md/1
id: verification-report:fuzz-seed-baseline-go-panic
kind: verification-report
status: draft
title: Fuzz candidate seed baseline finds Go synthesis panic
relations:
- verifies: story:fuzz-the-specification-surface
revision: 1
---
# Actual fuzz-candidate seed baseline

Preparation for story:fuzz-the-specification-surface; no implementation or fuzzing run.
The current 0.20.0 CLI was executed from the continuing coordinator at d2057ff. All tracked
crates and root manifests are byte-identical to the published/gated 239996d source.
Binary SHA256 4cb115678097c243034f0ef6525c9db87c6c82a03ef7f799f6fe3ac5e38b534e.

Three minimal local seeds were reconstructed from existing feasibility.rs controls: system-level
type at line 404, String to Optional<String> binding from lines 382–399, and no-delivery/no-retry
system from line 409. All three passed actual CLI validation. No external consumer source was
needed. Each accepted seed then reached all six generation kinds and all four synthesis targets.

| Seed | Command lane | Actual exit | Panic marker |
| --- | --- | ---: | --- |
| system-types | validate | 0 | false |
| system-types | generate-docs | 0 | false |
| system-types | generate-site | 0 | false |
| system-types | generate-docs-ir | 0 | false |
| system-types | generate-schema | 0 | false |
| system-types | generate-openapi | 0 | false |
| system-types | generate-asyncapi | 0 | false |
| system-types | synthesize-rust | 1 | false |
| system-types | synthesize-go | 101 | true |
| system-types | synthesize-web | 1 | false |
| system-types | synthesize-clap | 0 | false |
| optional-binding | validate | 0 | false |
| optional-binding | generate-docs | 0 | false |
| optional-binding | generate-site | 0 | false |
| optional-binding | generate-docs-ir | 0 | false |
| optional-binding | generate-schema | 0 | false |
| optional-binding | generate-openapi | 0 | false |
| optional-binding | generate-asyncapi | 0 | false |
| optional-binding | synthesize-rust | 1 | false |
| optional-binding | synthesize-go | 0 | false |
| optional-binding | synthesize-web | 1 | false |
| optional-binding | synthesize-clap | 0 | false |
| no-retry | validate | 0 | false |
| no-retry | generate-docs | 0 | false |
| no-retry | generate-site | 0 | false |
| no-retry | generate-docs-ir | 0 | false |
| no-retry | generate-schema | 0 | false |
| no-retry | generate-openapi | 0 | false |
| no-retry | generate-asyncapi | 0 | false |
| no-retry | synthesize-rust | 0 | false |
| no-retry | synthesize-go | 0 | false |
| no-retry | synthesize-web | 0 | false |
| no-retry | synthesize-clap | 0 | false |

There are 33 actual CLI invocations, including three validation commands; these are subprocess
counts, not Rust test cases or fuzz iterations. The Go system-type run exited 101 and printed:

```text
thread 'main' (4181657) panicked at crates/generate/ess-synth/src/go/layout.rs:290:13:
`demo.Code` is not a declaration this layout knows: it was derived from a different IR
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

The Go panic is an observed production defect at src/go/layout.rs:290. The fuzz-only scope
cannot repair it. Before selecting that story, refresh its scope to include the concrete Go
synthesis owner and decide a checked representation/refusal path. Do not discard the mandatory
seed, turn its panic into an expected success or call the unimplemented harness green.

Four nonzero Rust/Web commands returned ordinary structured target refusals:

system-types / synthesize-rust:

```text

```

system-types / synthesize-web:

```text

```

optional-binding / synthesize-rust:

```text

```

optional-binding / synthesize-web:

```text

```

The other target invocations exited zero. No generated program was compiled, and a finite
three-seed observation does not establish the universal fuzz property. The baseline neither
changes production nor selects the next wave. Current coverage-writer implementation is separate.

All seed bytes, 33 argv/exit/timing/hash receipts, complete stdout/stderr and produced artifacts
are retained under target/review-boundaries-11/fuzz-preparation. The complete evidence manifest
contains 199 files and has SHA256 1deff9b0138340598a9e6e8bb620c4dce735c31b71567516b8e120ae4c3bc847.
