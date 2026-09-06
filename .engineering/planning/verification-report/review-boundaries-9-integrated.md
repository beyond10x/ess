---
format: aep.planning-md/1
id: verification-report:review-boundaries-9-integrated
kind: verification-report
status: draft
title: Composition contract integrated verification
relations:
- verifies: story:review-composition-contract
revision: 1
---
## Exact source and result

The complete declared ESS integration gate, site-build and planning validation passed on4777c1de3a80ea7645a255e4e229ad30a8a5cc8e. All10 direct command exits are0; the workspace test lane executed1983 cases,0failed/ignored,170 summaries. Source manifests were identical before and after the run. This incorporates published b55efa4 native-codec/positional-normalization work and the final reviewed composition correction75df9e6d.

| Lane | Exit | Seconds | Log SHA256 |
|---|---:|---:|---|
| fmt-check | 0 | 1.280 | e96f1cab699f6a4bb3c4dd64f88e7ce3dfb5aaaa299891416e29335e005cd893 |
| clippy | 0 | 3.883 | e98cc0452db268239d96b2c4e6ede33888bffe853688fa47c5e4566a48661f7d |
| test | 0 | 97.446 | 341dc71810fc142a8c147002d38eb0aadf659a26b9c37c06eb4488c6ff01a74a |
| doc-check | 0 | 2.564 | d46b6bf359ad9a1e73d08eea7df98c46327281567a61ac7b112a1456980b78a7 |
| example-check | 0 | 5.618 | ed589817ee6baa31cfa20701f4bdc592332a146236d0eb890353581a9e7876f5 |
| projection-check | 0 | 0.588 | d67703f04c862e265e9b0d6276196cbaa9e98ec58d32cc62b88b08fff8e94f66 |
| release-check | 0 | 0.096 | d271889a56fa46d206d407608e43481422caab5f83b671671f67ab563de1b69c |
| action-check | 0 | 0.047 | ce62e1f35e7bcdfebdd7ed28735f5d7e206b8f32f6f5fc8e2172e216eac900b9 |
| site-build | 0 | 14.679 | 37f066dcfef75eb7320a8adfe51945cafed28b07d0aebb60cd6d91e8ad2b7747 |
| planning | 0 | 15.324 | 55c84af6787558844d7a3d6818e3512e5212cbce6aae864201b6801a09e3ade1 |

## Review and command evidence

The immutable composition source reviews are review-result:composition-contract-adversary-pass-1 and review-result:composition-contract-adversary-pass-2. Their comparison has no carried, new or resolved findings. The first attack retained two additive tests: private descriptor construction/fields refuse in separately compiled callers, and a scratch payload-dropping mutation fails the original exact-byte fixture. The package executed12 cases and the original downstream client fixture3 distinct cases.

Root retained the first copyable-command failure as verification-report:composition-cli-example-first-check. Creating its output parent corrected the setup. The final review predeclared then executed fresh-output success and a conflicting-destination refusal, preserving companion-output absence and existing bytes. All generated client files and the companion plan matched the original corpus. No production statement, persisted format, generated golden byte or dependency changed in the composition unit.

The earlier1948-case full run atd565eb0 remains attributed to that earlier source. The fresh1983-case run is required because main advanced. Root preserved all1319 incoming journal lines,148 unrelated artifacts and8 local artifacts through24 semantic CLI commands replaying29 events. No journal tails were concatenated.

Raw evidence: target/review-boundaries-9/gate-4777c1de3a80/{source.json,source-manifest.json,results.json,complete.json} and every named log in the coordinator; immutable unit reports and manifests under its assigned target/review-boundaries-9. Final gate ended 2026-09-06T14:13:55.329933+00:00.

This record establishes source verification. Public source publication, deterministic Website/Atlas delivery, live comparison and task-owned managed cleanup are separate pending observations. No release, tag, version bump, default switch or installed-binary replacement is selected.
