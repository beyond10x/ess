---
title: Getting started
sidebar_position: 2
description: Install ess, write a one-file specification, pin the toolchain, generate a contract from it, and hold a small implementation to the conformance suite it obliges.
---

# Getting started

Getting started is four pages. Follow them in order; each continues in the directory the one
before it left.

1. [Install ess](./start/install.md): a verified release archive on macOS or Linux, `cargo
   install`, or an agent plugin; then pin the release for a project.
2. [Write your first specification](./start/first-specification.md): a one-file specification of a
   task list, validated, with an OpenAPI contract generated from it.
3. [Run your first conformance suite](./start/first-conformance-run.md): a TypeScript test package
   synthesized from the specification, a small implementation held to it, and the run that fails
   when the implementation is wrong.
4. [Use ess with an agent](./start/use-with-an-agent.md): the same path through a coding agent.

Then pick the runner for your implementation's language: [TypeScript](./start/runners/typescript.md),
[Go](./start/runners/go.md) or [Rust](./start/runners/rust.md).
[Explore the repository example](./start/explore-the-example.md) runs a larger specification from
a checkout.
