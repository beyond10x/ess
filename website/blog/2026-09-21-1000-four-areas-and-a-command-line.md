---
title: "0.12–0.14 — four areas, and a command line as a declared surface"
description: >
  `ess --help` lists four commands, one per area, and every flat spelling stays a hidden alias that
  prints the same bytes. A component can declare that its callers are people at a terminal, and
  `--target clap` synthesizes the parser from that declaration. Release execution gets an explicit
  executor boundary.
slug: four-areas-and-a-command-line
tags: [release, ess]
date: 2026-09-21T10:00:00+02:00
release_tag: "0.14.0"
release_commit: a25090533d2dcaf8e3de6736e7cced236571b563
---

Three releases about surfaces: the one this tool presents, and the one a specified component
presents to a person at a terminal.

{/* truncate */}

## The first level is four words

`ess --help` lists `specify`, `generate`, `verify`, `infra` and nothing else. The crates had moved
under `crates/{specify,generate,verify,infra}/` in 0.11.1; the command surface now says the same
thing, so **where a thing is implemented and how it is spelled are one fact instead of two**.

Every flat spelling survives as a hidden alias of its area path. `ess validate --path .` is
`ess specify validate --path .`: the same arguments, and when the command runs, the same stdout, the
same stderr and the same exit status, with no deprecation line anywhere. For a refusal the argument
parser writes, and for `--help`, only the `Usage:` line differs — it names the path that was typed,
which is what the flat spelling has always printed.

Both spellings are mounted from one definition in the derive, so they cannot drift apart, and a test
enumerates every leaf of the clap tree and asserts the pairing rather than checking a list somebody
maintains. Nothing is deprecated, and no pinned caller needs changing.

## Release execution gets a boundary, not a capability

0.13.0 makes the implementation repository the owner of its semantic, realization, build, runtime and
release units through `ess-component/1` and canonical `ess-component-ir/1`, and carries the complete
verified release chain as one OCI payload in `ess-release-bundle/1`.

It also, for the first time, invokes things: `publish` uses ORAS at the credential edge, `fetch`
requires a **digest-pinned source** and revalidates canonical bytes, `build execute` invokes Docker
Buildx Bake, and `deployment reconcile` computes the affected release set, follows the rollout DAG,
fetches charts by digest and invokes Helm **only for changed releases**. It refuses implicit removal.

The design point is in the phrasing of the changelog: *side effects are an explicit CLI executor
boundary*. The deployment compiler and every projection stay deterministic and offline and reuse
their exact IR. What changed is that there is now one named place where a command may reach the
world, rather than a capability spread through the projections.

## A component whose callers are people

`reached_by: command_line` is a third answer to the question `in_process` and `network` already
answer — *where are the callers* — and like them it names no wire, no port, no path and no verb.
What it states is the one fact neither of the others can: **the surface leaves the process as a
grammar rather than as a call**, and a command-line caller is deployed with the binary and is not a
program.

A `cli:` block then says where each accepted command sits in that grammar. Paths within a group are
derived from `naming.wire`, as OpenAPI paths already are. Grouping is declared, because which
*activity* a command belongs to cannot be derived from anything the model holds.

Eight refusals come with it, and they are the reason the tree is declared here rather than written
beside a parser: a block on a component reached another way, a command-line surface with no block, a
placed command the component does not accept, a placed view no domain it owns projects, an accepted
command or projected view the tree places nowhere, and either placed twice.

The view half exists because of a defect found in a consuming repository rather than imagined here.
`connectors` serves `kubernetes.workloads` from its local daemon and had no command-line verb that
reads a datasource at all — so an operator on that machine could not reach a projection the process
beside them was already publishing. Nobody wrote that down until somebody noticed.

`ess generate synthesize --target clap` emits the parser, the handler obligations and the completion
scripts from the declaration.
