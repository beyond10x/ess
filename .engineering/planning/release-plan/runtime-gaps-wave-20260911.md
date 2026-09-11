---
format: aep.planning-md/1
id: release-plan:runtime-gaps-wave-20260911
kind: release-plan
status: draft
title: Deliver runtime-measured gaps 9 through 13 in PR29
relations:
- serves: vision:O2
- informed_by: task:runtime-adopter-gaps-9-13
revision: 1
---
## Authorized scope and process

Wave skill 0.8.1. Standing user authorization: fix all gaps, use multiple subagents, take new asks at high priority, deliver one final PR against main. No local full gate, no unchanged passing test reruns, no release or main merge in this wave. Intake/scoping is recorded in task:runtime-adopter-gaps-9-13 and five decomposing stories.

The computed scope graph reports collisions between gaps9/13 in domain command and compiler IR/resolve and between10/13 in Go runtime. Use the skill symbol-split fallback: gap9 owns ErrorSpec/RawErrorSpec/ResolvedError/error lowering and error diff/native HTTP codes; gap13 owns CommandSpec/Outcome/PayloadSource/response lowering, response observation and event completeness; gap10 owns predicate parsing and persisted predicate admission only. Require hunk headers and merge-tree dry run before integration. Root owns shared format selection/schema/docs integration and adopter verification.

Preflight: 74GiB free disk,10GiB free tmpfs. Max3 implementors, max2 compiler builds at once,2 compiler jobs each,sccache enabled. Root target cache is warm; exact changed CI test completed1pass in0.12s,0.70s compilation. Old private trees/evidence are preserved pending governed publication; do not discard their history. ESS primary user-staged planning remains untouched. Existing integration control changes remain uncommitted due Gates cumulative256MiB scan ceiling; no policy bypass. Opening control commit deferred until safely consolidated publication, source branches start exact published1fd6ba62.

## Units

| Story | Managed id | Branch | Build target | Scratch | State |
|---|---|---|---|---|---|
| error-wire-codes | wt-05ba20e2a7ca | impl/error-wire-codes | build-targets/gap9 | runtime-gaps/gap9 | implementation queued |
| refuse-misparsed-predicate-disjunctions | wt-25bf1c564b6d | impl/refuse-misparsed-predicate-disjunctions | build-targets/gap10 | runtime-gaps/gap10 | implementation queued |
| typed-response-outcome-payloads | wt-24004780cb22 | impl/typed-response-outcome-payloads | build-targets/gap13 | runtime-gaps/gap13 | design then implementation queued |
| verify-compiled-view-delivery | wt-643e4be649fc | fix/ess-pr29-ci | build-targets/integration | runtime-gaps | adopter provenance + CLI witness |
| adopt-truthful-conformance-counts | wt-643e4be649fc | fix/ess-pr29-ci | build-targets/integration | runtime-gaps | adopter report format + witness |

Local managed paths resolve through worktree registry; build/scratch paths under private priority-wave evidence root. Each implementor owns session ess-gapN-20260911, root owns ess-pr29-ci-20260911. Root is sole AEP writer. Public examples exclude private consumer names. Each finished unit gets an immutable adversarial review before correction/integration, at most2 attacks.

## Computed waves (verbatim)

