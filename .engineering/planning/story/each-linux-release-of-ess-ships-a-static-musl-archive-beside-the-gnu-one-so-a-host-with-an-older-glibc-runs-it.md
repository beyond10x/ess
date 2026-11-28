---
format: aep.planning-md/3
id: story:each-linux-release-of-ess-ships-a-static-musl-archive-beside-the-gnu-one-so-a-host-with-an-older-glibc-runs-it
kind: story
status: draft
title: Each Linux release of ess ships a static musl archive beside the gnu one, so a host with an older glibc runs it
summary: 'The owner''s A of 2026-11-25: static musl Linux archives, mimalloc on musl where faster, release-component takes the musl target; consumers pin the musl archives'
tags:
- issues-2026-11-26
- release
relations:
- serves: vision:O2
revision: 1
---
## Need

The Linux release archives of `ess` are linked against glibc. A host that runs an older glibc than the one the release runner built against warns or fails to start the binary: a build node on a distribution with glibc 2.34 is the case that was reported. A static binary has no such dependency.

The owner's order (2026-11-25, relayed by the operator session): static musl Linux releases for the Rust tools of the organization, with mimalloc where allocation-heavy; consumers pin the musl archives.

## Outcome

Each Linux release of `ess` ships a static musl archive beside the gnu one (x86_64, and aarch64 where an aarch64 Linux archive ships), in the existing archive-name pattern, with the checks the release already makes: the `package.yml` matrix, the preparation check of the archive names and `SHA256SUMS`, and the publish asset check. A CI step proves the binary static (`file` and `readelf` show no interpreter and no needed library) and runs `ess --version` in a plain `busybox` container with no network. The composite action `release-component` takes the musl target as an input, so a repository that uses it bumps its pin; a user that names no musl target keeps today's behaviour. The allocator of the musl build is mimalloc where it is measured faster.

## Acceptance

1. Red first: `ci_lanes.rs` fails without an `x86_64-unknown-linux-musl` archive beside each Linux gnu archive in the matrix, the preparation check and the publish check; the same file holds the count of archives and assets.
2. The static proof runs in CI: `file` says statically linked, the binary has no `INTERP` and no `NEEDED`, and `ess --version` prints the release version in a `busybox` container with `--network none`.
3. `release-component` has an `ess-target` input; a test of the delivery trust (`delivery_trust`) covers the input and its default.
4. mimalloc is the global allocator of `ess` on the musl target only; it stays only while one real synthesis workload is not slower with it than without.
5. `AGENTS.md` and `CHANGELOG.md` name the new archive count and targets wherever they said four archives.
6. Every new test runs in 60 s or less.

## Fit review

1. **Need.** A release binary that starts on any Linux host, whatever its glibc. A host with glibc 2.34 against a gnu archive built on a newer runner is the reproduction.
2. **Class.** *Gap* in the release: no archive is independent of the host's libc.
3. **Already expressible?** No: a consumer cannot build the static binary from the gnu archive; building from source is the only other route.
4. **Fit.** It adds rows to the existing release matrix, the existing preparation and publish checks and the existing composite action; it adds no source format.
5. **Second adopter.** A container image based on `busybox` or `alpine`, which has no glibc.
6. **Cost.** Two more archives and assets per release (six archives, seven assets with `SHA256SUMS`), a longer package job, a musl toolchain on the runner; the musl allocator is slower on allocation-heavy work, hence mimalloc.
7. **Considered.** (a) Change nothing: older hosts keep failing. (b) Build the gnu archive against an older glibc: it moves with the runner image. (c) Static musl archives beside the gnu ones (chosen, the owner's A).

## Decisions

- The owner's A of 2026-11-25: static musl Linux archives for all organization Rust tools, mimalloc where allocation-heavy; consumers pin the musl archives.
- The gnu archives stay beside them; the install page, the README and the toolchain installer keep naming the gnu archives until a release that carries a musl archive exists (the coordinator, 2026-11-26).

## Delivery

The owner's A of 2026-11-25; branch `issues/open-2026-11-26-e` at `08be5a4838`; mimalloc on musl only (one synthesis workload median 151.4 s → 128.0 s). Merged into the open-issues wave; not released when written.

Measured on the way: `ess verify conform synthesize` of the largest committed example takes under 0.1 s with or without mimalloc, so no committed model shows a difference; the repository's timing model (48 flags, 24 readings, 270 scenarios) took 67.9 s on gnu and 193.1 s on musl before mimalloc.

## Scope

Cited: `.github/workflows/release.yml`, `.github/workflows/package.yml`, `.github/workflows/publish.yml`, `.github/actions/release-component/`, `crates/edge/ess-xtask/tests/ci_lanes.rs`, `AGENTS.md`, `CHANGELOG.md`, `crates/edge/ess-cli/Cargo.toml` and `crates/edge/ess-cli/src/main.rs` (the musl-only allocator).
