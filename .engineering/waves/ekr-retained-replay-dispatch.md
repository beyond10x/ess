# Retained-result implementation ownership

This is one coupled implementation of story:retained-command-result-replay under
the adopted docs/design/retained-command-results.md, based on cc2609e6. The earlier
scope proposal is retained as discovery, not as the adopted semantic contract.
The operator's completion instruction covers this dependency and its publication.

The implementation tree is `<worktrees>/ekr-retained-replay-unit-20260922` on
`codex/ekr-retained-replay-unit-20260922`. The primary implementor owns the Rust
domain/compiler, synthesis, runtime, native generators, diff and schema work.
It alone runs Cargo against `<cache>/b10x-target/ekr-ess-retained-replay-20260922`.
Scratch is `<cache>/ekr-completion-20260922/ess-retained-replay/implementation`.

Once the closed Observation and capture/expect step interface existed, a second
implementor was assigned only these disjoint paths in the same coupled unit:

- crates/verify/ess-conformance/src/go/replay.go (new)
- crates/verify/ess-conformance/src/go/runtime.go
- crates/verify/ess-conformance/src/go/mod.rs (emitter inclusion only)
- crates/verify/ess-conformance/tests/fixtures/retained-replay-runtime.go (new)

The second implementor runs Go checks only, never Cargo or repository-wide
formatters; the primary implementor does not edit these paths until handback.
Both keep their own worktree leases. Their shared source interface is the
primary implementor's src/replay.rs::Observation and runner.rs::retained_result.
The adopted design, not provisional implementation, decides semantics.
The Go worker retains red/green parity evidence under the sibling go-runtime
scratch directory. This split adds no scope beyond the recorded crate scope.

The coordinator owns all AEP calls, normative documents, version/release work
and integration commits. Neither worker owns those files or may dispatch further
agents. Independent adversarial review follows the combined implementation;
package evidence does not substitute for the complete repository gate.