```json
{
  "waves": [
    {
      "wave": 1,
      "artifacts": [
        {
          "id": "story:adopt-truthful-conformance-counts",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "crates/edge/ess-cli/tests/runtime_gap_report_counts.rs"
            }
          ]
        },
        {
          "id": "story:architecture-review-and-outlook",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "docs/reviews/2026-09-05-architecture-review.md"
            },
            {
              "confidence": "cited",
              "path": "website/docs/status/outlook.md"
            },
            {
              "confidence": "cited",
              "path": "website/docs/status/roadmap.md"
            },
            {
              "confidence": "cited",
              "path": "website/sidebars.ts"
            }
          ]
        },
        {
          "id": "story:browser-fixture-startup-deadline",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli/tests/support/browser.rs"
            }
          ]
        },
        {
          "id": "story:component-declares-its-settings",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/generate/ess-deployment/src/runtime.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-compiler/src/ir.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-domain/src/component.rs"
            },
            {
              "confidence": "cited",
              "path": "schemas/generated/ess.schema.json"
            }
          ]
        },
        {
          "id": "story:create-only-command-cannot-refuse",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "crates/generate/ess-gen"
            },
            {
              "confidence": "inferred",
              "path": "crates/generate/ess-synth"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-compiler"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-domain"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance"
            },
            {
              "confidence": "inferred",
              "path": "crates/verify/ess-diff"
            },
            {
              "confidence": "inferred",
              "path": "docs/design"
            }
          ]
        },
        {
          "id": "story:fixtures-carry-workstation-paths",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli/tests/fixtures/coverage-producers/input-catalog.json"
            },
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli/tests/fixtures/coverage-producers/semantic-plan.json"
            }
          ]
        },
        {
          "id": "story:integrate-source-driven-realizations",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "CHANGELOG.md"
            },
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli/src/main.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-synth/tests/feasibility.rs"
            },
            {
              "confidence": "cited",
              "path": "docs/design/review-format-catalog.md"
            },
            {
              "confidence": "cited",
              "path": "docs/design/review-rust-target-feasibility.md"
            },
            {
              "confidence": "cited",
              "path": "website/docs/reference/formats.md"
            }
          ]
        },
        {
          "id": "story:literal-representation-walk-exhaustion",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/specify/ess-domain/src/binding.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-domain/src/command.rs"
            }
          ]
        },
        {
          "id": "story:native-realization-ci",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": ".github/workflows/ci.yml"
            },
            {
              "confidence": "cited",
              "path": "Taskfile.yml"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/schema-contract/Cargo.toml"
            }
          ]
        },
        {
          "id": "story:normalization-equality-eligibility",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "docs/design/normalization-equality-eligibility.md"
            }
          ]
        },
        {
          "id": "story:primitive-canonical-serialization",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/generate/ess-gen/src/types.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-synth/src/go/http.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-synth/src/rust/wire.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-primitives/src/facts.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-primitives/src/node.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance/assets/coverage-admission.js"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance/src/go/runtime.go"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance/src/input.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance/src/report.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance/src/witness.rs"
            }
          ]
        },
        {
          "id": "story:release-status-publication-state",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/edge/ess-xtask/src/main.rs"
            }
          ]
        },
        {
          "id": "story:verify-compiled-view-delivery",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "crates/edge/ess-cli/tests/compiled_view_delivery.rs"
            }
          ]
        }
      ]
    },
    {
      "wave": 2,
      "artifacts": [
        {
          "id": "story:enum-variant-in-an-entity-invariant",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "crates/specify/ess-compiler/tests/billing.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-domain"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-domain/src/entity.rs"
            },
            {
              "confidence": "inferred",
              "path": "crates/specify/ess-domain/src/view.rs"
            }
          ]
        },
        {
          "id": "story:error-wire-codes",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "crates/generate/ess-synth/src/go/http.rs"
            },
            {
              "confidence": "inferred",
              "path": "crates/generate/ess-synth/src/rust/http.rs"
            },
            {
              "confidence": "inferred",
              "path": "crates/specify/ess-compiler/src/ir.rs"
            },
            {
              "confidence": "inferred",
              "path": "crates/specify/ess-compiler/src/resolve.rs"
            },
            {
              "confidence": "inferred",
              "path": "crates/specify/ess-domain/src/command.rs"
            },
            {
              "confidence": "inferred",
              "path": "crates/verify/ess-diff/src/change.rs"
            },
            {
              "confidence": "inferred",
              "path": "crates/verify/ess-diff/src/diff.rs"
            }
          ]
        },
        {
          "id": "story:java-conformance-target",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance"
            }
          ]
        },
        {
          "id": "story:refuse-misparsed-predicate-disjunctions",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "crates/specify/ess-primitives/src/facts.rs"
            },
            {
              "confidence": "inferred",
              "path": "crates/specify/ess-primitives/src/predicate.rs"
            },
            {
              "confidence": "inferred",
              "path": "crates/verify/ess-conformance/src/go/predicate.go"
            },
            {
              "confidence": "inferred",
              "path": "crates/verify/ess-conformance/src/go/runtime.go"
            }
          ]
        },
        {
          "id": "story:schema-unique-items-signed-zero",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "Cargo.lock"
            },
            {
              "confidence": "inferred",
              "path": "Cargo.toml"
            },
            {
              "confidence": "inferred",
              "path": "crates/generate/schema-contract/Cargo.toml"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/schema-contract/src/bundle.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/schema-contract/src/realize/normalize/source.rs"
            },
            {
              "confidence": "inferred",
              "path": "crates/generate/schema-contract/tests/schema_unique_items.rs"
            }
          ]
        }
      ]
    },
    {
      "wave": 3,
      "artifacts": [
        {
          "id": "story:the-generated-go-runtime-is-gofmt-clean",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "Taskfile.yml"
            },
            {
              "confidence": "inferred",
              "path": "crates/edge/ess-xtask"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance"
            }
          ]
        },
        {
          "id": "story:typed-response-outcome-payloads",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "crates/specify/ess-compiler/src/ir.rs"
            },
            {
              "confidence": "inferred",
              "path": "crates/specify/ess-compiler/src/resolve.rs"
            },
            {
              "confidence": "inferred",
              "path": "crates/specify/ess-domain/src/command.rs"
            },
            {
              "confidence": "inferred",
              "path": "crates/verify/ess-conformance/src/go/runtime.go"
            },
            {
              "confidence": "inferred",
              "path": "crates/verify/ess-conformance/src/target.rs"
            }
          ]
        }
      ]
    }
  ],
  "collisions": [
    {
      "a": "story:component-declares-its-settings",
      "b": "story:error-wire-codes",
      "path": "crates/specify/ess-compiler/src/ir.rs",
      "confidence": "inferred"
    },
    {
      "a": "story:component-declares-its-settings",
      "b": "story:typed-response-outcome-payloads",
      "path": "crates/specify/ess-compiler/src/ir.rs",
      "confidence": "inferred"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:enum-variant-in-an-entity-invariant",
      "path": "crates/specify/ess-domain",
      "confidence": "cited"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:java-conformance-target",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:error-wire-codes",
      "b": "story:literal-representation-walk-exhaustion",
      "path": "crates/specify/ess-domain/src/command.rs",
      "confidence": "inferred"
    },
    {
      "a": "story:error-wire-codes",
      "b": "story:primitive-canonical-serialization",
      "path": "crates/generate/ess-synth/src/go/http.rs",
      "confidence": "inferred"
    },
    {
      "a": "story:error-wire-codes",
      "b": "story:typed-response-outcome-payloads",
      "path": "crates/specify/ess-compiler/src/ir.rs",
      "confidence": "inferred"
    },
    {
      "a": "story:error-wire-codes",
      "b": "story:typed-response-outcome-payloads",
      "path": "crates/specify/ess-compiler/src/resolve.rs",
      "confidence": "inferred"
    },
    {
      "a": "story:error-wire-codes",
      "b": "story:typed-response-outcome-payloads",
      "path": "crates/specify/ess-domain/src/command.rs",
      "confidence": "inferred"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:literal-representation-walk-exhaustion",
      "b": "story:typed-response-outcome-payloads",
      "path": "crates/specify/ess-domain/src/command.rs",
      "confidence": "inferred"
    },
    {
      "a": "story:native-realization-ci",
      "b": "story:schema-unique-items-signed-zero",
      "path": "crates/generate/schema-contract/Cargo.toml",
      "confidence": "inferred"
    },
    {
      "a": "story:native-realization-ci",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "Taskfile.yml",
      "confidence": "cited"
    },
    {
      "a": "story:primitive-canonical-serialization",
      "b": "story:refuse-misparsed-predicate-disjunctions",
      "path": "crates/specify/ess-primitives/src/facts.rs",
      "confidence": "inferred"
    },
    {
      "a": "story:primitive-canonical-serialization",
      "b": "story:refuse-misparsed-predicate-disjunctions",
      "path": "crates/verify/ess-conformance/src/go/runtime.go",
      "confidence": "inferred"
    },
    {
      "a": "story:primitive-canonical-serialization",
      "b": "story:typed-response-outcome-payloads",
      "path": "crates/verify/ess-conformance/src/go/runtime.go",
      "confidence": "inferred"
    },
    {
      "a": "story:refuse-misparsed-predicate-disjunctions",
      "b": "story:typed-response-outcome-payloads",
      "path": "crates/verify/ess-conformance/src/go/runtime.go",
      "confidence": "inferred"
    }
  ],
  "unassessed": [
    "story:outcome-decided-by-environment"
  ],
  "cycles": []
}

```

