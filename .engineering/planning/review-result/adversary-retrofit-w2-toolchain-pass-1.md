---
format: aep.planning-md/2
id: review-result:adversary-retrofit-w2-toolchain-pass-1
kind: review-result
status: active
title: 'Adversary pass 1: toolchain pin edge cases, seven red'
relations:
- reviews: story:ess-manages-its-toolchain
revision: 1
---
Adversary pass 1 against story:ess-manages-its-toolchain (beyond10x/ess#147), aep:adversary, 2026-09-27.

verdict: red
cases: executed 68→76, red 7
origin: introduced 7, pre-existing 0, undecided 0

New cases in `crates/edge/ess-cli/tests/toolchain_adversary.rs`. Red: empty `ESS_TOOLCHAIN` refuses
every command; an unreadable working directory fails every command including `--version`; under an
uninstallable pin `--version` prints nothing on stdout; `--pin` drops a comment on an unchanged `/2`
format line; `--pin` writes LF into a CRLF manifest; `install 0.33.0 --pin` writes a manifest 0.33.0
cannot read; a symlinked nearest manifest is skipped for the one above. Green: the `XDG_CACHE_HOME`
cache root. Held: recursion guard, checksum before extraction, tarball member matching, `/1`→`/2`
upgrade, auto-install note without prompting, `--version` unchanged when not delegated.

Coordinator decisions: all eight back to the implementor; pins below 0.34.0 are refused; plain
`http://` sources are refused; `--pin` refuses to write through a symlink.

```findings
[{"file": "crates/edge/ess-cli/src/toolchain.rs", "line": 125, "category": "boundary", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "An empty ESS_TOOLCHAIN refuses every command with exit 1, although the two sibling toolchain variables treat empty as unset."},
 {"file": "crates/edge/ess-cli/src/toolchain.rs", "line": 135, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "The pre-dispatch hook makes every command, including --version, fail in a working directory that cannot be read."},
 {"file": "crates/edge/ess-cli/src/toolchain.rs", "line": 182, "category": "acceptance", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "Under a pin that cannot be installed, ess --version prints nothing on stdout."},
 {"file": "crates/edge/ess-cli/src/input_discovery.rs", "line": 212, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "write_pin rewrites an unchanged /2 format line and drops its comment."},
 {"file": "crates/edge/ess-cli/src/input_discovery.rs", "line": 198, "category": "boundary", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "write_pin ends the lines it writes with LF in a CRLF manifest."},
 {"file": "crates/edge/ess-cli/src/toolchain.rs", "line": 450, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "install --pin to a release before 0.34.0 writes an ess-inputs/2 manifest the pinned release cannot read."},
 {"file": "crates/edge/ess-cli/src/input_discovery.rs", "line": 183, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "nearest_pin skips a symlinked nearest manifest and uses one further up the tree."},
 {"file": "crates/edge/ess-cli/src/toolchain.rs", "line": 402, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "An http:// ESS_TOOLCHAIN_BASE_URL is accepted, making the checksum check meaningless."}]
```
