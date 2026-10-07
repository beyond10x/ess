# Repository invariants (ess), wave 2026-10-07c

Read `AGENTS.md` at the worktree root first; it wins over this list.

- Running code is Rust, with clap derive for command lines. No Python, no shell checkers committed.
- Spec first: model the change in this repository's ESS specification, validate it with the
  newest `ess`, regenerate, then implement against the generated code. If the specification
  cannot express it, stop and report that; do not hand-write a parallel model.
  (Installed `ess --version` is 0.55.0, the newest release. The repository's own models live under
  `models/`; say in the report whether the change touches anything they declare.)
- Before writing a fix, reproduce the issue on the installed `ess` 0.55.0 with its reproducer
  (renamed to neutral nouns if needed). If it does not reproduce, stop and report that with the
  output.
- Each worktree builds into its own `target/` inside the worktree. Never set `CARGO_TARGET_DIR`.
  Never point two trees at one build directory. Never delete a `target/` by hand.
- `task fmt-check` or `cargo fmt -p <crate>`, never `cargo fmt --all`. Package-scoped gates only:
  `cargo clippy -p <crate> --all-targets --locked -- -D warnings`, `cargo test -p <crate> --locked`.
  The full gate runs once, later, on the integration branch.
- Never write under `.engineering/`, and no `aep plan artifact` write verb. Reads are fine.
- No `git commit`, `git add`, `git stash`, branch or `git worktree` command. Leave changes in the
  working tree; the coordinator commits. Never add `.agents/skills/worktree/` (untracked, not the
  repository's).
- Nothing under `/tmp`. Scratch goes to the scratch root named in your brief.
- `CHANGELOG.md` is the coordinator's. Put the entry you would write into your report instead.
- No absolute home-directory path (`/home/<name>/`) in any file you write: the security scan fails
  every commit that carries one.
- Never name a customer, an employer or another coordination session in code, tests or docs.
  Use neutral nouns (`catalog`, `items`, `orders`).
- Another session is working in this repository. Do not edit `crates/specify/ess-domain/src/command.rs`,
  `crates/specify/ess-compiler/src/ir.rs`, or add files under `crates/specify/ess-domain/src/command/`
  or `crates/specify/ess-compiler/src/ir/`. If a fix needs one of them, stop and report it.
- Disk is shared and tight. Check `df -h /` before every build; under 20G free, stop and report.
  Use `CARGO_INCREMENTAL=0` for every cargo command.
- While the other session's selection-plan wave 3 runs, also keep off: `crates/generate/ess-entity-runtime/src/lib.rs`;
  `crates/generate/ess-synth/src/rust/behaviour.rs`, `src/go/behaviour.rs`, `src/determined.rs`, `src/plan.rs`;
  `crates/specify/ess-domain/src/command/subject_state.rs` and `src/command/related_guard.rs`. If a fix needs one, stop and report it.
