---
format: aep.planning-md/3
id: review-result:adversary-retrofit-w2-toolchain-pass-2
kind: review-result
status: active
title: 'Adversary pass 2: version line, old hand-written pins, relative cache dir, stale entries'
relations:
- reviews: story:ess-manages-its-toolchain
revision: 1
---
Adversary pass 2 against story:ess-manages-its-toolchain (beyond10x/ess#147), aep:adversary, 2026-09-27, after correction round 1.

verdict: NEEDS-CHANGE
cases: executed 73→78, red 4
origin: introduced 5, pre-existing 0, undecided 0

New cases in `crates/edge/ess-cli/tests/toolchain_adversary2.rs`. Red: `--version` under a pin ends
its first line in a manifest path; a hand-written pin below 0.34.0 is delegated to; a relative
`ESS_TOOLCHAIN_DIR` installs again per subdirectory; a cache entry without `ess` blocks installs.
Green: 12 concurrent installs. Held: argument bytes, exit codes, recursion guard, `which`, routing,
`--version` without a pin.

Coordinator decisions (final correction round, no third attack): reason line to stderr and every
stdout version line ends in a release; FIRST_PINNABLE applied in choose(); relative
ESS_TOOLCHAIN_DIR ignored; a stale cache entry is replaced; the delegated release clears
ESS_TOOLCHAIN_DELEGATED from its own environment.

```findings
[{"file": "crates/edge/ess-cli/src/toolchain.rs", "line": 203, "category": "contract-drift", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "Under an exact pin, ess --version starts with a dispatcher line ending in the manifest path, so the awk smoke-check pattern reads a path, not a release."},
 {"file": "crates/edge/ess-cli/src/toolchain.rs", "line": 172, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "A hand-written requires pin below 0.34.0 is downloaded and delegated to a release that cannot read ess-inputs/2, although --pin refuses the same pin."},
 {"file": "crates/edge/ess-cli/src/toolchain.rs", "line": 279, "category": "boundary", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "A relative ESS_TOOLCHAIN_DIR resolves against the working directory, so each subdirectory of one pinned project installs its own copy."},
 {"file": "crates/edge/ess-cli/src/toolchain.rs", "line": 375, "category": "concurrency", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "A cache entry directory without its ess refuses every install with ENOTEMPTY and no recovery hint, and a zero-length cached ess counts as already cached."},
 {"file": "crates/edge/ess-cli/src/toolchain.rs", "line": 250, "category": "judgement", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "ESS_TOOLCHAIN_DELEGATED=1 is inherited by every descendant of the delegated release."}]
```
