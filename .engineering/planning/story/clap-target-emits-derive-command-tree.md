---
format: aep.planning-md/3
id: story:clap-target-emits-derive-command-tree
kind: story
status: draft
title: The clap target emits the command tree in derive form, with typed handlers
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#476
relations:
- serves: vision:O2
revision: 2
---
## Outcome

Feature request https://github.com/beyond10x/ess/issues/476: the clap target can emit its command
tree in clap derive form, with handlers that take typed argument structs instead of
`clap::ArgMatches`.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md`; read on 0.55.0 sources, nothing built.

1. **Need.** A repository whose rule is clap derive cannot use `ess generate synthesize --target clap`:
   it emits a builder tree (`tree.rs`) and a `Handler` trait over `&clap::ArgMatches`
   (`handler.rs`), and its `TARGET.md` records that as a weakening of "a command's input arrives as
   its declared type". The adopter writes the CLI by hand and holds it to the specification with a
   test that walks the clap tree against `ess specify compile` output. Requester's syntax, theirs:
   `--target clap --form derive` or a separate `clap-derive` target.
2. **Class.** Gap: the target's own record names the weakening.
3. **Already expressible?** No. The other CLI route, an `ess-cli/*` binding with `ess generate cli`,
   also builds its tree with the builder API (`crates/generate/ess-cli-project/src/runtime.rs:255`).
4. **Fit.** A form option on the existing clap target reuses the `cli:` block as it is. The issue's
   own open points are design questions, not syntax: flag spelling (field name today, kebab case in
   derive by default), required flags (every non-optional field today), view presentation options,
   and the undeclared `completions` subcommand. Presentation options declared in the `cli:` block
   would be new authored surface and a source-format bump.
5. **Second adopter.** Any Rust repository that standardises on clap derive.
6. **Cost.** (b) below: one target form, generated-code tests, `TARGET.md`. (c): a source-format
   version and authored keys on top.
7. **Considered.** (a) change nothing; (b) a derive form that keeps today's semantics (same flags,
   same required set, typed handler structs) and no new authored keys; (c) (b) plus `cli:` block keys
   for input-from-JSON and view formats.

## Decisions

- **accept, redesigned**: (b) first. The flag-spelling and required-flag points are settled when
  the story is scoped, against the clap target's existing conformance tests; (c) needs its own
  request. Not scheduled while the issue backlog is open; the issue is closed with a pointer to this
  story and reopened when it is.
