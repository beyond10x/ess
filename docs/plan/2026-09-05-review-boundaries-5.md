# ESS review boundary remediation — wave 5

Standing-approved proposal selecting two independently scoped P1 stories after the completed wave 4. Both serve vision:O2. Nine of the original31 remediation stories are implemented; this wave does not claim all17 review findings resolved.

**Skill version 0.7.0** — aep-drive:wave. Dispatch roles are aep-drive:implementor and aep-drive:adversary; independent scoping uses aep-drive:story-scoper. Native plugin agent types are unavailable in this harness, so existing collaboration threads receive the exact installed charters as the recorded adapter deviation. No model override or human-review claim is made.

## Authority and selection

Approval-record:review-remediation-standing-implementation and approval-record:review-remediation-standing-publication cover the selected unit and correction/test commits, integration merges, closing store commit, merge to main, publication of green remediation source and task-owned managed cleanup. Necessary downstream preparation and validated publication remain covered by the operator's instruction to handle all wave implementations. No release version or tag is authorized.

The clean primary ESS main and advertised main are6616b26fe41548af9cb7ff9cf833ae977883f625. The existing managed coordinator wt-752828a285ba is reused on wave/review-boundaries-5; merged wave4 branch was deleted non-forcibly. All five wave4 implementation/support trees and their targets are removed with verified scratch archives. The original review/outlook tree and unrelated worktrees remain untouched. Clean Atlas authority wt-90ec680c6073 equals advertised a58a98048d5adf60b96d86a00220aeb0f10f5218; the final full fence for ESS6616b26 exited1 at2026-09-05T18:33:55.197477Z after182.709s. Its115 Rust tests passed and only the three previously recorded sibling manifest/portal/map failures remained; raw evidence is in coordinator target/review-boundaries-4/final-publication-fence.

Fresh AEP list/graph/blocked/waves commands all exited0. There are25 drafts before selection (22 remediation plus3 unrelated legacy), no blockers, no dependency cycles and no unassessed scope. The full draft computation below contains11 sets and121 excluded pairs. This is a computed selection; the greedy set order is not a priority order. The selected proposed-set computation separately reports one pair and no collisions.

Selected story:review-rust-target-feasibility addresses compiler-invalid or ambiguous target representations using target-specific feasibility and explicit pre-write refusal. Its package/CLI reservations are cited; its new binding design is inferred. Current scope has high ownership confidence. Source naming, cycle and direct public emitter decisions must be written before implementation. A bounded external consumer inventory and checked public API decision are coordinator-owned. SDK's historical facade caller drops the target report; source compatibility alone cannot establish that it handles a new Rust refusal. Any required relying-party migration must be governed and exercised before the affected downstream pin is adopted; no SDK upgrade or compatibility result is assumed.

Selected story:review-format-catalog documents the actual currently persisted formats, decoder/semantic-check distinctions and byte identities, and corrects the change guide's stale diff/impact defaults and nonexistent --generated option. Existing guide/sidebar scope is cited; new pages are inferred. Actual edits are the two named public pages, sidebar and internal catalog; the broader website/docs token is scheduling conservatism. Its final review must reflect any actual Rust-report semantics merged in this wave.

Left out: counts remains P0 but requires the scoped AEP domain/fact/policy and both-reader migration before opt-in report2. Coverage follows its bound design and counts. Consumer coverage is freshly scoped but still requires binding authoritative inventory and actual behavioral-evidence rules; it is not padded into this wave. Fuzz is freshly scoped, but mandatory seed crashes have not been measured and likely share the Rust feasibility owner. Expression, observation, primitive/OpenAPI and other CLI/synth stories collide or still need substantial binding choices. Remaining recovery, vocabulary, provenance and public claims stories stay in the remediation plan. The3 unrelated legacy drafts are not claimed.

## Resources and ownership

N=2 implementors within3 worker slots and default model budget4; a numeric budget was previously requested and not supplied. Measured resources are recorded below. Prior unit targets measured904332 KiB (delivery) and297096 KiB (TypeScript); previous full coordinator target was about2.2 GiB. The8 GiB disk reserve is enforced when units return. Generated adversarial Rust fixtures may cost more than package checks; use bounded compile matrices, record their sizes and stop expansion before the reserve is crossed.

The reusable coordinator and clean authority targets are explicitly retained while the overall remediation continues. No removed wave4 unit build directory is reused. The coordinator-owned foreground sccache service remains at target/w4-cache.sock, queried successfully (10 GiB bounded cache, no cache read/write errors); only cache access is shared. Each new unit owns target plus target/review-boundaries-5 scratch beneath its own managed checkout. TMPDIR is its own target. CARGO_TARGET_DIR is forbidden; incremental/dev-debug/test-debug/cache-rustc-info are0, Cargo is offline by default. The coordinator owns eventual service shutdown and observes termination.

