---
title: "0.51 — Branches selected by existence, and output that regenerates anywhere"
description: >
  0.51.0 generates the existence lookup a Rust server needs to choose between creating and
  refusing, and lets committed generated output regenerate in another checkout.
slug: existence-lookup-and-portable-output
tags: [release, ess]
date: 2026-10-01T23:30:00+02:00
release_tag: "0.51.0"
---

0.51.0 is a feature and fix release. Specifications, suites and deltas keep their formats.

{/* truncate */}

## Branches selected by whether the record exists

A command with an `existing_instance:` refusal beside a creation, or a creating
`unknown_instance:` branch beside an update, needs to know whether the record is already stored.
Until now the generated Rust server owed such a command as an obligation. From 0.51.0 it looks the
input identity up in the storage port before dispatch and selects the branch itself.

- A command whose lookup would not match its creations (another entity, another input field, an
  identity that is not an input) stays an obligation.
- Go, Web and Clap still refuse both forms by name.
- If your realization implemented one of these commands' `…Behavior` obligation, `Generated<P>`
  now runs the generated behaviour and no longer forwards that command to it
  (beyond10x/ess#310).

## Committed output regenerates in another checkout

Generated output committed with its `.ess-output` now regenerates in a clone, a second worktree or
CI, under any umask, when its owned files still have their recorded bytes:

- a regeneration that changes nothing leaves `state.json` byte-identical, and the first one that
  writes records the new root;
- a state carried without matching files refuses, lists them, and prints the steps to re-enroll
  with `ess generate output adopt`;
- a root replaced by a copy mid-transaction refuses recovery there (beyond10x/ess#306).
