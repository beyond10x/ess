# ESS review boundaries — wave 8

**Skill version 0.8.0 — aep-drive:wave.** This is the next replan under approval-record:review-remediation-standing-implementation and approval-record:review-remediation-standing-publication. The operator already authorized all remediation waves, green source publication and task-owned cleanup. No release/tag/version bump or default switch is part of this wave.

## Selection

Select only story:a-skipped-scenario-is-not-a-failed-one, serving vision:O2. Both dependency stories are implemented. The separate AEP reader prerequisite is published at30aeef2 with both independent readers, typed admission/replay and passing full source gate/CI. The accepted ESS binding and Atlas ADR0039 require actual producer compatibility before the opt-in writer publishes. Reader installation and adopter regeneration remain separate.

The independent story-scoper refreshed the exact source boundaries and revealed two additional documentation status rows: public formats and the internal format catalog. Root wrote the condensed scope and every typed confidence marker through AEP. The actual wave computation below has no unassessed stories or cycles. The count writer overlaps the other eligible implementation packages/public-doc tokens; the consumer-matrix candidate also collides at the shared lock and still needs a binding mechanism decision. Fuzzing is a separate broad target/toolchain scope, deferred while this P0 migration is bounded. Other-session normalization and unrelated legacy feature drafts are not selected. A wave of one avoids inventing source separation or taking their ownership.

## Resource preflight and ownership

The clean primary was on main1667d022 and the manager-owned coordinator was clean at the same commit before this proposal. Prior expression/OpenAPI/AEP implementation records are removed; their build directories are gone and evidence is hash-verified in the user cache. Other sessions' linked trees are listed below and untouched. The existing coordinator is deliberately reused across the already authorized remediation loop; its own measured target is retained for that checkout only. It is not shared with the new unit.

Budget handling remains the established default four concurrent agents with no model override; this wave uses one implementor and one subsequent independent adversary, within the three available worker slots. Free disk was33,688,248,320 bytes against8,589,934,592 floor. The retained coordinator target measured5,668,868,595 bytes; earlier actual single-unit build/cleanup measurements bound this one unit. The old sccache socket is dead, so direct rustc is the established resumed setting: no replacement/shared server is started. Each checkout owns its own target, four Cargo jobs, debug/incremental off. Scratch and Go caches are inside the unit target; no process is allowed to borrow another target.

Coordinator: managed wt-752828a285ba, branch wave/review-boundaries-8. Reserve unit ID ess-conformance-count-writer, branch impl/ess-conformance-count-writer; actual manager path, opening commit and triple are recorded before dispatch. Root owns planning, commits, the shared lock/changelog, AEP compatibility orchestration, Atlas delivery and cleanup. The unit owns only cited ESS package and guide/catalog surfaces. Native plugin agent types are unavailable; collaboration agents read aep-drive:implementor and aep-drive:adversary0.8.0 charters as the existing harness deviation.

Standing approval authorizes this opening commit, the unit/correction/reviewer-test commits, integration merges, evidence/closing commits, green main publication and exact-id cleanup. It does not authorize a release or manufacture any gate result.

## Contract and validation

The accepted docs/design/review-conformance-coverage.md remains binding. Implement explicit report2 and detailed run2 for original legacy suites1–4 with coverage exactly unknown; preserve suite4/report1/diagnostic defaults and all frozen legacy bytes. Unknown conformance cannot qualify even with all scenarios passing. Admission/configuration must precede target identity/callbacks/writes. Exact integers, original suite hashing, producer category semantics, complete selected-ID membership, Go uninvoked subtests, strict exits and separate detailed/standalone shapes are required.

The implementor establishes package baseline counts, writes/runs behavioral red cases first, then passes the complete assigned package suites, formatting and strict Clippy. Actual generated Go execution is mandatory; compile/API setup errors are retained separately from semantic red evidence. Root freezes green producers and executes original Rust/Go report/suite pairs through both published AEP readers and typed replay before writer publication. The independent adversary has at most two complete attacks, immutable reports and separate outcomes. Root runs all actual Taskfile gate steps plus site-build and planning, retaining each command exit and executed-case count. No suite5, binary installation, release or default transition is inferred.

## Initial measured preflight