## Runtime and migration progress

Gap10 fixture prefix /tmp/ess-gap10-predicate-<test-process-id>, exact process paths owned by gap10. No compiler outputs there. Root adopter managed wt-19e0cf57edd6, owning session ess-adopter-migration-20260911; primary files preserved. Isolated migration to source3 plus43 authored2 scenarios:10sourcefiles valid,43scenarios admitted,0refusals (exact candidate1fd6ba62). Actualreport2 run0.681s produced340total,96passed,228skipped,16failed; no conformance-success claim.

## Confirmed work and remaining integration

Gap9 source unit committed locally as f12afa8a8ce475c14d1c8e7f7c5222ba71da920f with exact bot author and committer; immutable error-wire-adversary-pass1 has no findings. Its shared format admission remains coordinator-owned and changes to diff4 jointly with response vocabulary. Gap10 source/browser implementation has one confirmed canonical roundtrip defect, recorded before correction in quoted-predicate-adversary-pass1; the same implementor is correcting it. Gap13 domain4cases pass after3 baseline failures; typed response runtime/native/schema integration continues. Root view CLI test1pass0.01s and targetedCLIClippy pass. Root report task delivery finding recorded in runtime-view-report-adversary-pass1; fixed absolute output path retains report2 independently of test exit. No full local gate.

