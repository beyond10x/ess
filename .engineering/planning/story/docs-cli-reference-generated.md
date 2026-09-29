---
format: aep.planning-md/3
id: story:docs-cli-reference-generated
kind: story
status: implemented
title: The CLI reference is generated from the command definition
relations:
- decomposes: epic:public-docs-overhaul
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T14:11:12Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-29T14:11:12Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-09-29T14:32:17Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

`website/docs/reference/cli.md` lists every `ess` command and flag the shipped CLI accepts, with its
help text and default, generated from the clap definition, and CI fails when the page and the CLI
disagree.

## Acceptance

- `cargo xtask cli-reference` writes the command sections between `<!-- ess-cli-begin -->` and
  `<!-- ess-cli-end -->`; prose outside the markers is kept byte for byte.
- `cargo xtask cli-reference --check` exits non-zero when a flag is added to the CLI and the page is
  not regenerated (a test adds one to a fixture command tree and sees the failure).
- The generated page names `project buildkit`, `project helm`, `conform author`, `conform select`,
  `--catalog`, `--chart`, `--stack-lock`, `--timeout` and `--retry-of`.
- Hidden aliases are not listed; `task projection-check` runs the check.

## Scope

- crates/edge/ess-xtask/src/cli_reference.rs (new)
- crates/edge/ess-xtask/src/main.rs
- Taskfile.yml
- website/docs/reference/cli.md
