# Downstream gaps — waves

Epic: `epic:downstream-reported-gaps` (serves `vision:O2`). Skill: aep:implementing 0.16.0, wave mode.
Waves from `aep plan artifact waves --kind story` on the typed scopes (2026-09-30).
Plan critics: rounds 1 and 2 recorded as `review-result:downstream-gaps-*-round-{1,2}`.

## Coordinator-owned files

- `CHANGELOG.md`, `changes/`, derived outputs (`generated/`, `suites/generated/`, `schemas/generated/`,
  `website/docs/reference/diagnostics.md`), `crates/edge/ess-cli/src/main.rs`, workspace `Cargo.toml`.

## Units

| Wave | Story | Branch | Tree id | Build dir | Scratch | Stage |
|---|---|---|---|---|---|---|
| 1 | feature-request-257 | impl/gaps-257 | gaps-257 | b10x-target/gaps-257 | ess-gaps/257 | merged 2a8ffd97b |
| 1 | feature-request-265 | impl/gaps-265 | gaps-265 | b10x-target/gaps-265 | ess-gaps/265 | dispatched |
| 1 | feature-request-271 | impl/gaps-271 | gaps-271 | b10x-target/gaps-271 | ess-gaps/271 | merged |
| 2 | feature-request-229 | impl/gaps-229 | gaps-229 | b10x-target/gaps-229 | ess-gaps/229 | dispatched |
| 2 | feature-request-275 | impl/gaps-275 | gaps-275 | b10x-target/gaps-275 | ess-gaps/275 | dispatched |
| 2 | feature-request-276 | impl/gaps-276 | gaps-276 | b10x-target/gaps-276 | ess-gaps/276 | dispatched |
| 3 | feature-request-266, -270 | | | | | planned |
| 4 | feature-request-267, -272 | | | | | planned |
| 5 | feature-request-268, -278 | | | | | planned |
| 6 | feature-request-269 | | | | | planned |
| 7 | feature-request-273 | | | | | planned |
| 8 | feature-request-279 | | | | | planned |
| 9 | feature-request-280 | | | | | planned |

Recomputed 2026-10-01 over the proposed stories after #275, #276, #278, #279 and #280 were scoped; waves 3-9 serialise on `crates/verify/ess-conformance/src/synthesize.rs`.