| Story | Branch | Managed ESS path suffix | Build/temp | Scratch | Stage |
| --- | --- | --- | --- | --- | --- |
| review-rust-target-feasibility | impl/review-rust-target-feasibility | review-rust-target-feasibility | target | target/review-boundaries-5 | implementation f9a7cf7;178 package tests; independent pass1 running |
| review-format-catalog | impl/review-format-catalog | review-format-catalog | target | target/review-boundaries-5 | pass1 corrected in d8dedad; frozen Rust format addendum underway |

All units fork from the same opening integration commit, recorded before creation. Root owns every AEP mutation and Git/worktree lifecycle. Implementors write only assigned source/tests/design files and leave uncommitted handoffs. Rust work runs package-scoped real compiler/regression checks; documentation work uses source-backed inventory/link checks with no invented prose tests. Adversaries use distinct threads, may add tests and never change production. Every full review is recorded immutably before routing; maximum2 full attacks. Root records any bounded final correction separately.

The opening commit receives all cheap applicable checks before unit creation. One complete eight-step offline ESS gate plus site build and planning validation runs on the final merged source. Public catalog publication additionally requires ESS source published first, a managed Website deterministic source-lock refresh, Atlas-owned snapshot, and Website/Atlas delivery gates. A local Docusaurus build is not public delivery. Final delivery status and any baseline external fence failures must be recorded honestly.

## Measured pre-flight resources

```json
{
  "disk_available_bytes": 90789679104,
  "disk_floor_bytes": 8589934592,
  "available_ram": "48849400 kB",
  "coordinator_target_kib": 2229556,
  "implementation_agents": 2,
  "worker_slots": 3,
  "default_model_budget": 4
}
```

## Complete draft computation

