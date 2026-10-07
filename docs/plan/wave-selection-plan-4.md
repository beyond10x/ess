# Wave 4 of `epic:one-selection-plan` — the interpreter, and the synthesis bytes pin

> **Status: approved 2026-10-08 by the operator (option A, "use multiple agents if possible").**
> Written on `integrate/selection-plan` `543967b56c` (waves 2 and 3 merged in; PR #488 and the
> wave-3 delivery not yet on `main`). Skill: `aep:implementing` 0.20.1, wave mode. The approval
> covers this wave and the two after it named in the operator's option A (synthesis, then the
> overlap witnesses); each gets its own page.

## 1. Pre-flight — measured 2026-10-08

| fact | evidence |
|---|---|
| root filesystem 93 G free | `df -h /`, after the wave-3 cleanup |
| ess linked trees for this epic: the integration tree only | `worktree status` |
| `aep 0.68.0` | `aep --version` |
| one measured unit build in `ess-conformance`: a 78-file candidate batch reached 19 G | the #487 tree |

## 2. Units

| unit | story | surface | depends on |
|---|---|---|---|
| U1 | `story:interpreter-reads-selection-plan` | `crates/verify/ess-conformance/src/interpret/execute.rs`, `interpret/execute/related.rs`, `interpret/execute/existence.rs`, a new test | wave 2 (implemented) |
| U2 | `story:synthesis-reads-selection-plan`, unit 1 of 5: the full suite-and-refusal bytes pin | a new test and fixture under `crates/verify/ess-conformance/tests/`; no source change | wave 2 |

The two touch disjoint files. U2 changes no synthesis code: it pins what synthesis writes before
units 2 to 5 change it (waves 5 and 6).

`aep plan artifact waves` places the interpreter story in its wave 4 because three draft stories
outside the epic (`feature-request-361`, `feature-request-362`,
`the-interpreter-executes-stored-field-guards`) also scope `interpret/execute.rs`; none is active.
The conductor is asked to keep the `ess` controller off those files while U1 runs.

## 3. Unit records

| unit | managed worktree id | branch | build directory | scratch root | stage |
|---|---|---|---|---|---|
| integration | `ess-selection-plan` | `integrate/selection-plan` | its own `target/` | — | at `543967b56c` |
| U1 | `ess-selection-plan-w4-u1` | `unit/selection-plan-w4-u1-interpreter` | its own `target/` | `~/.cache/ess-selection-plan/w4-u1-scratch` | to dispatch |
| U2 | `ess-selection-plan-w4-u2` | `unit/selection-plan-w4-u2-synthesis-pin` | its own `target/` | `~/.cache/ess-selection-plan/w4-u2-scratch` | to dispatch |

Dispatch types: `aep:implementor`, then `aep:adversary`, at most two passes each.

## 4. Commits this wave makes

The opening store commit (this page, the two stories moved to `active`); one commit per unit; the
two merges into `integrate/selection-plan`; the closing store commit. Delivery to `main` is the
wave-3 pull request or its successor, merged through the App on a green `Gate`. No tag, no release.
