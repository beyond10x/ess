# Wave 13 opening scope computation

Complete draft-only CLI output after cache-wave closure; no entries omitted.

```json
{
  "waves": [
    {
      "wave": 1,
      "artifacts": [
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
            },
            {
              "confidence": "cited",
              "path": "website/docs/guides/verify-conformance.md"
            },
            {
              "confidence": "cited",
              "path": "website/docs/guides/write-a-specification.md"
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
          "id": "story:fuzz-the-specification-surface",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "Taskfile.yml"
            },
            {
              "confidence": "inferred",
              "path": "crates/edge/ess-cli/tests/target_failure.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-synth"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-specification-fuzzing.md"
            },
            {
              "confidence": "cited",
              "path": "fuzz"
            },
            {
              "confidence": "inferred",
              "path": "website/docs/reference/formats.md"
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
          "id": "story:review-glossary-boundaries",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "docs/design/review-concept-boundaries.md"
            },
            {
              "confidence": "cited",
              "path": "website/docs/concepts/ess.md"
            },
            {
              "confidence": "cited",
              "path": "website/docs/guides/record-realization.md"
            },
            {
              "confidence": "cited",
              "path": "website/docs/guides/write-a-specification.md"
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
              "confidence": "cited",
              "path": "docs/design/review-format-catalog.md"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-observation-completeness.md"
            },
            {
              "confidence": "inferred",
              "path": "examples/k3d-dev-cluster"
            },
            {
              "confidence": "cited",
              "path": "website/docs/guides/check-infrastructure.md"
            },
            {
              "confidence": "cited",
              "path": "website/docs/reference/formats.md"
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
      "wave": 5,
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
    }
  ],
  "collisions": [
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:enum-variant-in-an-entity-invariant",
      "path": "crates/specify/ess-domain",
      "confidence": "cited"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:fuzz-the-specification-surface",
      "path": "crates/generate/ess-synth",
      "confidence": "inferred"
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
      "a": "story:fuzz-the-specification-surface",
      "b": "story:integrate-source-driven-realizations",
      "path": "website/docs/reference/formats.md",
      "confidence": "inferred"
    },
    {
      "a": "story:fuzz-the-specification-surface",
      "b": "story:native-realization-ci",
      "path": "Taskfile.yml",
      "confidence": "cited"
    },
    {
      "a": "story:fuzz-the-specification-surface",
      "b": "story:review-consumer-coverage",
      "path": "Taskfile.yml",
      "confidence": "cited"
    },
    {
      "a": "story:fuzz-the-specification-surface",
      "b": "story:review-observation-completeness",
      "path": "website/docs/reference/formats.md",
      "confidence": "inferred"
    },
    {
      "a": "story:fuzz-the-specification-surface",
      "b": "story:review-primitive-semantics",
      "path": "crates/generate/ess-synth",
      "confidence": "cited"
    },
    {
      "a": "story:fuzz-the-specification-surface",
      "b": "story:review-public-support-claims",
      "path": "Taskfile.yml",
      "confidence": "inferred"
    },
    {
      "a": "story:fuzz-the-specification-surface",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "Taskfile.yml",
      "confidence": "cited"
    },
    {
      "a": "story:integrate-source-driven-realizations",
      "b": "story:review-observation-completeness",
      "path": "docs/design/review-format-catalog.md",
      "confidence": "cited"
    },
    {
      "a": "story:integrate-source-driven-realizations",
      "b": "story:review-observation-completeness",
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
      "a": "story:review-authored-discovery",
      "b": "story:review-delivery-trust-contract",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-authored-discovery",
      "b": "story:review-glossary-boundaries",
      "path": "website/docs/guides/write-a-specification.md",
      "confidence": "cited"
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
      "b": "story:schema-unique-items-signed-zero",
      "path": "Cargo.lock",
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
