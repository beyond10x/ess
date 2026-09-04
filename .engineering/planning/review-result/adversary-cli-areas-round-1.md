---
format: aep.planning-md/1
id: review-result:adversary-cli-areas-round-1
kind: review-result
status: active
title: 'Adversary, round 1: ess CLI first level is the four areas'
relations:
- reviews: story:cli-first-level-is-the-four-areas
revision: 1
---
# Adversary, round 1 — story:cli-first-level-is-the-four-areas (ess)

Verdict: NEEDS-CHANGE. Cases executed 12 → 16, red 4. Origin: introduced 4 / undecided 2.
Agent: `adp:adversary` (opus). Cases added (kept): `crates/edge/ess-cli/tests/command_surface_adversary.rs`.

Attacked, did not break: all 33 leaves flat vs area on real fixtures identical except the clap-refusal
`Usage:` line; agentide's exact vectors (`ess compile …`, `ess generate --path … --kind docs|schema|
openapi|asyncapi --out …`) byte-identical to base; ess-xtask drift vector and `provenance::REGENERATE`
bare `ess generate` identical; `task example-check` exit 0 over 16 flat + 4 area invocations; hidden-ness
of flat aliases in every `--help`; `--version`, bare `ess`, `ess help <verb>` identical; mutants against
`AREA_LEAVES = 33` all caught; `ess infra diagnose|graph|diff` and `ess infra infra …` parse.

```findings
- file: crates/edge/ess-cli/src/main.rs
  line: 776
  category: boundary
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: >-
    ess generate accepts its own --kind/--out/--path alongside a sibling subcommand and silently
    discards them, exiting 0 without ever creating the --out directory, where the base commit
    refused the same invocation with exit 2.
- file: crates/edge/ess-cli/src/main.rs
  line: 779
  category: mutant
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: >-
    ess generate --path examples/billing synthesize runs synthesize against the default "." instead
    of the path the caller wrote, because generate_area ignores the area-level GenerateArgs.
- file: crates/edge/ess-cli/src/main.rs
  line: 744
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: >-
    hide_generate_flat_arguments removes --path/--kind/--include/--out/--format from ess generate
    --help and from its usage line, so the spelling used by provenance.rs, ess-xtask, agentide,
    README and cli.md is answered by a usage line saying the command takes no options.
- file: CHANGELOG.md
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: >-
    the claim that a flat spelling and its area path print the same bytes on both streams and that
    a caller cannot tell which one it ran is false for every clap-written refusal and for --help on
    all 32 leaves, which differ on the Usage line.
- file: crates/edge/ess-cli/tests/command_surface.rs
  line: 62
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: undecided
  message: >-
    the Acceptance asks for byte-identity asserted by a test that enumerates the clap tree, but the
    enumerating test compares argument shape only and the output test is a hand-written list
    covering 10 of 33 leaves whose single refusal case cannot observe the Usage-line divergence.
- file: Taskfile.yml
  line: 74
  category: acceptance
  severity: note
  verdict: CONFIRMED
  origin: undecided
  message: >-
    the Acceptance says example-check is unchanged and the diff appends four new invocations to it;
    the pre-existing invocations are untouched and the task exits 0, so this is a wording call for
    the coordinator rather than a defect.
```
