# Repository invariants (ess), wave 2026-10-07b

Read `AGENTS.md` at the worktree root first; it wins over this list.

- Running code is Rust, with clap derive for command lines. No Python, no shell checkers committed.
- Spec first: model the change in this repository's ESS specification, validate it with the
  newest `ess`, regenerate, then implement against the generated code. If the specification
  cannot express it, stop and report that; do not hand-write a parallel model.
  (Installed `ess --version` is 0.55.0, the newest release. The repository's own models live under
  `models/`; say in the report whether the change touches anything they declare.)
- Each worktree builds into its own `target/` inside the worktree. Never set `CARGO_TARGET_DIR`.
  Never point two trees at one build directory. Never delete a `target/` by hand.
- `cargo fmt -p <crate>`, never `cargo fmt --all` on a shared tree. Package-scoped gates only:
  `cargo clippy -p <crate> --all-targets --locked -- -D warnings`, `cargo test -p <crate> --locked`.
  The full gate runs once, later, on the integration branch.
- Never write under `.engineering/`, and no `aep plan artifact` write verb. Reads are fine.
- No `git commit`, `git add`, `git stash`, branch or `git worktree` command. Leave changes in the
  working tree; the coordinator commits.
- Nothing under `/tmp`. Scratch goes to the scratch root named in your brief.
- `CHANGELOG.md` is the coordinator's. Put the entry you would write into your report instead.
- No absolute home-directory path (`/home/<name>/`) in any file you write: the security scan fails
  every commit that carries one.
- Never name a customer, an employer or another coordination session in code, tests or docs.
  Use neutral nouns (`catalog`, `items`, `orders`).