```json
{
  "waves": [
    {
      "wave": 1,
      "artifacts": [
        {
          "id": "story:a-skipped-scenario-is-not-a-failed-one",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "Cargo.lock"
            },
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance"
            },
            {
              "confidence": "inferred",
              "path": "website/docs"
            },
            {
              "confidence": "cited",
              "path": "website/docs/guides/verify-conformance.md"
            }
          ]
        },
        {
          "id": "story:fuzz-the-specification-surface",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "fuzz"
            }
          ]
        },
        {
          "id": "story:review-execution-recovery-design",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "docs/design/review-execution-recovery.md"
            }
          ]
        },
        {
          "id": "story:review-typed-diagnostics",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/specify/ess-compiler"
            },
            {
              "confidence": "inferred",
              "path": "crates/specify/ess-domain"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-typed-diagnostics.md"
            }
          ]
        }
      ]
    },
    {
      "wave": 2,
      "artifacts": [
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
          "id": "story:review-cache-origin",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli"
            }
          ]
        },
        {
          "id": "story:review-composition-contract",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/specify/ess-composition"
            },
            {
              "confidence": "inferred",
              "path": "website/docs"
            }
          ]
        },
        {
          "id": "story:review-consumer-coverage",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "Cargo.lock"
            },
            {
              "confidence": "cited",
              "path": "Taskfile.yml"
            },
            {
              "confidence": "cited",
              "path": "crates/edge/ess-xtask"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-consumer-coverage.md"
            }
          ]
        }
      ]
    },
    {
      "wave": 3,
      "artifacts": [
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
          "id": "story:review-format-catalog",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "docs/design/review-format-catalog.md"
            },
            {
              "confidence": "inferred",
              "path": "website/docs"
            },
            {
              "confidence": "cited",
              "path": "website/docs/guides/track-change.md"
            },
            {
              "confidence": "inferred",
              "path": "website/docs/reference/formats.md"
            },
            {
              "confidence": "cited",
              "path": "website/sidebars.ts"
            }
          ]
        }
      ]
    },
    {
      "wave": 4,
      "artifacts": [
        {
          "id": "story:review-browser-replay-fidelity",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-replay-subset.md"
            }
          ]
        },
        {
          "id": "story:review-delivery-trust-contract",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": ".github/actions/release-component"
            },
            {
              "confidence": "inferred",
              "path": "crates/edge/ess-cli"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-deployment"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-delivery-trust.md"
            },
            {
              "confidence": "inferred",
              "path": "website/docs"
            }
          ]
        }
      ]
    },
    {
      "wave": 5,
      "artifacts": [
        {
          "id": "story:review-expression-typechecking",
          "inferred": true,
          "scope": [
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
              "path": "docs/design/review-expression-typechecking.md"
            }
          ]
        },
        {
          "id": "story:review-glossary-boundaries",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "docs/design/review-concept-boundaries.md"
            },
            {
              "confidence": "inferred",
              "path": "website/docs"
            }
          ]
        },
        {
          "id": "story:review-observation-completeness",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli"
            },
            {
              "confidence": "cited",
              "path": "crates/infra/ess-kubernetes"
            },
            {
              "confidence": "cited",
              "path": "crates/infra/infra-analyze"
            },
            {
              "confidence": "cited",
              "path": "crates/infra/infra-compiler"
            },
            {
              "confidence": "cited",
              "path": "crates/infra/infra-domain"
            },
            {
              "confidence": "cited",
              "path": "crates/infra/infra-project"
            },
            {
              "confidence": "cited",
              "path": "crates/infra/infra-spec"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-observation-completeness.md"
            },
            {
              "confidence": "inferred",
              "path": "examples/k3d-dev-cluster"
            }
          ]
        }
      ]
    },
    {
      "wave": 6,
      "artifacts": [
        {
          "id": "story:review-openapi-semantic-accounting",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-openapi"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-openapi-accounting.md"
            }
          ]
        },
        {
          "id": "story:review-primitive-semantics",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/generate/ess-gen"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-synth"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-primitives"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-primitive-semantics.md"
            }
          ]
        },
        {
          "id": "story:review-public-support-claims",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "Taskfile.yml"
            },
            {
              "confidence": "inferred",
              "path": "crates/edge/ess-xtask"
            },
            {
              "confidence": "cited",
              "path": "website/docs"
            }
          ]
        }
      ]
    },
    {
      "wave": 7,
      "artifacts": [
        {
          "id": "story:review-output-ownership",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-output-ownership.md"
            }
          ]
        },
        {
          "id": "story:review-schema-resource-identity",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "docs/design/review-schema-resource-identity.md"
            },
            {
              "confidence": "inferred",
              "path": "website/docs"
            }
          ]
        },
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
        }
      ]
    },
    {
      "wave": 8,
      "artifacts": [
        {
          "id": "story:review-rust-target-feasibility",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli"
            },
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli/src/main.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-synth"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-rust-target-feasibility.md"
            }
          ]
        }
      ]
    },
    {
      "wave": 9,
      "artifacts": [
        {
          "id": "story:scenarios-directory-compiles-nothing",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli"
            }
          ]
        }
      ]
    },
    {
      "wave": 10,
      "artifacts": [
        {
          "id": "story:review-authored-discovery",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-authored-discovery.md"
            }
          ]
        }
      ]
    },
    {
      "wave": 11,
      "artifacts": [
        {
          "id": "story:review-conformance-coverage",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-conformance-coverage.md"
            }
          ]
        }
      ]
    }
  ],
  "collisions": [
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:create-only-command-cannot-refuse",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:java-conformance-target",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:java-conformance-target",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-authored-discovery",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-browser-replay-fidelity",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-cache-origin",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-composition-contract",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-conformance-coverage",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-conformance-coverage",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-consumer-coverage",
      "path": "Cargo.lock",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-delivery-trust-contract",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-delivery-trust-contract",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-expression-typechecking",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-format-catalog",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-glossary-boundaries",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-observation-completeness",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-openapi-semantic-accounting",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-output-ownership",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-primitive-semantics",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-public-support-claims",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-rust-target-feasibility",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-schema-resource-identity",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:scenarios-directory-compiles-nothing",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "crates/verify/ess-conformance",
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
      "b": "story:review-browser-replay-fidelity",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-conformance-coverage",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-expression-typechecking",
      "path": "crates/specify/ess-compiler",
      "confidence": "cited"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-expression-typechecking",
      "path": "crates/specify/ess-domain",
      "confidence": "cited"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-expression-typechecking",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-primitive-semantics",
      "path": "crates/generate/ess-gen",
      "confidence": "inferred"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-primitive-semantics",
      "path": "crates/generate/ess-synth",
      "confidence": "inferred"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-primitive-semantics",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-rust-target-feasibility",
      "path": "crates/generate/ess-synth",
      "confidence": "inferred"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-typed-diagnostics",
      "path": "crates/specify/ess-compiler",
      "confidence": "cited"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-typed-diagnostics",
      "path": "crates/specify/ess-domain",
      "confidence": "inferred"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-authored-discovery",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-browser-replay-fidelity",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-cache-origin",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-conformance-coverage",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-conformance-coverage",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-delivery-trust-contract",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-expression-typechecking",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-observation-completeness",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-openapi-semantic-accounting",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-output-ownership",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-primitive-semantics",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-rust-target-feasibility",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:scenarios-directory-compiles-nothing",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-authored-discovery",
      "b": "story:review-cache-origin",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-authored-discovery",
      "b": "story:review-conformance-coverage",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-authored-discovery",
      "b": "story:review-delivery-trust-contract",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-authored-discovery",
      "b": "story:review-observation-completeness",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-authored-discovery",
      "b": "story:review-openapi-semantic-accounting",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-authored-discovery",
      "b": "story:review-output-ownership",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-authored-discovery",
      "b": "story:review-rust-target-feasibility",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-authored-discovery",
      "b": "story:scenarios-directory-compiles-nothing",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-browser-replay-fidelity",
      "b": "story:review-conformance-coverage",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-browser-replay-fidelity",
      "b": "story:review-expression-typechecking",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-browser-replay-fidelity",
      "b": "story:review-primitive-semantics",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-browser-replay-fidelity",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-cache-origin",
      "b": "story:review-conformance-coverage",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-cache-origin",
      "b": "story:review-delivery-trust-contract",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-cache-origin",
      "b": "story:review-observation-completeness",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-cache-origin",
      "b": "story:review-openapi-semantic-accounting",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-cache-origin",
      "b": "story:review-output-ownership",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-cache-origin",
      "b": "story:review-rust-target-feasibility",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-cache-origin",
      "b": "story:scenarios-directory-compiles-nothing",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-composition-contract",
      "b": "story:review-delivery-trust-contract",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-composition-contract",
      "b": "story:review-format-catalog",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-composition-contract",
      "b": "story:review-glossary-boundaries",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-composition-contract",
      "b": "story:review-public-support-claims",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-composition-contract",
      "b": "story:review-schema-resource-identity",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-conformance-coverage",
      "b": "story:review-delivery-trust-contract",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-conformance-coverage",
      "b": "story:review-expression-typechecking",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-conformance-coverage",
      "b": "story:review-observation-completeness",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-conformance-coverage",
      "b": "story:review-openapi-semantic-accounting",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-conformance-coverage",
      "b": "story:review-output-ownership",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-conformance-coverage",
      "b": "story:review-primitive-semantics",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-conformance-coverage",
      "b": "story:review-rust-target-feasibility",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-conformance-coverage",
      "b": "story:scenarios-directory-compiles-nothing",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-conformance-coverage",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-consumer-coverage",
      "b": "story:review-public-support-claims",
      "path": "Taskfile.yml",
      "confidence": "inferred"
    },
    {
      "a": "story:review-consumer-coverage",
      "b": "story:review-public-support-claims",
      "path": "crates/edge/ess-xtask",
      "confidence": "inferred"
    },
    {
      "a": "story:review-consumer-coverage",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "Taskfile.yml",
      "confidence": "cited"
    },
    {
      "a": "story:review-consumer-coverage",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "crates/edge/ess-xtask",
      "confidence": "inferred"
    },
    {
      "a": "story:review-delivery-trust-contract",
      "b": "story:review-format-catalog",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-delivery-trust-contract",
      "b": "story:review-glossary-boundaries",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-delivery-trust-contract",
      "b": "story:review-observation-completeness",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-delivery-trust-contract",
      "b": "story:review-openapi-semantic-accounting",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-delivery-trust-contract",
      "b": "story:review-output-ownership",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-delivery-trust-contract",
      "b": "story:review-public-support-claims",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-delivery-trust-contract",
      "b": "story:review-rust-target-feasibility",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-delivery-trust-contract",
      "b": "story:review-schema-resource-identity",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-delivery-trust-contract",
      "b": "story:scenarios-directory-compiles-nothing",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-expression-typechecking",
      "b": "story:review-primitive-semantics",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-expression-typechecking",
      "b": "story:review-typed-diagnostics",
      "path": "crates/specify/ess-compiler",
      "confidence": "cited"
    },
    {
      "a": "story:review-expression-typechecking",
      "b": "story:review-typed-diagnostics",
      "path": "crates/specify/ess-domain",
      "confidence": "inferred"
    },
    {
      "a": "story:review-expression-typechecking",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-format-catalog",
      "b": "story:review-glossary-boundaries",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-format-catalog",
      "b": "story:review-public-support-claims",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-format-catalog",
      "b": "story:review-schema-resource-identity",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-glossary-boundaries",
      "b": "story:review-public-support-claims",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-glossary-boundaries",
      "b": "story:review-schema-resource-identity",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-observation-completeness",
      "b": "story:review-openapi-semantic-accounting",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-observation-completeness",
      "b": "story:review-output-ownership",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-observation-completeness",
      "b": "story:review-rust-target-feasibility",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-observation-completeness",
      "b": "story:scenarios-directory-compiles-nothing",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-openapi-semantic-accounting",
      "b": "story:review-output-ownership",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-openapi-semantic-accounting",
      "b": "story:review-rust-target-feasibility",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-openapi-semantic-accounting",
      "b": "story:scenarios-directory-compiles-nothing",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-output-ownership",
      "b": "story:review-rust-target-feasibility",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-output-ownership",
      "b": "story:scenarios-directory-compiles-nothing",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-primitive-semantics",
      "b": "story:review-rust-target-feasibility",
      "path": "crates/generate/ess-synth",
      "confidence": "cited"
    },
    {
      "a": "story:review-primitive-semantics",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-public-support-claims",
      "b": "story:review-schema-resource-identity",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-public-support-claims",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "Taskfile.yml",
      "confidence": "inferred"
    },
    {
      "a": "story:review-public-support-claims",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "crates/edge/ess-xtask",
      "confidence": "inferred"
    },
    {
      "a": "story:review-rust-target-feasibility",
      "b": "story:scenarios-directory-compiles-nothing",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    }
  ],
  "unassessed": [],
  "cycles": []
}
```

