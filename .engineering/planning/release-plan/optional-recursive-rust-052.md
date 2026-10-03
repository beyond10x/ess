---
format: aep.planning-md/3
id: release-plan:optional-recursive-rust-052
kind: release-plan
status: draft
title: Release ESS 0.52.0 for generated runtime contract adoption
relations:
- derived_from: story:optional-recursive-rust
revision: 1
---
## Intent

Release ESS0.52.0 so the operator-approved EKR implementation can pin a verified generator rather than an unreleased local build. Include already merged main changes and the reviewed optional-self recursion/documentation emitter correction. This follows the explicit generation prerequisite; no EKR feature completion is claimed.

## Candidate and review

Optional recursive Rust source is126c2b3905d0f4279086b9d3030096147955dfec, integrated with current main f5be9eafd3190e05e4cc053f260112cfc634e7d5 at baaa6b090edaa30247c3904555390aa4f11449dd. Local package380 passed,0 failed,1 existing ignored; strict Clippy, formatting, ci-lint and actual EKR fresh compilation in both layouts passed before the main merge/version bump. The independent adversary found a bare-CR failure; the permanent regression and unchanged adversary case pass after correction. Publication copies of both reports redact personal path prefixes and preserve original hashes; exact originals remain private.

## Required completion

Recheck affected code after integrating main and changing version, publish bot-authenticated candidate and require common security plus repository Gate. Before tagging run task check and task site-lab on the exact source commit. Verify exact tag, required release checks, published GitHub Release and required native archives/checksums. Then verify downloaded CLI version and regenerate actual EKR contracts. Source release completion does not wait for Atlas/Website; documentation publication is pending unless separately observed.

## Scope

Release metadata Cargo.toml/Cargo.lock/CHANGELOG.md, existing verification/release workflows unchanged, and AEP evidence. No consumer pin promotion except the EKR generator pin explicitly requested by this task. No other repository release, facade deployment or shared delivery-control change.
