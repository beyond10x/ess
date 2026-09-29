---
title: Install ess
description: Install the ess command from a verified release archive on macOS or Linux, with cargo, or through an agent plugin, and pin the release a project uses.
---

# Install ess

There are three ways to get `ess`: a verified release archive, `cargo install` from the release
tag, or a coding agent that installs it for you. Each gives you the same binary.

Every file the Start-here pages have you write and every `ess` command they have you run is also run
by the repository's own tests, and the output shown is the output they compare. Where a command
prints more than is worth reading, `…` marks the lines left out. Paths print as they do when you
start in your home directory, `~`.

## From a release archive

Each release publishes the `ess` binary for four targets:

| Machine | Target |
|---|---|
| Linux x86-64 | `x86_64-unknown-linux-gnu` |
| Linux ARM64 | `aarch64-unknown-linux-gnu` |
| macOS Intel | `x86_64-apple-darwin` |
| macOS Apple Silicon | `aarch64-apple-darwin` |

Download the archive for your machine and the release's `SHA256SUMS`, check the archive against
it, then extract it. `SHA256SUMS` covers all four archives; filtering the one filename lets the
checksum tool verify the archive you downloaded without treating the other three as missing.

On Linux, with `sha256sum`:

```shell-session
$ version=0.46.0
$ target=x86_64-unknown-linux-gnu
$ archive="ess-${version}-${target}.tar.gz"
$ base="https://github.com/beyond10x/ess/releases/download/${version}"
$ curl --fail --location --remote-name "${base}/${archive}"
$ curl --fail --location --remote-name "${base}/SHA256SUMS"
$ grep -F "  ${archive}" SHA256SUMS | sha256sum --check
ess-0.46.0-x86_64-unknown-linux-gnu.tar.gz: OK
$ tar -xzf "${archive}"
$ "./ess-${version}-${target}/ess" --version
ess 0.46.0
```

On macOS, with `shasum`, which macOS ships in place of `sha256sum`:

```shell-session
$ version=0.46.0
$ target=aarch64-apple-darwin
$ archive="ess-${version}-${target}.tar.gz"
$ base="https://github.com/beyond10x/ess/releases/download/${version}"
$ curl --fail --location --remote-name "${base}/${archive}"
$ curl --fail --location --remote-name "${base}/SHA256SUMS"
$ grep -F "  ${archive}" SHA256SUMS | shasum -a 256 --check
ess-0.46.0-aarch64-apple-darwin.tar.gz: OK
$ tar -xzf "${archive}"
$ "./ess-${version}-${target}/ess" --version
ess 0.46.0
```

Use `target=x86_64-apple-darwin` on an Intel Mac. Stop if the check does not print `OK`. Then put
the extracted `ess` on your `PATH`; the rest of these pages call it `ess`:

```shell-session ess-tutorial
$ ess --version
ess 0.46.0
```

## With cargo

With a Rust toolchain, build and install the release from its tag. The package is `ess-cli`; the
binary it installs is `ess`:

```shell-session
$ cargo install --locked --git https://github.com/beyond10x/ess --tag 0.46.0 ess-cli
```

This also covers a machine outside the four targets above. To work from current `main` in a
checkout of the repository, run `cargo build --locked --release --bin ess` and use
`./target/release/ess`.

## Through an agent

If you work with a coding agent (Claude Code or Codex), let it install `ess` and its plugin. Tell
the agent:

> Set up Beyond10x for specifications: follow
> https://github.com/beyond10x/agentplugins/releases/latest/download/SETUP.md

The setup installs the `ess@b10x` plugin from
[`beyond10x/agentplugins`](https://github.com/beyond10x/agentplugins) and this CLI (prebuilt, or
with `cargo`), asking before each change. Then `/ess:init` takes the first step in your project.
[Use ess with an agent](./use-with-an-agent.md) says what the plugin does after that.

## Pin the release for a project

A project names the release it is maintained with, so a later upgrade of the `ess` on your `PATH`
does not change what the project generates. Make a project directory with an `ess-inputs.yaml`, the
manifest naming the files that make up the specification. The next page writes
`spec/system.yaml`:

```shell-session ess-tutorial
$ mkdir -p tasks
$ cd tasks
```

```yaml ess-tutorial file=tasks/ess-inputs.yaml title="ess-inputs.yaml"
format: ess-inputs/2
specification: [spec/system.yaml]
scenarios: []
```

`--pin` downloads the release, verifies it against the release's `SHA256SUMS`, caches it, and
writes the pin into the nearest `ess-inputs.yaml`:

```shell-session ess-tutorial
$ ess specify toolchain install --pin 0.46.0
installed ess 0.46.0 at ~/.cache/ess/toolchains/0.46.0/ess
pinned ~/tasks/ess-inputs.yaml: requires: ess 0.46.0
```

The manifest now reads:

```yaml ess-tutorial expect=tasks/ess-inputs.yaml title="ess-inputs.yaml"
format: ess-inputs/2
requires: ess 0.46.0
specification: [spec/system.yaml]
scenarios: []
```

From now on, any `ess` run in `tasks` or below it runs 0.46.0 from the cache, whatever release is
on your `PATH`, until you move the pin. `ess specify toolchain which` prints the release that would
run and why.

The first level of `ess` is four areas: `specify` (write and resolve a specification), `generate`
(turn it into artifacts), `verify` (hold an implementation or a later revision to it) and `infra`
(read an observed cluster). `ess <area> --help` lists what each holds, and the
[CLI reference](../reference/cli.md) describes every command.

Next: [Write your first specification](./first-specification.md).