```json
{
  "free_disk_bytes": 33688248320,
  "disk_floor_bytes": 8589934592,
  "git_head": "1667d022ed2041342c8928250ab8bddfe9c988b9",
  "branch": "wave/review-boundaries-7",
  "status": "",
  "worktrees": "worktree /home/timo/beyond10x/ess\nHEAD 1667d022ed2041342c8928250ab8bddfe9c988b9\nbranch refs/heads/main\n\nworktree /home/timo/.local/state/worktree/trees/b10x/ess/wt-1f0716ada0f4\nHEAD 6ef4af76b99a8d2cd861a3cc76140c88c1361129\nbranch refs/heads/feat/build-graph-projection\n\nworktree /home/timo/.local/state/worktree/trees/b10x/ess/wt-3726210974f1\nHEAD 4ee2653d2c5e1b5a754d7b3b0ed5d35c792882b3\nbranch refs/heads/feat/accepting-wrong-state-and-view-params\n\nworktree /home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba\nHEAD 1667d022ed2041342c8928250ab8bddfe9c988b9\nbranch refs/heads/wave/review-boundaries-7\n\nworktree /home/timo/.local/state/worktree/trees/b10x/ess/wt-8850a8418f1f\nHEAD e6caac9492b8d5ada56dfabed23639a583ad8ac8\nbranch refs/heads/plan/java-conformance-target\n\nworktree /home/timo/.local/state/worktree/trees/b10x/ess/wt-bf45625a6a50\nHEAD 60ffcb2238ffef3a48d0db9555b6f2ca709ca2f7\nbranch refs/heads/impl/normalization-base64-resume\n\nworktree /home/timo/.local/state/worktree/trees/b10x/ess/wt-c12a5474a249\nHEAD 60ffcb2238ffef3a48d0db9555b6f2ca709ca2f7\nbranch refs/heads/impl/normalization-raw-json\n\n",
  "build_usage": "5668868595\ttarget\n",
  "primary_head": "1667d022ed2041342c8928250ab8bddfe9c988b9",
  "primary_status": "",
  "primary_branch": "main"
}
```

## Complete refreshed wave computation