## Selected proposed computation

```json
{
  "waves": [
    {
      "wave": 1,
      "artifacts": [
        {
          "id": "story:review-format-catalog",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "docs/design/review-format-catalog.md"
            },
            {
              "confidence": "inferred",
              "path": "website/docs"
            },
            {
              "confidence": "cited",
              "path": "website/docs/guides/track-change.md"
            },
            {
              "confidence": "inferred",
              "path": "website/docs/reference/formats.md"
            },
            {
              "confidence": "cited",
              "path": "website/sidebars.ts"
            }
          ]
        },
        {
          "id": "story:review-rust-target-feasibility",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli"
            },
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli/src/main.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-synth"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-rust-target-feasibility.md"
            }
          ]
        }
      ]
    }
  ],
  "collisions": [],
  "unassessed": [],
  "cycles": []
}
```

## Opening and dispatch

Opening4b66aac7b608b1deee9de88942390d4a6c5ec745 has exact bot author/committer. Post-commit fmt, release, action, planning and diff checks all exited0; raw record is target/review-boundaries-5/opening-checks. Both managed unit trees were created from that exact opening, with leases ess-review-wave5-rust and ess-review-wave5-format. Full paths are the ESS managed tree root plus the table suffix; each build/scratch is nested inside that tree. Format brief is its assigned scratch/unit-brief.md; Rust binding proposal and consumer inventory were completed before code; the measured zero-capability failure required the corrected binding below.

