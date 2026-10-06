---
format: aep.planning-md/3
id: review-result:ess-054-466-adversary-1
kind: review-result
status: active
title: 'Adversary pass 1, #466 trailing argument list'
tags:
- ess-0.54.0
relations:
- reviews: story:feature-request-466
revision: 1
---
verdict: GREEN

I couldn't break the change. I wrote 17 new cases and all of them pass; no finding.

unit: ESS #466 `{kind: trailing}`, uncommitted tree at `~/.local/state/worktree/trees/b10x/ess/ess-054-466-20261006` (base a231697e6)
cases: executed 29→46, red 0. The 29 comes from the implementor's `~/.cache/ess-054-466/green-full.log`: binding 18, trailing_arguments 6, projection 5.
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 directory
needs-coordinator: two items, listed at the end.

**1. Diff stat**

`git diff --stat` shows the same 6 tracked files as the tree I was handed, with 261 insertions. I added only two untracked test files. No implementation file was touched.

**2. Cases (each file run alone first)**

| file | cases | now |
|---|---|---|
| `crates/specify/ess-cli-contract/tests/adversary_466_pass1.rs` | 8 | green |
| `crates/generate/ess-cli-project/tests/adversary_466_pass1.rs` | 9 | green |

Commands: `cargo test --locked --offline -p ess-cli-contract --test adversary_466_pass1` and the same with `-p ess-cli-project`. Both exited 0.

The project file's first run was red, 2 failed, but the fault was mine. My expected JSON literals put `ok` first, and serde_json sorts keys alphabetically. I changed those two checks to compare parsed JSON, and both now pass.

**3. Suite run (after the cases existed)**
- `cargo test --locked --offline -p ess-cli-contract --test binding --test adversary_466_pass1`: 18 + 8 passed, exit 0.
- `cargo test --locked --offline -p ess-cli-project --test trailing_arguments --test projection --test adversary_466_pass1`: 6 + 5 + 9 passed, exit 0.

**4. Judgement findings:** none.

**5. What I attacked and could not break (all are now green controls)**

| area | what was checked |
|---|---|
| Binding reader | All 10 extra keys tried on `{kind: trailing}` are refused, and so is the bare `source: trailing`. The plan the generated package reads back from `binding.json` is closed too. |
| Admission | 12 further field types are refused with the trailing message. "At most one" applies per command, not per binding. The trailing source counts as no stdin source. It does not fill a positional gap, cannot bind a dynamic String field, and a renamed field is bound under its wire name. |
| Runtime | A required positional is filled only before `--`. Group paths and both aliases work. Help after a required positional shows correctly. `--help`, `-V` and `--config`/`--output` after `--` are passed on as values and change nothing in the process context. JSON-mode refusals and `invalid_input` carry no trailing word. |
| Plan file and generated files | Reading `binding.json` back gives the same outputs and handler inputs as the in-memory plan. `project()`, `completions bash` and clap's `debug_assert` all succeed. |
| Property | 400 runs with a fixed seed over 41 awkward words: every word after `--` reaches the handler unchanged. |
| Unchanged bytes | I regenerated the fixture with the installed `ess 0.53.0`. Its `binding.json` is byte-identical to the unit's `fixtures/binding.json`, so the "unchanged" test checks against real 0.53.0 bytes, not a copy of its own output. |
| Missed mutants | Each of six mutations I worked through would turn a test red: removing `last(true)`, a wrong trailing index, a missing `continue`, the count threshold, the shape check, and allowing the optional list. Changing `num_args(1..)` to `(0..)` makes no observable difference, so no test can catch it. |

**6. Path written outside the worktree**
- `~/.cache/ess-054-466/adversary-466-pass1/` holds the 0.53.0 regeneration (`spec/system.yaml`, `cli.yaml`, `gen053/`, about 72 KB). The build output went to the assigned `/dev/shm/ess-054/466`.

**Needs-coordinator**
- **CHANGELOG:** the story's Decisions say "the CHANGELOG says so", but the diff has no CHANGELOG entry. Commit a231697e6 suggests you write changelog entries after merging, so I'm passing this to you.
- **Clippy:** the brief limited me to `cargo test`, so I didn't run clippy on my two files. If you keep them, run `cargo clippy --all-targets` on both packages first.

**Files I created**
- `~/.local/state/worktree/trees/b10x/ess/ess-054-466-20261006/crates/specify/ess-cli-contract/tests/adversary_466_pass1.rs`
- `~/.local/state/worktree/trees/b10x/ess/ess-054-466-20261006/crates/generate/ess-cli-project/tests/adversary_466_pass1.rs`

```findings
[]
```