Root owns source4, suite8/9 and diff4 routing, generatedschema/catalog and exact older-reader refusal fixtures. Corrected futureversion fixtures use source5, suite10 and diff5 for unsupported vocabulary; covered versions9 withoutrequiredcoverage still refuse. Existing old source/suite/delta bytes remain contractual.

Additional exact scratch path: /tmp/ess-gap13-runtime-20260911, runtimefixture only. All compiler outputs remain in assigned disk caches. Fresh remote main remains6b666e58; PRhead remains1fd6ba62 while fixes are consolidated for one finalPR update. No new gaps14+ present in current primarytaskrevision4.

## Local completion and publication boundary

All five runtime asks9–13 have implementation or verified adopter repair, and the additional Go selection primitive defect is corrected. Gap11 already serialized complete views; the actual reader now uses them. Gap12 already provided separate report2 counts; the actual invocation and report destination are corrected. This is established adopter evidence, not absence of an adopter. The older six capability stories keep their explicit downstream/context/released-version acceptance; no complete live binding or full target conformance is inferred from controlled fixtures.

Reviewed source units are f12afa8a8ce475c14d1c8e7f7c5222ba71da920f,09a16599b1e85532869fbcb837dd37cb9c5f9ad6 and47d4241057014c370be09f0ca751ad085b22035a, plus retained reviewed correction patches. Central integration enforces source4, suites8/9 and diff4; shared-file overlaps were inspected and explicitly combined. CLI build, focused predicate/version/selection and delta tests, direct typed-response admission, strict scoped Clippy, generated schema and task site-build passed. The original CI path-collision expectation correction passed separately. No full local or ownership gate was run by operator instruction. The remote PR correctness workflow remains the delivery check.

The exact current remote main remains6b666e58f2e87dd8798d27f935e9a012203296a3 and PR head1fd6ba62c497c1c0fd4354745e2f60e26859c235 after fresh bot fetch. Publish one consolidated reviewed candidate from main to the same PR, using an exact old-head lease. Consolidation is necessary because scanning repeated full planning-journal blobs across outgoing history exceeds the enrolled256MiB cumulative limit. No policy, hook or admission baseline is bypassed. Public planning projection is made through AEP CLI with generic aliases; original source events, private evidence and draft commits remain retained. Existing historical main records remain unchanged.

Private adopter proof stays in its managed checkout with its isolated patch and original-input custody. It is not pushed or deployed. Source release/main merge is outside the operator's final-PR boundary. Retire only the current owned source worker checkouts after the complete wanted candidate is published and recovery/evidence inspected; preserve unknown older records. Compiler concurrency remained at most2 builds with2jobs each. Exact agent token/tool totals are not exposed by this harness and are not invented.

<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 f5515646cca6e8f9b558de5dd8033d10abe23d8b6ae4b25a02d99836f22e9e0e, retained as local-evidence:runtime-gaps/publication-replay/snapshots/f5515646cca6e8f9b558de5dd8033d10abe23d8b6ae4b25a02d99836f22e9e0e.md. Source creation recorded at 2026-09-11T04:59:35Z. Private labels and local paths are projected to descriptive aliases.
<!-- public-import-provenance:end -->
