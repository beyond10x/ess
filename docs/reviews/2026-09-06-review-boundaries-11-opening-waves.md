# Wave 11 opening selection — complete CLI output

Command: `aep plan artifact waves --kind story --status draft --format json`.
Exit status: 0. All lists below are the command output verbatim.

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
      "wave": 4,
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
              "confidence": "cited",
              "path": "crates/verify/ess-diff"
            },
            {
              "confidence": "cited",
              "path": "docs/design/review-conformance-coverage-transport.md"
            },
            {
              "confidence": "cited",
              "path": "docs/design/review-conformance-coverage.md"
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
      "b": "story:review-conformance-coverage",
      "path": "crates/verify/ess-diff",
      "confidence": "inferred"
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
      "b": "story:review-conformance-coverage",
      "path": "docs/design/review-format-catalog.md",
      "confidence": "cited"
    },
    {
      "a": "story:integrate-source-driven-realizations",
      "b": "story:review-conformance-coverage",
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
      "a": "story:review-conformance-coverage",
      "b": "story:review-delivery-trust-contract",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-conformance-coverage",
      "b": "story:review-delivery-trust-contract",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-conformance-coverage",
      "b": "story:review-glossary-boundaries",
      "path": "website/docs",
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
      "b": "story:review-public-support-claims",
      "path": "website/docs",
      "confidence": "inferred"
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
      "a": "story:review-glossary-boundaries",
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

## Primary Atlas status retained without mutation

```text
 M docs/architecture.md
 M docs/catalog.md
?? catalog/store/subjects/61746c61732e636f6d706f6e656e74/6f72672d627261696e2f627261696e2d636865636b.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e74/6f72672d627261696e2f627261696e2d636f7265.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e74/6f72672d627261696e2f627261696e2d6c6564676572.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e74/6f72672d627261696e2f627261696e2d72756c6573.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e74/6f72672d627261696e2f627261696e2d7669657773.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e742d646570656e64656e6379/656e746974792d72756e74696d653a656e746974792d636f72652d2d6f72672d627261696e3a627261696e2d636865636b2f656e746974792d636f7265.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e742d646570656e64656e6379/656e746974792d72756e74696d653a656e746974792d636f72652d2d6f72672d627261696e3a627261696e2d636f72652f656e746974792d636f7265.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e742d646570656e64656e6379/656e746974792d72756e74696d653a656e746974792d636f72652d2d6f72672d627261696e3a627261696e2d6c65646765722f656e746974792d636f7265.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e742d646570656e64656e6379/656e746974792d72756e74696d653a656e746974792d636f72652d2d6f72672d627261696e3a627261696e2d72756c65732f656e746974792d636f7265.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e742d646570656e64656e6379/656e746974792d72756e74696d653a656e746974792d636f72652d2d6f72672d627261696e3a627261696e2d76696577732f656e746974792d636f7265.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e742d646570656e64656e6379/656e746974792d72756e74696d653a656e746974792d71756572792d2d6f72672d627261696e3a627261696e2d636865636b2f656e746974792d7175657279.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e742d646570656e64656e6379/656e746974792d72756e74696d653a656e746974792d71756572792d2d6f72672d627261696e3a627261696e2d6c65646765722f656e746974792d7175657279.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e742d646570656e64656e6379/656e746974792d72756e74696d653a656e746974792d71756572792d2d6f72672d627261696e3a627261696e2d72756c65732f656e746974792d7175657279.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e742d646570656e64656e6379/656e746974792d72756e74696d653a656e746974792d71756572792d2d6f72672d627261696e3a627261696e2d76696577732f656e746974792d7175657279.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e742d646570656e64656e6379/656e746974792d72756e74696d653a656e746974792d7368656c6c2d2d6f72672d627261696e3a627261696e2d636865636b2f656e746974792d7368656c6c.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e742d646570656e64656e6379/656e746974792d72756e74696d653a656e746974792d7368656c6c2d2d6f72672d627261696e3a627261696e2d6c65646765722f656e746974792d7368656c6c.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e742d646570656e64656e6379/656e746974792d72756e74696d653a656e746974792d7368656c6c2d2d6f72672d627261696e3a627261696e2d72756c65732f656e746974792d7368656c6c.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e742d646570656e64656e6379/656e746974792d72756e74696d653a656e746974792d7368656c6c2d2d6f72672d627261696e3a627261696e2d76696577732f656e746974792d7368656c6c.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e742d646570656e64656e6379/656e746974792d72756e74696d653a656e746974792d73746f72652d2d6f72672d627261696e3a627261696e2d636865636b2f656e746974792d73746f7265.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e742d646570656e64656e6379/656e746974792d72756e74696d653a656e746974792d73746f72652d2d6f72672d627261696e3a627261696e2d6c65646765722f656e746974792d73746f7265.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e742d646570656e64656e6379/656e746974792d72756e74696d653a656e746974792d73746f72652d2d6f72672d627261696e3a627261696e2d72756c65732f656e746974792d73746f7265.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e742d646570656e64656e6379/656e746974792d72756e74696d653a656e746974792d73746f72652d2d6f72672d627261696e3a627261696e2d76696577732f656e746974792d73746f7265.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e742d646570656e64656e6379/656e746974792d72756e74696d653a656e746974792d79616d6c2d2d6f72672d627261696e3a627261696e2d636865636b2f656e746974792d79616d6c.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e742d646570656e64656e6379/656e746974792d72756e74696d653a656e746974792d79616d6c2d2d6f72672d627261696e3a627261696e2d636f72652f656e746974792d79616d6c.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e742d646570656e64656e6379/656e746974792d72756e74696d653a656e746974792d79616d6c2d2d6f72672d627261696e3a627261696e2d6c65646765722f656e746974792d79616d6c.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e742d646570656e64656e6379/656e746974792d72756e74696d653a656e746974792d79616d6c2d2d6f72672d627261696e3a627261696e2d72756c65732f656e746974792d79616d6c.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e742d646570656e64656e6379/656e746974792d72756e74696d653a656e746974792d79616d6c2d2d6f72672d627261696e3a627261696e2d76696577732f656e746974792d79616d6c.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e742d646570656e64656e6379/6f72672d627261696e3a627261696e2d636865636b2d2d6f72672d627261696e3a627261696e2f627261696e2d636865636b.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e742d646570656e64656e6379/6f72672d627261696e3a627261696e2d636f72652d2d6f72672d627261696e3a627261696e2d636865636b2f627261696e2d636f7265.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e742d646570656e64656e6379/6f72672d627261696e3a627261696e2d636f72652d2d6f72672d627261696e3a627261696e2d6c65646765722f627261696e2d636f7265.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e742d646570656e64656e6379/6f72672d627261696e3a627261696e2d636f72652d2d6f72672d627261696e3a627261696e2d72756c65732f627261696e2d636f7265.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e742d646570656e64656e6379/6f72672d627261696e3a627261696e2d636f72652d2d6f72672d627261696e3a627261696e2d76696577732f627261696e2d636f7265.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e742d646570656e64656e6379/6f72672d627261696e3a627261696e2d636f72652d2d6f72672d627261696e3a627261696e2f627261696e2d636f7265.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e742d646570656e64656e6379/6f72672d627261696e3a627261696e2d6c65646765722d2d6f72672d627261696e3a627261696e2f627261696e2d6c6564676572.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e742d646570656e64656e6379/6f72672d627261696e3a627261696e2d72756c65732d2d6f72672d627261696e3a627261696e2f627261696e2d72756c6573.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e742d646570656e64656e6379/6f72672d627261696e3a627261696e2d72756c65732d2d6f72672d627261696e3a787461736b2f627261696e2d72756c6573.json
?? catalog/store/subjects/61746c61732e636f6d706f6e656e742d646570656e64656e6379/6f72672d627261696e3a627261696e2d76696577732d2d6f72672d627261696e3a627261696e2f627261696e2d7669657773.json
```

## Planning validation after scope refresh

```text
167 file(s) in /home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/.engineering/planning: 167 artifact(s)
28 review(s) recorded no findings block:
  - review-result:binary64-structural-codecs-adversary-pass1 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:composition-contract-adversary-pass-1 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:composition-contract-adversary-pass-2 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:normalization-base64-adversary-pass2-public states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:normalization-binary64-adversary-pass1-public states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:normalization-binary64-count-integration-pass1 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:normalization-positional-adversary-pass1 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:normalization-raw-json-adversary-pass1-public states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:review-boundaries-2-containment-adversary-pass-2 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:review-boundaries-2-diagnostic-adversary-pass-1 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:review-boundaries-3-delivery-adversary-pass-1 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:review-boundaries-3-typescript-adversary-pass-1 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:review-boundaries-4-infra-adversary-pass-1 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:review-boundaries-5-format-adversary-pass-2 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:review-boundaries-6-recovery-adversary-pass-2 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:review-boundaries-6-scenarios-adversary-pass-1 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:review-boundaries-7-expression-adversary-pass-1 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:review-boundaries-7-openapi-adversary-pass-2 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:review-remediation-design-round-1 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:review-remediation-design-round-2 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:review-remediation-parallel-safety-round-1 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:review-remediation-parallel-safety-round-2 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:review-remediation-scope-round-1 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:review-remediation-scope-round-2 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:schema-resource-identity-adversary-pass-1 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:typescript-equality-binding states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:typescript-normalization-adversary-pass1 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:typescript-normalization-docs-pass1 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
valid
```