Command: aep plan artifact waves --kind story --status draft --format json, exit0. The full waves, collisions, unassessed and cycles follow verbatim.

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
              "confidence": "cited",
              "path": "docs/design/review-format-catalog.md"
            },
            {
              "confidence": "inferred",
              "path": "website/docs"
            },
            {
              "confidence": "cited",
              "path": "website/docs/guides/verify-conformance.md"
            },
            {
              "confidence": "cited",
              "path": "website/docs/reference/formats.md"
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
          "id": "story:model-binary64-fields",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/generate/ess-gen/src/types.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/schema-contract/src/realize/normalize/check.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/schema-contract/src/realize/normalize/recipe.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-domain/src/types.rs"
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
          "id": "story:release-status-publication-state",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/edge/ess-xtask/src/main.rs"
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
          "id": "story:normalize-positional-array-input",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/generate/schema-contract/src/realize/normalize/check.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/schema-contract/src/realize/normalize/recipe.rs"
            }
          ]
        },
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
          "id": "story:raw-json-normalization-provenance",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "CHANGELOG.md"
            },
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli/tests/normalization.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/schema-contract/src/realize/normalize.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/schema-contract/src/realize/normalize/check.rs"
            },
            {
              "confidence": "inferred",
              "path": "crates/generate/schema-contract/src/realize/normalize/eval.rs"
            },
            {
              "confidence": "inferred",
              "path": "crates/generate/schema-contract/src/realize/normalize/execute.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/schema-contract/src/realize/normalize/go_input.go.txt"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/schema-contract/src/realize/normalize/go_runtime.go.txt"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/schema-contract/src/realize/normalize/go_target.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/schema-contract/src/realize/normalize/input.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/schema-contract/src/realize/normalize/recipe.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/schema-contract/src/realize/normalize/rust_runtime.rs.txt"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/schema-contract/src/realize/normalize/target.rs"
            },
            {
              "confidence": "inferred",
              "path": "crates/generate/schema-contract/tests/fixtures/normalization_raw_json.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/schema-contract/tests/normalization_go.rs"
            },
            {
              "confidence": "inferred",
              "path": "crates/generate/schema-contract/tests/normalization_raw_json.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/schema-contract/tests/normalization_rust.rs"
            },
            {
              "confidence": "cited",
              "path": "docs/design/source-pinned-data-normalization.md"
            },
            {
              "confidence": "cited",
              "path": "website/docs/guides/generate-artifacts.md"
            },
            {
              "confidence": "cited",
              "path": "website/docs/reference/formats.md"
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
        },
        {
          "id": "story:typescript-normalization-target",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "CHANGELOG.md"
            },
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli/src/normalize.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/schema-contract/src/realize/normalize.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/schema-contract/src/realize/normalize/target.rs"
            },
            {
              "confidence": "cited",
              "path": "docs/design/source-pinned-data-normalization.md"
            },
            {
              "confidence": "cited",
              "path": "docs/design/typescript-normalization.md"
            },
            {
              "confidence": "cited",
              "path": "website/docs/guides/generate-artifacts.md"
            },
            {
              "confidence": "cited",
              "path": "website/docs/reference/cli.md"
            },
            {
              "confidence": "cited",
              "path": "website/docs/reference/formats.md"
            }
          ]
        }
      ]
    },
    {
      "wave": 5,
      "artifacts": [
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
        }
      ]
    },
    {
      "wave": 6,
      "artifacts": [
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
        }
      ]
    },
    {
      "wave": 8,
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
      "b": "story:integrate-source-driven-realizations",
      "path": "docs/design/review-format-catalog.md",
      "confidence": "cited"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:integrate-source-driven-realizations",
      "path": "website/docs/reference/formats.md",
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
      "b": "story:raw-json-normalization-provenance",
      "path": "website/docs/reference/formats.md",
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
      "b": "story:review-schema-resource-identity",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:typescript-normalization-target",
      "path": "website/docs/reference/formats.md",
      "confidence": "cited"
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
      "a": "story:enum-variant-in-an-entity-invariant",
      "b": "story:review-typed-diagnostics",
      "path": "crates/specify/ess-domain",
      "confidence": "inferred"
    },
    {
      "a": "story:integrate-source-driven-realizations",
      "b": "story:raw-json-normalization-provenance",
      "path": "CHANGELOG.md",
      "confidence": "inferred"
    },
    {
      "a": "story:integrate-source-driven-realizations",
      "b": "story:raw-json-normalization-provenance",
      "path": "website/docs/reference/formats.md",
      "confidence": "cited"
    },
    {
      "a": "story:integrate-source-driven-realizations",
      "b": "story:typescript-normalization-target",
      "path": "CHANGELOG.md",
      "confidence": "cited"
    },
    {
      "a": "story:integrate-source-driven-realizations",
      "b": "story:typescript-normalization-target",
      "path": "website/docs/reference/formats.md",
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
      "b": "story:review-observation-completeness",
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
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:model-binary64-fields",
      "b": "story:normalize-positional-array-input",
      "path": "crates/generate/schema-contract/src/realize/normalize/check.rs",
      "confidence": "cited"
    },
    {
      "a": "story:model-binary64-fields",
      "b": "story:normalize-positional-array-input",
      "path": "crates/generate/schema-contract/src/realize/normalize/recipe.rs",
      "confidence": "cited"
    },
    {
      "a": "story:model-binary64-fields",
      "b": "story:raw-json-normalization-provenance",
      "path": "crates/generate/schema-contract/src/realize/normalize/check.rs",
      "confidence": "cited"
    },
    {
      "a": "story:model-binary64-fields",
      "b": "story:raw-json-normalization-provenance",
      "path": "crates/generate/schema-contract/src/realize/normalize/recipe.rs",
      "confidence": "cited"
    },
    {
      "a": "story:native-realization-ci",
      "b": "story:review-consumer-coverage",
      "path": "Taskfile.yml",
      "confidence": "cited"
    },
    {
      "a": "story:native-realization-ci",
      "b": "story:review-public-support-claims",
      "path": "Taskfile.yml",
      "confidence": "inferred"
    },
    {
      "a": "story:native-realization-ci",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "Taskfile.yml",
      "confidence": "cited"
    },
    {
      "a": "story:normalize-positional-array-input",
      "b": "story:raw-json-normalization-provenance",
      "path": "crates/generate/schema-contract/src/realize/normalize/check.rs",
      "confidence": "cited"
    },
    {
      "a": "story:normalize-positional-array-input",
      "b": "story:raw-json-normalization-provenance",
      "path": "crates/generate/schema-contract/src/realize/normalize/recipe.rs",
      "confidence": "cited"
    },
    {
      "a": "story:raw-json-normalization-provenance",
      "b": "story:typescript-normalization-target",
      "path": "CHANGELOG.md",
      "confidence": "inferred"
    },
    {
      "a": "story:raw-json-normalization-provenance",
      "b": "story:typescript-normalization-target",
      "path": "crates/generate/schema-contract/src/realize/normalize.rs",
      "confidence": "cited"
    },
    {
      "a": "story:raw-json-normalization-provenance",
      "b": "story:typescript-normalization-target",
      "path": "crates/generate/schema-contract/src/realize/normalize/target.rs",
      "confidence": "cited"
    },
    {
      "a": "story:raw-json-normalization-provenance",
      "b": "story:typescript-normalization-target",
      "path": "docs/design/source-pinned-data-normalization.md",
      "confidence": "cited"
    },
    {
      "a": "story:raw-json-normalization-provenance",
      "b": "story:typescript-normalization-target",
      "path": "website/docs/guides/generate-artifacts.md",
      "confidence": "cited"
    },
    {
      "a": "story:raw-json-normalization-provenance",
      "b": "story:typescript-normalization-target",
      "path": "website/docs/reference/formats.md",
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
      "b": "story:review-output-ownership",
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
      "b": "story:review-output-ownership",
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
      "b": "story:review-observation-completeness",
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
      "b": "story:review-output-ownership",
      "path": "crates/edge/ess-cli",
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
    }
  ],
  "unassessed": [],
  "cycles": []
}
```

## Opening lifecycle

The first attempted draft→active move was refused by the actual lifecycle; it wrote no transition. Root then used the admitted draft→proposed→active sequence. The initial draft wave computation remains preserved above; no refusal was bypassed.

## Actual provisioned unit

Openingbd6d82f0d551fcf1cc2ec2eab65aab2fe7539947 has verified bot author/committer; fmt, action, planning and diff steps each exited0 before manager provisioning. Raw argv/log/exits are target/review-boundaries-8/opening/results.json. The manager-created unit is:

```json
{
  "id": "ess-conformance-count-writer",
  "worktree": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer",
  "branch": "impl/ess-conformance-count-writer",
  "base": "bd6d82f0d551fcf1cc2ec2eab65aab2fe7539947",
  "build": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target",
  "scratch": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8",
  "brief": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/unit-brief.md",
  "lease": "ess-review-wave8-count-writer",
  "stage": "provisioned; brief written; ready for dispatch"
}
```

The implementation agent count_writer_impl8 is active under that exact on-disk brief. Its first complete package baseline passed; it retained the original runtime source before edits. Both documentation support worktrees were archived and removed by reviewed exact-id cleanup; public-delivery archiveb18e9daedc737f39e4615601ebbb4bb73d1c2a075508786ead945719beaf023f remains in the user cache. Atlas shipment records are published at84cadb68. Root retains two Atlas checkouts deliberately: one clean current-main authority, and one with the already-built fence tool for the pending count-writer shipment; neither is an orphaned implementation unit.

## Compatibility preparation and retained evidence

Root prepared a standalone Rust harness under the AEP reader coordinator's target/ess-conformance-v2-counts/producer-compat. It uses only published AEP source APIs, original report/suite files and independently supplied expected model, exact-suite digest, selected IDs, categories/statuses and evaluation time. Its typed-envelope, checked recording, snapshot/source readback, changed-suite no-mutation refusal, missing-reader refusal and legacy-reader rejection controls passed on a clearly labeled synthetic setup fixture. The actual planning --from/--suite reader also preserved that setup's exact diagnostics in a CLI-created scratch store. Two initial harness setup defects (missing trait import and sha2 output formatting) were corrected with the original build failure retained. None of this is actual ESS producer compatibility; the implementor must first freeze real Rust/Go exports.

Published AEP100fc25b0aa212df0dd549c6bc7ab869a5289eed adds only its internal shipment plan to reader source30aeef2. Root verified that sole changed path and the clean checkout; all reader/API/protocol bytes still correspond to30aeef2. Current Atlas authority84cadb68 records the reader-first shipment and the separate fence limitations.

Coordinator archives are complete and verified in the ESS review user cache: Wave7 has5,364 files/SHA25656733e41fdccf34bc22c66042546c9e9be26ebc476e1c60557aa317acbdeb989; AEP reader evidence2,406 files/SHA25648da639466da4376def1c98d9f3bd912790809bae775370b95e37ff412e66f72, excluding the new in-progress producer compatibility; Atlas reader shipment22 files/SHA256be6dc7a21642e137317018fb589bdf118d7a75caf84b06432b3fb98e4f0f8007. Per-file manifests and earlier failures remain retained. Existing coordinator build directories stay assigned to their own checkouts for the continuing remediation; finished implementation/delivery unit directories are removed.

## Legacy DTO compatibility clarification

The implementor's attempted strict legacy Deserialize wiring exposed two existing parser fixtures; the original failures remain retained. Root kept the old unadmitted DTO/from_json/Deserialize semantics and added the bounded S3/S4 clarification to the accepted binding and story through the CLI. Direct original-byte admission remains strict; every execution entry checks typed membership/vocabulary; serialize-once in-memory input binds only its newly issued buffer. Discarded original JSON cannot be called admitted or acquire that buffer's digest. Required controls prove those separate properties without weakening historical cases. The unit remains active until complete package checks, independent review, actual producer compatibility and integration gate.
