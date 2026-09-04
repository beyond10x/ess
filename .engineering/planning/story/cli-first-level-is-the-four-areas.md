---
format: aep.planning-md/1
id: story:cli-first-level-is-the-four-areas
kind: story
status: implemented
title: ess --help shows the four areas; every flat verb stays as a hidden alias
summary: Group the 20 verbs under specify, generate, verify, infra with hidden flat aliases and a clap-tree test.
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: README.md
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/edge/ess-cli
- confidence: cited
  path: website/docs/reference/cli.md
revision: 6
---
# Story: `ess --help` shows the four areas; every flat verb stays as a hidden alias

## Context

`ess` has 20 top-level verbs: validate compile compose build realization runtime release stack
deployment inspect graph diff impact generate synthesize conform import project schema infra. The
crates were grouped under `crates/{specify,generate,verify,infra,edge}` in 0.11.1; the command
surface should say the same thing.

Target:

```
ess
├── specify    validate · compile · compose · inspect · graph · realization · runtime
├── generate   generate · synthesize · project · schema · build · release · stack · deployment
├── verify     conform · diff · impact
└── infra      infra · import
```

Every existing flat spelling (`ess validate --path …`) keeps working and prints the same bytes: a
hidden top-level alias forwards to the grouped command. No deprecation line on stdout or stderr.

## Acceptance

`ess --help` lists exactly four commands. Every leaf is reachable by its grouped path and by its
flat alias: a test enumerates the clap tree and asserts the alias set equals the leaf set. When a
command runs, flat and grouped spellings produce byte-identical stdout, stderr and exit status,
asserted over a representative set of leaves with fixtures and at least one clap-written refusal
per area; for clap-written refusals and `--help`, only the `Usage:` line names the path that was
typed, and the CLI reference and CHANGELOG say so. Where an area name is also a verb (`generate`),
area-level arguments combined with a sibling subcommand are refused with exit 2 as before the
change, and `ess generate --help` shows the verb's options beside the area's subcommands. `task
check` exits 0; the pre-existing `example-check` invocations are unchanged and grouped ones are
added. The CLI reference lists the grouped spellings and one paragraph on the aliases.

## Notes

Callers to check before release: agentide and service-sdk invoke `ess` verbs by name; the aliases
keep them working. Placement of `realization`/`runtime` under `specify` and `build`/`release`/
`stack`/`deployment` under `generate` follows the crates that implement them (ess-realization in
specify/, ess-deployment in generate/).