## Downstream reader coordination

Fresh source inventory confirmed SDK advertised main48833c6d14ec37cb3b614fca05cf7dd78f63b743 and AgentIDE main176a57f58457a7c16f105584c66964263b3c2e41. Both primaries are clean but stale; exact Git objects, then managed SDK source, supply the evidence. SDK build_ess drops target; its existing ESS pin already exposes the report so a guard can land readers-first without a dependency upgrade. AgentIDE already propagates the builder Result before write/check, but its exact older SDK pin is not moved by updating SDK main. No direct external rust::workspace caller is established by the bounded inventory; unknown external callers remain unverified.

Two coordinator-owned support trees were created and recorded before implementation. Atlas ess-rust-refusal-governance at a58a980 on docs/ess-rust-refusal-governance has lease ess-review-wave5-governance and story:ess-rust-target-refusal-migration (58-artifact store, valid with existing advisories). SDK ess-rust-refusal-reader at48833c6 on fix/ess-rust-refusal-reader has lease ess-review-wave5-sdk and story:reject-ess-rust-target-refusals (28-artifact store, valid). Their full roots are the manager's b10x/atlas and b10x/service-sdk roots plus these suffixes; each owns target and target/review-boundaries-5 scratch. No shared build directory or SDK pin upgrade is used.

The support implementation increases the potential worker count to3 within the same4-agent budget/3-worker limit, with Atlas orchestration remaining at root. Resource reserve and cleanup rules apply to support trees as to ESS units. SDK source guard needs independent review, complete SDK task check and exact frozen ESS refusal/valid controls; unit-only synthetic reports are insufficient end-to-end evidence. Atlas ADR0037 records contract choices and move order before source publication. Public catalog delivery will use an additional managed Website tree only when its published ESS input exists.


## Checked Rust failure decision and reader stage

The source-only consumer inventory is target/review-boundaries-5/pre-scope-rust/consumer-inventory.md (SHA2565592979d6a49a55b459e0fff06bbdac7899fba053332457f467e2cb40a968f83). It inspected36 sibling HEADs, refreshed SDK/AgentIDE advertised refs, and established the actual SDK omission plus AgentIDE propagation order. It is a bounded inventory, not a claim about every external caller.

The implementor's untouched-production regression an_empty_domain_cannot_overwrite_the_rust_crate_root compiled demo.lib with zero capabilities, then panicked on duplicate crates/demo-types/src/lib.rs (exit101, one executed/failed test). Raw zero-capabilities-red artifacts and the finding are in the Rust unit's scratch. The plan has no capability that could honestly carry this global refusal. The coordinator therefore replaced the initial metadata-only preference before the affected production edits: synthesize, synthesize_for, rust::workspace and web::workspace return checked Result with TargetFailure. Successful fields, canonical plan bytes, valid artifacts and existing reports remain unchanged. Fatal Rust/Web failure returns no synthesis or artifacts; CLI returns1 and admits failure before destination writes.

The versioned Serialize-only ess-target-failure/1 envelope has target, unchanged plan and nonempty deterministic typed global causes. Its eight initial codes and actual allocation/type-layout/binding/codec checks are bound in docs/design/review-rust-target-feasibility.md. No empty TargetReport exception, fake capability or neutral semantic rewrite is used. Root CHANGELOG.md was added as inferred typed scope; the complete revised active computation follows below and still has no collision.

