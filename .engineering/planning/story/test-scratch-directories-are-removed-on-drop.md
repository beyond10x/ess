---
format: aep.planning-md/3
id: story:test-scratch-directories-are-removed-on-drop
kind: story
status: draft
title: Every test that creates a TMPDIR directory removes it on drop, panics included
relations:
- serves: vision:O2
revision: 1
---
## Finding

Measured on 2026-10-07 on a shared workstation: the test suites leave their scratch directories in
`$TMPDIR`. After the 0.56.0 release gate and earlier runs, the shared temporary directory held 4,781
`ess-*` directories (18,524 MiB): `ess-ownership-*`, `ess-cache-origin-*`, `ess-delivery-trust-*`,
`ess-bidi-*`, `ess-replay-*`, `ess-browser-*`, `ess-go-parity-*` and others, each named with the
creating test process's pid. One gate run wrote about 9 GiB there. Some fixtures are read-only
(`ess-ownership-*`) and cannot be removed without restoring the write bit.

## Acceptance

- Every test helper that creates a directory under `std::env::temp_dir()` (or `$TMPDIR`) removes it
  when its guard drops, including when the test panics; a read-only fixture gets its write bit back
  before removal.
- A test (or an `ess-xtask` check) runs a representative set of the suites with `TMPDIR` set to an
  empty directory and finds it empty afterwards, failing and naming the leaking prefix otherwise.
- `CARGO_TARGET_TMPDIR` usage is unchanged; only `TMPDIR` scratch is in scope.
