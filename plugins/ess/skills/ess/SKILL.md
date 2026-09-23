---
name: ess
description: >-
  Start here for Executable System Specifications (ESS): install the `ess` binary, read the skill set that matches it, and route a spec-driven task to the right skill or agent. Use when the user points at the ESS repository or its documentation, asks how to adopt ESS, asks which ESS skill or agent to use, or starts spec-driven work — writing a new specification, retrofitting one onto an existing codebase, or holding an implementation to one — and no narrower ESS skill has been chosen yet.
---

# ESS for agents

ESS is a typed domain model that validates, compiles, projects schemas and contracts, and
synthesizes conformance suites. The `ess` binary is the authority; these skills tell you how to
drive it without guessing.

## 1. Have a binary, and read the guidance it was built with

```console
ess --version
```

No binary: install the release archive for your platform from
<https://github.com/beyond10x/ess/releases>, verify it against `SHA256SUMS`, and put `ess` on
`PATH`. The README's "Point your agent here" block has the exact commands.

The binary carries this whole skill set. It is the copy that matches the binary you run:

```console
ess skill                    # this page and an index of every skill and agent
ess skill specify            # one skill
ess skill agents/author      # one agent charter
ess skill --json             # the index, machine-readable
```

When you were loaded from an installed plugin, compare `ess --version` with the plugin version.
If they differ, trust `ess skill <name>` over the plugin copy: the binary's copy describes the
commands the binary has.

## 2. Pick the work

| The task | Skill | Claude agent |
|---|---|---|
| write a new specification, add an entity, validate, compile or project one | `specify` | `author` |
| give an existing codebase a specification it never had | `retrofit` | `retrofitter` |
| raise or audit what a conformance suite actually tests | `coverage` | `conformance` |

A Claude agent is a thin wrapper that runs its skill with a fixed charter. In Codex, or from the
CLI alone, invoke the skill directly; nothing is lost.

## 3. Rules that hold in every skill

- The compiler's output is the answer. Relay every refusal verbatim; never edit generated output
  around one.
- Imports never guess, and a draft an agent writes is an import. Mark what you could not read with
  `UNMAPPED:` and name it in the report.
- Spell a command by its area: `ess specify|generate|verify|infra <verb>`. The flat spellings are
  hidden aliases kept for pinned callers.
- Finish with the repository's own gate and report the exact command and exit status.

## Install the plugin instead

The same skills install as the `ess` plugin from the marketplace in the ESS repository, pinned to
the release tag that matches your binary:

```text
/plugin marketplace add https://github.com/beyond10x/ess.git#<version>
/plugin install ess@ess
```

Codex reads the same repository through `.agents/plugins/marketplace.json`:

```console
codex plugin marketplace add https://github.com/beyond10x/ess.git --ref <version>
codex plugin add ess@ess
```