Atlas ADR0037 was committed and published as3666a5091c0a13f3a8c1c5ad82a1ac5642acac56. Its source/docs/planning checks passed (140 Markdown files, zero findings). Full fences exited1 at2026-09-05T19:39:18.851795Z after198.075431676s with the same three independently observed sibling failures: primary AgentIDE's unsupported documentation manifest, primary Website's outdated Docs System pin, and Widgets' missing Serves. All other fence lanes were green. This is not an organization-wide green claim. Raw private evidence remains in the Atlas support target/review-boundaries-5/decision-fence; the clean authority was advanced to this exact advertised main.

SDK implementationc6bd6e7e88f76196a228a76e9ad5fdbb3f937d7d keeps a private conversion for both direct Synthesis and checked Result, preserving original typed errors, then rejects any wrong target/refusal/weakening before projection or output construction. Its actual current-pin generation reproduced all63 files byte-for-byte. Independent pass1 added six cases (24 to30, zero red) and found nothing; immutable review-result:ess-rust-refusal-reader-pass-1 is recorded in the SDK store with exact report SHA256e6091f358865eb7c8d8e6eff6359e8d6176951fe5414b5766f5e2876e03b1333. The tests and Unreleased note are frozene9fcbb960b7a5283666c23c4e0478d4e7b7406d7; the full seven-lane SDK gate is running on that source. Actual corrected-producer CLI refusal and valid-byte checks remain pending a frozen ESS subject; none of these current-pin/synthetic cases substitutes for them. Existing SDK/AgentIDE pins are unchanged.

## Revised active computation

```json
{
  "waves": [
    {
      "wave": 1,
      "artifacts": [
        {
          "id": "story:review-format-catalog",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "docs/design/review-format-catalog.md"
            },
            {
              "confidence": "inferred",
              "path": "website/docs"
            },
            {
              "confidence": "cited",
              "path": "website/docs/guides/track-change.md"
            },
            {
              "confidence": "inferred",
              "path": "website/docs/reference/formats.md"
            },
            {
              "confidence": "cited",
              "path": "website/sidebars.ts"
            }
          ]
        },
        {
          "id": "story:review-rust-target-feasibility",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "CHANGELOG.md"
            },
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli"
            },
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli/src/main.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-synth"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-rust-target-feasibility.md"
            }
          ]
        }
      ]
    }
  ],
  "collisions": [],
  "unassessed": [],
  "cycles": []
}
```

## Public catalog delivery preparation

Source-backed runbook target/review-boundaries-5/pre-scope-delivery/runbook.md established the current bundle source-set publication lane. It resolves every catalog source, derives a deterministic effective source lock, renders the Atlas snapshot with exact pinned Docs System/Website tooling, and verifies a self-contained v2 publication before root-only reconciliation. Neither the retained-lock updater nor the bundle resolver offers an ESS-only flag; any other source movement must be inspected explicitly. The pinned Website runtime and advertised Website main differ. Node24 and exact runtime gates, published source bundle identity, all changed inputs, current publication hold state and live provenance must be established before claiming delivery. No Website mutation, source pin promotion, legacy facade redeployment or delivery success has occurred at this checkpoint.


## Delivery runtime preparation and review checkpoint

Atlas now records story:publish-ess-format-catalog, active revision4, with cited daily-log scope and existing source-set contracts. Root provisioned unchanged published runtime trees before dependency work: /home/timo/.local/state/worktree/trees/b10x/website/ess-format-catalog-delivery at815fad1b977992d01695f6b5c79495c02576212b (lease ess-review-wave5-website), and /home/timo/.local/state/worktree/trees/b10x/docs-system/ess-format-catalog-tooling at1c8c31697e87235dda8bec9467264b22a7fa0c95 (lease ess-review-wave5-docs-tooling). Each owns target, target/review-boundaries-5 and TMPDIR. These are executable pinned tooling, not source collection worktrees. Node24.20.0 dependency installation passed in both; exact Docs System bundle validator cargo build --locked --offline passed. Actual paths and per-command exits are retained in each preparation.json. No source-set resolution, snapshot or publication gate ran before the ESS source commit exists.

Format adversary pass1 is immutable review-result:review-boundaries-5-format-adversary-pass-1, exact report SHA2560251b7d9e850126bbe137202c2517a698ed36b70b25cc80c1693aa4179cdc14e. Its two introduced documentation findings were corrected in d8dedad: compact sorted typed InfraSpec identity and persisted projection input digests, and the actual EssImpact result type. Both fixed outcomes were recorded when the correction landed. The correction has190 link/17 sidebar checks and actual fixture-byte evidence, not invented prose tests. The final Rust failure rows and second independent catalog attack remain pending frozen Rust source.

The SDK seven-lane gate on e9fcbb960b7a5283666c23c4e0478d4e7b7406d7 finished2026-09-05T19:50:19.449504Z, all exits0,111 Rust and4 web cases passed with0 failed/ignored. Actual corrected-producer proof is still pending; the guard has not yet been published or its story marked implemented.


## Exact producer comparison resource

Root reserves one detached managed ESS source fixture, id ess-rust-producer-compatibility and lease ess-review-wave5-producer, initially at published base6616b26fe41548af9cb7ff9cf833ae977883f625. After an implementation is committed, root may advance this clean fixture to that exact frozen commit for the SDK comparison while the original Rust unit receives test-only review changes. The fixture owns target and target/review-boundaries-5 scratch; no shared target is used. SDK baseline and candidate compilation live in separate assigned SDK target/review-boundaries-5/exact-producer-baseline/build and exact-producer/build directories. SDK Cargo.lock is backed up byte-for-byte, temporarily resolved with all seven exact ESS path overrides, and restored in a finally block; no manifest or published pin changes. Full SDK gates already passed and are not repeated. Comparing the current published ESS producer to the corrected producer distinguishes this wave's valid-output behavior from prior source/digest migrations. Root records every actual subject, resolved graph, output map, command/exit and restoration proof. This adds no implementing agent or scope collision.


## Frozen implementation and published SDK reader

Rust implementationf9a7cf7fcca79448a34b2754adb12f1a411573bd was committed after root verified all19 handoff source hashes and both bot identities. Implementor report SHA25627dced0e7e530441e525de7b1169e27b42b99b1b0b6a717065e63df31835d4b3 carries178 package cases (CLI53+synth125) against146 baseline,22 distinct retained behavioral reds, package fmt/strict Clippy, actual fresh compiler/replay checks and complete valid billing Rust/Web plus gatepass Rust map equality. A separate thread now performs the first full tests-only adversarial pass; the implementation is not merged or declared complete.

SDK source6e5141f3ead0e4d0c8f75787aec7051e2a1f41d0 was published to main through the bot wrapper and its clean primary advanced non-forcibly. SDK story:reject-ess-rust-target-refusals is implemented revision11 on its full gate, immutable admission review and actual frozen producer compatibility. ESS and AgentIDE dependency pins are unchanged. The exact-source consumer fixture is /home/timo/.local/state/worktree/trees/b10x/ess/ess-rust-producer-compatibility, first at published6616b26 then at frozenf9a7cf7; each source was clean during its own comparison. Both real SDK CLI builds resolved all7 ESS packages to that exact0.18.0 path graph, with no non-ESS resolution movement. The corrected producer returned the actual collision or recursive-layout error for eight generate/check attempts across absent/owned-sentinel outputs. Every rejected destination inventory remained identical. Six valid controls distinguished ordinary drift, exercised successful owned-sentinel replacement and verified all63 valid files exactly equal to the published6616 producer. The older SDK-pinned d1a6677 tree still has the previously observed14/49 changed/equal split; candidate checking records that drift without writes. Original lock/manifest/source/planning state was restored; locked offline metadata again resolved all7 historical packages at d1a6677/0.13.1.

SDK's detailed result is committed in its story at6e5141f, with complete raw configs/graphs/locks/maps/argv/exits under its target/review-boundaries-5/exact-producer* and a standalone frozen-producer-compatibility.md report. First setup attempts retained an unused patch selection and an overly short expected recursive source path; the actual graph check stopped before compiling stale dependencies, and the exact field-path diagnostic was corrected without production changes. Those setup failures are not hidden or counted as target regressions. The final14-command matrix completed2026-09-05T20:07:39.556686Z. A full clean Atlas fence is running after SDK publication; no new organization-wide green claim is made.


## Concurrent ESS work: reviewed landing boundary

On 2026-09-05 the review coordinator fetched ESS origin and confirmed remote main remains `6616b26fe41548af9cb7ff9cf833ae977883f625`. Clean Atlas authority `3666a5091c0a13f3a8c1c5ad82a1ac5642acac56` also equals its advertised main. This is the review-remediation session; the separate source-driven work remains owned by its existing session.

Read-only inventory found source-driven imports, types-only realizations, normalization, authored-site work and unique wire-field validation in managed `wt-46ef382d9f07`, based on `6cbe372`. Its separate `wt-6525314b6daa` preview is based on our Rust candidate `f9a7cf7`, with both feature source and older catalog changes overlaid. The source session already records `story:integrate-source-driven-realizations` revision 5 with explicit dependencies on this wave's Rust and catalog stories. Its proposed order is wave 5 first, source-driven integration afterward. The review coordinator adopts that order. No other session's source, branch, journal, lifecycle or evidence was modified by this audit.

The independent source preview records a combined `task check` and `task site-build` exit 0 after reconciling duplicate wire-field admission; those are attributed reports, not a fresh gate executed or certified by this coordinator. Its actual adopter regeneration reports 73 files with 51 accounted provenance/digest differences, so this is not a clean adopter drift result or final producer adoption.

Our fresh six three-way file checks retain exact input/output hashes under `target/review-boundaries-5/concurrent-merge-audit/three-way-records.json`. CLI wiring against both published main and Rust candidate exits 0. Public catalog, current Rust binding correction and current Rust test correction also merge without text conflicts. The latest internal catalog versus the older preview has one conflict, confined to the final checked-result compatibility paragraph. A scratch-only resolved proposal retains the newer reviewed paragraph and the separate feature format inventory. Neither scratch proposal nor the live uncommitted preview is a frozen combined publication candidate.

The semantic conflict is explicit: the new domain validation rejects duplicate effective JSON object keys before any target runs because schema projection would otherwise overwrite declarations. This supersedes the current Rust test's source-admission assumption for pure Rust when the feature lands. The review coordinator accepts that source invariant at the combined integration boundary; it is not a Rust target blanket blacklist or a reason to remove target-only codec collision, distinct-wire, name-allocation or representation regressions. Current wave 5 does not introduce domain source validation, so its existing target test remains correct on this wave's own source. Integration must replace that source-admission expectation with precise source-location rejection and retain the independent target checks.

Landing obligations: complete and publish wave 5 first; refresh the feature integration from its exact published main; reconcile CLI/changelog, the five new format markers and distinct source/bundle/model/recipe/file digest contracts; reconcile AEP through its CLI with one journal writer; rerun full ESS, site and actual affected consumers on the exact combined source. Recheck remote ancestry immediately before each publication and do not force-push. Do not schedule new review implementations in the feature-owned compiler/domain/schema-contract/ess-gen/CLI/public-doc surfaces until the integration boundary is reassessed. Count/coverage preparation remains read-only and needs fresh scoping before selection. All other sessions' managed trees remain outside our cleanup set.

The other session's plan says cross-root messaging is unavailable. This document records this coordinator's acknowledgement and decisions in its own repository; it is not a claim that a message was delivered to another root session.


## Review checkpoint and format integration

Format final subject `f2c81b8ed07a6b522bb97b1e6709a9f2d0a3be57` passed its second and final full adversarial review by scope_conformance_design. Immutable `review-result:review-boundaries-5-format-adversary-pass-2` contains the complete 14,228-byte report with SHA256 `b3a63197aa40e349ef9a140e36e8130bafc04fd0a7f8d3a1b9eaf4cb8780cf7c`. Both original findings are resolved; no carried or new findings. The exact formal findings-ledger output follows. The unit owns four documentation/navigation files and its final link inventory checks 209 links and 17 sidebar entries, with exact frozen Git-object fallback for new Rust source not present in the documentation-only tree. It is ready for source integration; story completion still requires the combined gate and public delivery.

```json
{
  "artifact": "story:review-format-catalog",
  "reviews": 10,
  "from": "review-result:review-boundaries-5-format-adversary-pass-1",
  "from_reviewer": "impl_diagnostic",
  "to": "review-result:review-boundaries-5-format-adversary-pass-2",
  "to_reviewer": "scope_conformance_design",
  "carried": [],
  "new": [],
  "resolved": [
    {
      "file": "docs/design/review-format-catalog.md",
      "line": 115,
      "category": "contract-drift",
      "severity": "blocker",
      "verdict": "NEEDS-CHANGE",
      "origin": "introduced",
      "message": "The intent row denies an existing InfraSpec::digest API and the catalogs omit the separate compact typed-intent digest persisted as projection provenance.specification_digest."
    },
    {
      "file": "docs/design/review-format-catalog.md",
      "line": 82,
      "category": "contract-drift",
      "severity": "warning",
      "verdict": "CONFIRMED",
      "origin": "introduced",
      "message": "The impact row names ImpactReport, but the public producer returns and exports EssImpact."
    }
  ]
}
```

Rust pass 1 is immutable `review-result:review-boundaries-5-rust-adversary-pass-1`, complete 597,874-byte report SHA256 `344184edfa8144f2f8331dd690c173810babce29e0ca9f5225a90415b1915723`. It measured 185 cases, 181 passes and four new failures, tracing both underlying Web dependency-module and HTTP outcome-binding collisions to exact opening source `4b66aac`. The active Rust story remains their owner; correction and second review are pending. The correction implementor additionally measured an empty-event Web match compiler failure while constructing a types-only fixture; this must be separately classified before changing the bounded correction. No new correction production edit or green claim exists at this checkpoint.

The SDK post-publication Atlas fence completed 2026-09-05T20:14:55.493803Z on clean authority `3666a50`: exit 1, 115 Rust cases passed, 51 live route observations, and the same three recorded baseline failures. Full raw evidence remains private under the Atlas support tree's `target/review-boundaries-5/sdk-publication-fence`. This is not an organization-wide green result.
