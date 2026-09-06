# ESS review boundaries — wave 12 initial computation

Read-only computation at d1fe6755e842c8ef5486a90493530a39050dae48, before candidate scope refresh. All original output is retained. No selection is implied.

## draft stdout

```json
[
  {
    "id": "story:architecture-review-and-outlook",
    "kind": "story",
    "status": "draft",
    "title": "Review ESS architecture and document its maturity outlook",
    "path": "story/architecture-review-and-outlook.md",
    "relations": [
      {
        "relation": "informed_by",
        "target": "epic:area-layout"
      },
      {
        "relation": "informed_by",
        "target": "epic:oci-component-delivery"
      }
    ],
    "refs": [],
    "blocked_by": []
  },
  {
    "id": "story:component-declares-its-settings",
    "kind": "story",
    "status": "draft",
    "title": "A component declares its settings, and the runtime slots are derived from them",
    "path": "story/component-declares-its-settings.md",
    "relations": [
      {
        "relation": "decomposes",
        "target": "epic:configuration-declared-once"
      }
    ],
    "refs": [],
    "blocked_by": []
  },
  {
    "id": "story:create-only-command-cannot-refuse",
    "kind": "story",
    "status": "draft",
    "title": "A command that only creates cannot declare a refusal",
    "path": "story/create-only-command-cannot-refuse.md",
    "relations": [],
    "refs": [],
    "blocked_by": []
  },
  {
    "id": "story:enum-variant-in-an-entity-invariant",
    "kind": "story",
    "status": "draft",
    "title": "An entity invariant may name an enum variant that does not exist, and validate accepts it",
    "path": "story/enum-variant-in-an-entity-invariant.md",
    "relations": [
      {
        "relation": "informed_by",
        "target": "story:review-expression-typechecking"
      }
    ],
    "refs": [],
    "blocked_by": []
  },
  {
    "id": "story:fuzz-the-specification-surface",
    "kind": "story",
    "status": "draft",
    "title": "Fuzz the specification surface",
    "path": "story/fuzz-the-specification-surface.md",
    "relations": [
      {
        "relation": "decomposes",
        "target": "epic:review-boundary-remediation"
      },
      {
        "relation": "serves",
        "target": "vision:O2"
      }
    ],
    "refs": [],
    "blocked_by": []
  },
  {
    "id": "story:integrate-source-driven-realizations",
    "kind": "story",
    "status": "draft",
    "title": "Integrate source-driven realizations with the remediation baseline",
    "path": "story/integrate-source-driven-realizations.md",
    "relations": [
      {
        "relation": "depends_on",
        "target": "story:review-format-catalog"
      },
      {
        "relation": "depends_on",
        "target": "story:review-rust-target-feasibility"
      },
      {
        "relation": "depends_on",
        "target": "story:types-only-realizations"
      },
      {
        "relation": "depends_on",
        "target": "story:source-pinned-data-normalization"
      }
    ],
    "refs": [],
    "blocked_by": []
  },
  {
    "id": "story:java-conformance-target",
    "kind": "story",
    "status": "draft",
    "title": "A conformance suite can be emitted as a Java test package",
    "path": "story/java-conformance-target.md",
    "relations": [],
    "refs": [],
    "blocked_by": []
  },
  {
    "id": "story:native-realization-ci",
    "kind": "story",
    "status": "draft",
    "title": "Run structural realization compiler checks in CI",
    "path": "story/native-realization-ci.md",
    "relations": [
      {
        "relation": "serves",
        "target": "vision:O2"
      },
      {
        "relation": "informed_by",
        "target": "story:types-only-realizations"
      }
    ],
    "refs": [],
    "blocked_by": []
  },
  {
    "id": "story:normalization-equality-eligibility",
    "kind": "story",
    "status": "draft",
    "title": "Specify numeric equality eligibility and its format compatibility boundary",
    "path": "story/normalization-equality-eligibility.md",
    "relations": [
      {
        "relation": "derived_from",
        "target": "story:typescript-normalization-target"
      },
      {
        "relation": "informed_by",
        "target": "review-result:typescript-equality-binding"
      },
      {
        "relation": "serves",
        "target": "vision:O2"
      }
    ],
    "refs": [],
    "blocked_by": []
  },
  {
    "id": "story:release-status-publication-state",
    "kind": "story",
    "status": "draft",
    "title": "Release status distinguishes drafts from public releases",
    "path": "story/release-status-publication-state.md",
    "relations": [
      {
        "relation": "informed_by",
        "target": "release-plan:consolidated-ess-019"
      }
    ],
    "refs": [],
    "blocked_by": []
  },
  {
    "id": "story:review-authored-discovery",
    "kind": "story",
    "status": "draft",
    "title": "Define predictable discovery for co-located ESS documents",
    "path": "story/review-authored-discovery.md",
    "relations": [
      {
        "relation": "decomposes",
        "target": "epic:review-boundary-remediation"
      },
      {
        "relation": "serves",
        "target": "vision:O2"
      },
      {
        "relation": "depends_on",
        "target": "story:scenarios-directory-compiles-nothing"
      }
    ],
    "refs": [],
    "blocked_by": []
  },
  {
    "id": "story:review-browser-replay-fidelity",
    "kind": "story",
    "status": "draft",
    "title": "Make browser replay faithful to its declared semantic subset",
    "path": "story/review-browser-replay-fidelity.md",
    "relations": [
      {
        "relation": "decomposes",
        "target": "epic:review-boundary-remediation"
      },
      {
        "relation": "serves",
        "target": "vision:O2"
      }
    ],
    "refs": [],
    "blocked_by": []
  },
  {
    "id": "story:review-cache-origin",
    "kind": "story",
    "status": "draft",
    "title": "Verify cached bundle bytes against their OCI identity",
    "path": "story/review-cache-origin.md",
    "relations": [
      {
        "relation": "decomposes",
        "target": "epic:review-boundary-remediation"
      },
      {
        "relation": "serves",
        "target": "vision:O2"
      },
      {
        "relation": "depends_on",
        "target": "story:review-persisted-delivery-validation"
      }
    ],
    "refs": [],
    "blocked_by": []
  },
  {
    "id": "story:review-consumer-coverage",
    "kind": "story",
    "status": "draft",
    "title": "Require explicit consumer coverage for model extensions",
    "path": "story/review-consumer-coverage.md",
    "relations": [
      {
        "relation": "decomposes",
        "target": "epic:review-boundary-remediation"
      },
      {
        "relation": "serves",
        "target": "vision:O2"
      },
      {
        "relation": "depends_on",
        "target": "story:review-semantic-diff-coverage"
      }
    ],
    "refs": [],
    "blocked_by": []
  },
  {
    "id": "story:review-delivery-trust-contract",
    "kind": "story",
    "status": "draft",
    "title": "Distinguish release consistency from verified evidence",
    "path": "story/review-delivery-trust-contract.md",
    "relations": [
      {
        "relation": "decomposes",
        "target": "epic:review-boundary-remediation"
      },
      {
        "relation": "serves",
        "target": "vision:O2"
      },
      {
        "relation": "depends_on",
        "target": "story:review-persisted-delivery-validation"
      },
      {
        "relation": "depends_on",
        "target": "story:review-report-reader-validation"
      }
    ],
    "refs": [],
    "blocked_by": []
  },
  {
    "id": "story:review-glossary-boundaries",
    "kind": "story",
    "status": "draft",
    "title": "Disambiguate ESS logical, interface and delivery concepts",
    "path": "story/review-glossary-boundaries.md",
    "relations": [
      {
        "relation": "decomposes",
        "target": "epic:review-boundary-remediation"
      },
      {
        "relation": "serves",
        "target": "vision:O2"
      }
    ],
    "refs": [],
    "blocked_by": []
  },
  {
    "id": "story:review-observation-completeness",
    "kind": "story",
    "status": "draft",
    "title": "Preserve observation scope and selector uncertainty",
    "path": "story/review-observation-completeness.md",
    "relations": [
      {
        "relation": "decomposes",
        "target": "epic:review-boundary-remediation"
      },
      {
        "relation": "serves",
        "target": "vision:O2"
      },
      {
        "relation": "depends_on",
        "target": "story:review-secret-sanitization"
      },
      {
        "relation": "depends_on",
        "target": "story:review-infra-ir-invariants"
      }
    ],
    "refs": [],
    "blocked_by": []
  },
  {
    "id": "story:review-output-ownership",
    "kind": "story",
    "status": "draft",
    "title": "Make generated output replacement recoverable and ownership-aware",
    "path": "story/review-output-ownership.md",
    "relations": [
      {
        "relation": "decomposes",
        "target": "epic:review-boundary-remediation"
      },
      {
        "relation": "serves",
        "target": "vision:O2"
      },
      {
        "relation": "depends_on",
        "target": "story:review-output-containment"
      }
    ],
    "refs": [],
    "blocked_by": []
  },
  {
    "id": "story:review-primitive-semantics",
    "kind": "story",
    "status": "draft",
    "title": "Align primitive admission and exact numeric semantics",
    "path": "story/review-primitive-semantics.md",
    "relations": [
      {
        "relation": "decomposes",
        "target": "epic:review-boundary-remediation"
      },
      {
        "relation": "serves",
        "target": "vision:O2"
      },
      {
        "relation": "depends_on",
        "target": "story:review-format-catalog"
      }
    ],
    "refs": [],
    "blocked_by": []
  },
  {
    "id": "story:review-public-support-claims",
    "kind": "story",
    "status": "draft",
    "title": "Keep public support claims aligned with shipped evidence",
    "path": "story/review-public-support-claims.md",
    "relations": [
      {
        "relation": "decomposes",
        "target": "epic:review-boundary-remediation"
      },
      {
        "relation": "serves",
        "target": "vision:O2"
      }
    ],
    "refs": [],
    "blocked_by": []
  },
  {
    "id": "story:review-typed-diagnostics",
    "kind": "story",
    "status": "draft",
    "title": "Carry diagnostic identity independently of rendered wording",
    "path": "story/review-typed-diagnostics.md",
    "relations": [
      {
        "relation": "decomposes",
        "target": "epic:review-boundary-remediation"
      },
      {
        "relation": "serves",
        "target": "vision:O2"
      }
    ],
    "refs": [],
    "blocked_by": []
  },
  {
    "id": "story:schema-unique-items-signed-zero",
    "kind": "story",
    "status": "draft",
    "title": "Correct size-dependent schema uniqueness for signed zero",
    "path": "story/schema-unique-items-signed-zero.md",
    "relations": [
      {
        "relation": "serves",
        "target": "vision:O2"
      }
    ],
    "refs": [],
    "blocked_by": []
  },
  {
    "id": "story:the-generated-go-runtime-is-gofmt-clean",
    "kind": "story",
    "status": "draft",
    "title": "The emitted Go runtime is not gofmt-stable, so an adopter's formatter changes it",
    "path": "story/the-generated-go-runtime-is-gofmt-clean.md",
    "relations": [],
    "refs": [],
    "blocked_by": []
  }
]
```

## draft stderr

```text
```

## proposed stdout

```json
[]
```

## proposed stderr

```text
```

## graph stdout

```json
{
  "approval-record:review-remediation-standing-implementation": {
    "id": "approval-record:review-remediation-standing-implementation",
    "kind": "approval-record",
    "version": "2",
    "status": "approved",
    "location": {
      "path": "approval-record/review-remediation-standing-implementation.md"
    },
    "relations": [
      {
        "decides": "epic:review-boundary-remediation"
      }
    ],
    "metadata": {
      "title": "Standing approval for review remediation implementation waves"
    }
  },
  "approval-record:review-remediation-standing-publication": {
    "id": "approval-record:review-remediation-standing-publication",
    "kind": "approval-record",
    "version": "2",
    "status": "approved",
    "location": {
      "path": "approval-record/review-remediation-standing-publication.md"
    },
    "relations": [
      {
        "decides": "epic:review-boundary-remediation"
      }
    ],
    "metadata": {
      "title": "Standing publication and cleanup approval for green remediation waves"
    }
  },
  "decision-blocker:relation-vocabulary": {
    "id": "decision-blocker:relation-vocabulary",
    "kind": "decision-blocker",
    "version": "3",
    "status": "cleared",
    "location": {
      "path": "decision-blocker/relation-vocabulary.md"
    },
    "relations": [
      {
        "blocks": "story:relations-in-the-domain-model"
      }
    ],
    "metadata": {
      "title": "Nobody has decided the relation vocabulary",
      "summary": "Kinds, cardinality form, direction and on_delete are open; default for silence is written in the body.",
      "owner": "ess"
    }
  },
  "epic:area-layout": {
    "id": "epic:area-layout",
    "kind": "epic",
    "version": "4",
    "status": "implemented",
    "location": {
      "path": "epic/area-layout.md"
    },
    "metadata": {
      "title": "Crates grouped by bounded context",
      "summary": "Group the 20 crates into specify, generate, verify, infra and edge; no crate renamed, no consumer re-pins."
    }
  },
  "epic:command-line-surface": {
    "id": "epic:command-line-surface",
    "kind": "epic",
    "version": "4",
    "status": "implemented",
    "location": {
      "path": "epic/command-line-surface.md"
    },
    "metadata": {
      "title": "A command line is a declared surface, not a hand-written one"
    }
  },
  "epic:configuration-declared-once": {
    "id": "epic:configuration-declared-once",
    "kind": "epic",
    "version": "1",
    "status": "draft",
    "location": {
      "path": "epic/configuration-declared-once.md"
    },
    "metadata": {
      "title": "Configuration is declared once and delivered many ways"
    }
  },
  "epic:entity-relations": {
    "id": "epic:entity-relations",
    "kind": "epic",
    "version": "5",
    "status": "implemented",
    "location": {
      "path": "epic/entity-relations.md"
    },
    "metadata": {
      "title": "An entity declares its relations, and the model checks them",
      "summary": "A relations: list on an entity, validated and projected, so an ownership relation is a declared fact rather than a typed id field plus an invariant somebody remembers.",
      "owner": "ess",
      "tags": [
        "domain",
        "relations"
      ]
    }
  },
  "epic:model-driven-interpretation": {
    "id": "epic:model-driven-interpretation",
    "kind": "epic",
    "version": "1",
    "status": "draft",
    "location": {
      "path": "epic/model-driven-interpretation.md"
    },
    "metadata": {
      "title": "A specification can be executed without being implemented",
      "summary": "A model-driven ConformanceTarget that reads the IR, so any specification runs before anybody fills an obligation.",
      "owner": "ess"
    }
  },
  "epic:oci-component-delivery": {
    "id": "epic:oci-component-delivery",
    "kind": "epic",
    "version": "3",
    "status": "active",
    "location": {
      "path": "epic/oci-component-delivery.md"
    },
    "metadata": {
      "title": "OCI-native independent component delivery",
      "summary": "Build, bundle, cache, resolve, and reconcile independently released ESS components."
    }
  },
  "epic:review-boundary-remediation": {
    "id": "epic:review-boundary-remediation",
    "kind": "epic",
    "version": "3",
    "status": "active",
    "location": {
      "path": "epic/review-boundary-remediation.md"
    },
    "relations": [
      {
        "serves": "vision:O2"
      },
      {
        "derived_from": "specification:architecture-review-baseline"
      }
    ],
    "metadata": {
      "title": "Preserve ESS guarantees across boundaries"
    }
  },
  "obligation:review-contract-rollout-coordination": {
    "id": "obligation:review-contract-rollout-coordination",
    "kind": "obligation",
    "version": "5",
    "status": "open",
    "location": {
      "path": "obligation/review-contract-rollout-coordination.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      }
    ],
    "metadata": {
      "title": "Coordinate relying readers before changed ESS formats become defaults"
    }
  },
  "obligation:review-execution-recovery-implementation": {
    "id": "obligation:review-execution-recovery-implementation",
    "kind": "obligation",
    "version": "2",
    "status": "open",
    "location": {
      "path": "obligation/review-execution-recovery-implementation.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      },
      {
        "depends_on": "story:review-execution-recovery-design"
      }
    ],
    "metadata": {
      "title": "Implement finite execution recovery after the typed design"
    }
  },
  "release-plan:consolidated-ess-019": {
    "id": "release-plan:consolidated-ess-019",
    "kind": "release-plan",
    "version": "8",
    "status": "active",
    "location": {
      "path": "release-plan/consolidated-ess-019.md"
    },
    "metadata": {
      "title": "Consolidate ESS worktrees and release 0.19.0"
    }
  },
  "release-plan:normalization-followups-020": {
    "id": "release-plan:normalization-followups-020",
    "kind": "release-plan",
    "version": "13",
    "status": "implemented",
    "location": {
      "path": "release-plan/normalization-followups-020.md"
    },
    "relations": [
      {
        "depends_on": "story:typescript-normalization-target"
      },
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Release ESS normalization followups as 0.20.0 and qualify IVR adoption"
    }
  },
  "resource-blocker:review-wave-storage-capacity": {
    "id": "resource-blocker:review-wave-storage-capacity",
    "kind": "resource-blocker",
    "version": "3",
    "status": "cleared",
    "location": {
      "path": "resource-blocker/review-wave-storage-capacity.md"
    },
    "relations": [
      {
        "blocks": "story:review-secret-sanitization"
      },
      {
        "blocks": "story:review-report-reader-validation"
      }
    ],
    "metadata": {
      "title": "Shared disk capacity prevents remaining wave verification"
    },
    "withholds": "test_result"
  },
  "review-result:adversary-area-layout-round-1": {
    "id": "review-result:adversary-area-layout-round-1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/adversary-area-layout-round-1.md"
    },
    "relations": [
      {
        "reviews": "story:crates-under-area-directories"
      }
    ],
    "metadata": {
      "title": "Adversary, round 1: crates under area directories"
    }
  },
  "review-result:adversary-cli-areas-round-1": {
    "id": "review-result:adversary-cli-areas-round-1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/adversary-cli-areas-round-1.md"
    },
    "relations": [
      {
        "reviews": "story:cli-first-level-is-the-four-areas"
      }
    ],
    "metadata": {
      "title": "Adversary, round 1: ess CLI first level is the four areas"
    }
  },
  "review-result:binary64-count-default-gate-harness": {
    "id": "review-result:binary64-count-default-gate-harness",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/binary64-count-default-gate-harness.md"
    },
    "relations": [
      {
        "reviews": "story:model-binary64-fields"
      }
    ],
    "metadata": {
      "title": "Default gate exposes optional Go compiler override assumption",
      "owner": "coordinator"
    }
  },
  "review-result:binary64-structural-codecs-adversary-pass1": {
    "id": "review-result:binary64-structural-codecs-adversary-pass1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/binary64-structural-codecs-adversary-pass1.md"
    },
    "relations": [
      {
        "reviews": "story:binary64-structural-codecs"
      }
    ],
    "metadata": {
      "title": "Independent review: structural Binary64 codecs, pass 1"
    }
  },
  "review-result:cache-origin-binding-pass1": {
    "id": "review-result:cache-origin-binding-pass1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/cache-origin-binding-pass1.md"
    },
    "relations": [
      {
        "reviews": "story:review-cache-origin"
      }
    ],
    "metadata": {
      "title": "Cache origin candidate binding review"
    }
  },
  "review-result:composition-contract-adversary-pass-1": {
    "id": "review-result:composition-contract-adversary-pass-1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/composition-contract-adversary-pass-1.md"
    },
    "relations": [
      {
        "reviews": "story:review-composition-contract"
      }
    ],
    "metadata": {
      "title": "Composition contract source attack pass 1"
    }
  },
  "review-result:composition-contract-adversary-pass-2": {
    "id": "review-result:composition-contract-adversary-pass-2",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/composition-contract-adversary-pass-2.md"
    },
    "relations": [
      {
        "reviews": "story:review-composition-contract"
      }
    ],
    "metadata": {
      "title": "Composition contract final source attack"
    }
  },
  "review-result:consumer-coverage-binding-pass1": {
    "id": "review-result:consumer-coverage-binding-pass1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/consumer-coverage-binding-pass1.md"
    },
    "relations": [
      {
        "reviews": "story:review-consumer-coverage"
      }
    ],
    "metadata": {
      "title": "Consumer coverage candidate binding review"
    }
  },
  "review-result:consumer-coverage-binding-pass2": {
    "id": "review-result:consumer-coverage-binding-pass2",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/consumer-coverage-binding-pass2.md"
    },
    "relations": [
      {
        "reviews": "story:review-consumer-coverage"
      }
    ],
    "metadata": {
      "title": "Consumer coverage candidate binding correction review"
    }
  },
  "review-result:coverage-writer-source-pass1": {
    "id": "review-result:coverage-writer-source-pass1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/coverage-writer-source-pass1.md"
    },
    "relations": [
      {
        "reviews": "story:review-conformance-coverage"
      }
    ],
    "metadata": {
      "title": "Coverage writer source pass 1: full refusal rendering and strict Go diagnostic"
    }
  },
  "review-result:coverage-writer-source-pass2": {
    "id": "review-result:coverage-writer-source-pass2",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/coverage-writer-source-pass2.md"
    },
    "relations": [
      {
        "reviews": "story:review-conformance-coverage"
      }
    ],
    "metadata": {
      "title": "Coverage writer final source pass: no implementation findings, view test scope disposition"
    }
  },
  "review-result:ess-conformance-coverage-binding-review-pass-1": {
    "id": "review-result:ess-conformance-coverage-binding-review-pass-1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/ess-conformance-coverage-binding-review-pass-1.md"
    },
    "relations": [
      {
        "reviews": "story:review-conformance-coverage"
      }
    ],
    "metadata": {
      "title": "ESS complete-selection binding review"
    }
  },
  "review-result:ess-conformance-coverage-binding-review-pass-2": {
    "id": "review-result:ess-conformance-coverage-binding-review-pass-2",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/ess-conformance-coverage-binding-review-pass-2.md"
    },
    "relations": [
      {
        "reviews": "story:review-conformance-coverage"
      }
    ],
    "metadata": {
      "title": "ESS complete-selection binding correction review"
    }
  },
  "review-result:ess-count-writer-adversary-pass-1": {
    "id": "review-result:ess-count-writer-adversary-pass-1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/ess-count-writer-adversary-pass-1.md"
    },
    "relations": [
      {
        "reviews": "story:a-skipped-scenario-is-not-a-failed-one"
      }
    ],
    "metadata": {
      "title": "ESS count writer first adversarial pass"
    }
  },
  "review-result:ess-count-writer-adversary-pass-2": {
    "id": "review-result:ess-count-writer-adversary-pass-2",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/ess-count-writer-adversary-pass-2.md"
    },
    "relations": [
      {
        "reviews": "story:a-skipped-scenario-is-not-a-failed-one"
      }
    ],
    "metadata": {
      "title": "ESS count writer adversary pass 2"
    }
  },
  "review-result:fuzz-specification-binding-pass1": {
    "id": "review-result:fuzz-specification-binding-pass1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/fuzz-specification-binding-pass1.md"
    },
    "relations": [
      {
        "reviews": "story:fuzz-the-specification-surface"
      }
    ],
    "metadata": {
      "title": "Specification fuzzing candidate binding review"
    }
  },
  "review-result:fuzz-specification-binding-pass2": {
    "id": "review-result:fuzz-specification-binding-pass2",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/fuzz-specification-binding-pass2.md"
    },
    "relations": [
      {
        "reviews": "story:fuzz-the-specification-surface"
      }
    ],
    "metadata": {
      "title": "Specification fuzzing binding correction review"
    }
  },
  "review-result:normalization-base64-adversary-pass1": {
    "id": "review-result:normalization-base64-adversary-pass1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/normalization-base64-adversary-pass1.md"
    },
    "relations": [
      {
        "reviews": "story:go-normalization-pattern-semantics"
      }
    ],
    "metadata": {
      "title": "Frozen base64 qualification adversary pass 1"
    }
  },
  "review-result:normalization-base64-adversary-pass2-public": {
    "id": "review-result:normalization-base64-adversary-pass2-public",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/normalization-base64-adversary-pass2-public.md"
    },
    "relations": [
      {
        "reviews": "story:go-normalization-pattern-semantics"
      }
    ],
    "metadata": {
      "title": "Frozen base64 normalization adversary pass 2"
    }
  },
  "review-result:normalization-binary64-adversary-pass1-public": {
    "id": "review-result:normalization-binary64-adversary-pass1-public",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/normalization-binary64-adversary-pass1-public.md"
    },
    "relations": [
      {
        "reviews": "story:model-binary64-fields"
      }
    ],
    "metadata": {
      "title": "Binary64 model and normalization adversary pass 1",
      "owner": "aep-drive:adversary"
    }
  },
  "review-result:normalization-binary64-count-integration-pass1": {
    "id": "review-result:normalization-binary64-count-integration-pass1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/normalization-binary64-count-integration-pass1.md"
    },
    "relations": [
      {
        "reviews": "story:model-binary64-fields"
      }
    ],
    "metadata": {
      "title": "Binary64 and count-writer integration adversary pass 1",
      "owner": "aep-drive:adversary"
    }
  },
  "review-result:normalization-followup-publication-docs-pass1": {
    "id": "review-result:normalization-followup-publication-docs-pass1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/normalization-followup-publication-docs-pass1.md"
    },
    "relations": [
      {
        "reviews": "story:normalization-followup-publication"
      }
    ],
    "metadata": {
      "title": "Shared normalization documentation review"
    }
  },
  "review-result:normalization-positional-adversary-pass1": {
    "id": "review-result:normalization-positional-adversary-pass1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/normalization-positional-adversary-pass1.md"
    },
    "relations": [
      {
        "reviews": "story:normalize-positional-array-input"
      }
    ],
    "metadata": {
      "title": "Independent positional normalization review, pass 1"
    }
  },
  "review-result:normalization-raw-json-adversary-pass1-public": {
    "id": "review-result:normalization-raw-json-adversary-pass1-public",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/normalization-raw-json-adversary-pass1-public.md"
    },
    "relations": [
      {
        "reviews": "story:raw-json-normalization-provenance"
      }
    ],
    "metadata": {
      "title": "Raw JSON normalization adversary pass 1",
      "owner": "aep-drive:adversary"
    }
  },
  "review-result:review-boundaries-1-report-adversary-pass-1": {
    "id": "review-result:review-boundaries-1-report-adversary-pass-1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/review-boundaries-1-report-adversary-pass-1.md"
    },
    "relations": [
      {
        "reviews": "story:review-report-reader-validation"
      }
    ],
    "metadata": {
      "title": "Report reader adversary pass 1, resource-limited"
    }
  },
  "review-result:review-boundaries-1-secret-adversary-pass-1": {
    "id": "review-result:review-boundaries-1-secret-adversary-pass-1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/review-boundaries-1-secret-adversary-pass-1.md"
    },
    "relations": [
      {
        "reviews": "story:review-secret-sanitization"
      }
    ],
    "metadata": {
      "title": "Secret boundary adversary pass 1"
    }
  },
  "review-result:review-boundaries-2-containment-adversary-pass-1": {
    "id": "review-result:review-boundaries-2-containment-adversary-pass-1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/review-boundaries-2-containment-adversary-pass-1.md"
    },
    "relations": [
      {
        "reviews": "story:review-output-containment"
      }
    ],
    "metadata": {
      "title": "Output containment adversary pass 1"
    }
  },
  "review-result:review-boundaries-2-containment-adversary-pass-2": {
    "id": "review-result:review-boundaries-2-containment-adversary-pass-2",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/review-boundaries-2-containment-adversary-pass-2.md"
    },
    "relations": [
      {
        "reviews": "story:review-output-containment"
      }
    ],
    "metadata": {
      "title": "Output containment second test review"
    }
  },
  "review-result:review-boundaries-2-diagnostic-adversary-pass-1": {
    "id": "review-result:review-boundaries-2-diagnostic-adversary-pass-1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/review-boundaries-2-diagnostic-adversary-pass-1.md"
    },
    "relations": [
      {
        "reviews": "story:review-kubectl-diagnostic-sanitization"
      }
    ],
    "metadata": {
      "title": "Diagnostic adversary pass 1"
    }
  },
  "review-result:review-boundaries-3-conformance-design-adversary-pass-1": {
    "id": "review-result:review-boundaries-3-conformance-design-adversary-pass-1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/review-boundaries-3-conformance-design-adversary-pass-1.md"
    },
    "relations": [
      {
        "reviews": "story:review-conformance-format-design"
      }
    ],
    "metadata": {
      "title": "Wave 3 conformance design adversary pass 1"
    }
  },
  "review-result:review-boundaries-3-conformance-design-adversary-pass-2": {
    "id": "review-result:review-boundaries-3-conformance-design-adversary-pass-2",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/review-boundaries-3-conformance-design-adversary-pass-2.md"
    },
    "relations": [
      {
        "reviews": "story:review-conformance-format-design"
      }
    ],
    "metadata": {
      "title": "Wave 3 conformance design adversary pass 2"
    }
  },
  "review-result:review-boundaries-3-delivery-adversary-pass-1": {
    "id": "review-result:review-boundaries-3-delivery-adversary-pass-1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/review-boundaries-3-delivery-adversary-pass-1.md"
    },
    "relations": [
      {
        "reviews": "story:review-persisted-delivery-validation"
      }
    ],
    "metadata": {
      "title": "Wave 3 delivery adversary pass 1"
    }
  },
  "review-result:review-boundaries-3-typescript-adversary-pass-1": {
    "id": "review-result:review-boundaries-3-typescript-adversary-pass-1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/review-boundaries-3-typescript-adversary-pass-1.md"
    },
    "relations": [
      {
        "reviews": "story:review-typescript-root-collision"
      }
    ],
    "metadata": {
      "title": "Wave 3 TypeScript adversary pass 1"
    }
  },
  "review-result:review-boundaries-4-infra-adversary-pass-1": {
    "id": "review-result:review-boundaries-4-infra-adversary-pass-1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/review-boundaries-4-infra-adversary-pass-1.md"
    },
    "relations": [
      {
        "reviews": "story:review-infra-ir-invariants"
      }
    ],
    "metadata": {
      "title": "Wave 4 infrastructure IR adversary pass 1",
      "summary": "Five independent regression cases pass; no findings at the reviewed implementation."
    }
  },
  "review-result:review-boundaries-4-semantic-adversary-pass-1": {
    "id": "review-result:review-boundaries-4-semantic-adversary-pass-1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/review-boundaries-4-semantic-adversary-pass-1.md"
    },
    "relations": [
      {
        "reviews": "story:review-semantic-diff-coverage"
      }
    ],
    "metadata": {
      "title": "Wave 4 semantic coverage adversary pass 1",
      "summary": "Four counterexamples confirm lost residual attribution and incomplete structured stamp admission."
    }
  },
  "review-result:review-boundaries-4-semantic-adversary-pass-2": {
    "id": "review-result:review-boundaries-4-semantic-adversary-pass-2",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/review-boundaries-4-semantic-adversary-pass-2.md"
    },
    "relations": [
      {
        "reviews": "story:review-semantic-diff-coverage"
      }
    ],
    "metadata": {
      "title": "Wave 4 semantic coverage adversary pass 2",
      "summary": "Three counterexamples confirm omitted network views and reusable row shapes in dependency slices."
    }
  },
  "review-result:review-boundaries-5-format-adversary-pass-1": {
    "id": "review-result:review-boundaries-5-format-adversary-pass-1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/review-boundaries-5-format-adversary-pass-1.md"
    },
    "relations": [
      {
        "reviews": "story:review-format-catalog"
      }
    ],
    "metadata": {
      "title": "Format catalog source and fixture adversary pass 1",
      "owner": "impl_diagnostic"
    }
  },
  "review-result:review-boundaries-5-format-adversary-pass-2": {
    "id": "review-result:review-boundaries-5-format-adversary-pass-2",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/review-boundaries-5-format-adversary-pass-2.md"
    },
    "relations": [
      {
        "reviews": "story:review-format-catalog"
      }
    ],
    "metadata": {
      "title": "Format catalog source and fixture adversary pass 2",
      "owner": "scope_conformance_design"
    }
  },
  "review-result:review-boundaries-5-rust-adversary-pass-1": {
    "id": "review-result:review-boundaries-5-rust-adversary-pass-1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/review-boundaries-5-rust-adversary-pass-1.md"
    },
    "relations": [
      {
        "reviews": "story:review-rust-target-feasibility"
      }
    ],
    "metadata": {
      "title": "Rust feasibility compiler and CLI adversary pass 1",
      "owner": "impl_diagnostic"
    }
  },
  "review-result:review-boundaries-5-rust-adversary-pass-2": {
    "id": "review-result:review-boundaries-5-rust-adversary-pass-2",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/review-boundaries-5-rust-adversary-pass-2.md"
    },
    "relations": [
      {
        "reviews": "story:review-rust-target-feasibility"
      }
    ],
    "metadata": {
      "title": "Final Rust feasibility adversary pass at a2ff04d",
      "owner": "impl_diagnostic"
    }
  },
  "review-result:review-boundaries-6-recovery-adversary-pass-1": {
    "id": "review-result:review-boundaries-6-recovery-adversary-pass-1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/review-boundaries-6-recovery-adversary-pass-1.md"
    },
    "relations": [
      {
        "reviews": "story:review-execution-recovery-design"
      }
    ],
    "metadata": {
      "title": "Wave 6 recovery design adversary pass 1"
    }
  },
  "review-result:review-boundaries-6-recovery-adversary-pass-2": {
    "id": "review-result:review-boundaries-6-recovery-adversary-pass-2",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/review-boundaries-6-recovery-adversary-pass-2.md"
    },
    "relations": [
      {
        "reviews": "story:review-execution-recovery-design"
      }
    ],
    "metadata": {
      "title": "Wave 6 recovery adversary pass 2"
    }
  },
  "review-result:review-boundaries-6-scenarios-adversary-pass-1": {
    "id": "review-result:review-boundaries-6-scenarios-adversary-pass-1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/review-boundaries-6-scenarios-adversary-pass-1.md"
    },
    "relations": [
      {
        "reviews": "story:scenarios-directory-compiles-nothing"
      }
    ],
    "metadata": {
      "title": "Wave 6 scenarios adversary pass 1"
    }
  },
  "review-result:review-boundaries-7-expression-adversary-pass-1": {
    "id": "review-result:review-boundaries-7-expression-adversary-pass-1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/review-boundaries-7-expression-adversary-pass-1.md"
    },
    "relations": [
      {
        "reviews": "story:review-expression-typechecking"
      }
    ],
    "metadata": {
      "title": "Expression typechecking independent review"
    }
  },
  "review-result:review-boundaries-7-openapi-adversary-pass-1": {
    "id": "review-result:review-boundaries-7-openapi-adversary-pass-1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/review-boundaries-7-openapi-adversary-pass-1.md"
    },
    "relations": [
      {
        "reviews": "story:review-openapi-semantic-accounting"
      }
    ],
    "metadata": {
      "title": "OpenAPI accounting first independent review"
    }
  },
  "review-result:review-boundaries-7-openapi-adversary-pass-2": {
    "id": "review-result:review-boundaries-7-openapi-adversary-pass-2",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/review-boundaries-7-openapi-adversary-pass-2.md"
    },
    "relations": [
      {
        "reviews": "story:review-openapi-semantic-accounting"
      }
    ],
    "metadata": {
      "title": "OpenAPI accounting final independent review"
    }
  },
  "review-result:review-remediation-acceptance-round-1": {
    "id": "review-result:review-remediation-acceptance-round-1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/review-remediation-acceptance-round-1.md"
    },
    "relations": [
      {
        "reviews": "epic:review-boundary-remediation"
      },
      {
        "reviews": "obligation:review-contract-rollout-coordination"
      },
      {
        "reviews": "obligation:review-execution-recovery-implementation"
      },
      {
        "reviews": "story:a-skipped-scenario-is-not-a-failed-one"
      },
      {
        "reviews": "story:fuzz-the-specification-surface"
      },
      {
        "reviews": "story:review-authored-discovery"
      },
      {
        "reviews": "story:review-browser-replay-fidelity"
      },
      {
        "reviews": "story:review-cache-origin"
      },
      {
        "reviews": "story:review-composition-contract"
      },
      {
        "reviews": "story:review-conformance-coverage"
      },
      {
        "reviews": "story:review-conformance-format-design"
      },
      {
        "reviews": "story:review-consumer-coverage"
      },
      {
        "reviews": "story:review-delivery-trust-contract"
      },
      {
        "reviews": "story:review-execution-recovery-design"
      },
      {
        "reviews": "story:review-expression-typechecking"
      },
      {
        "reviews": "story:review-format-catalog"
      },
      {
        "reviews": "story:review-glossary-boundaries"
      },
      {
        "reviews": "story:review-infra-ir-invariants"
      },
      {
        "reviews": "story:review-observation-completeness"
      },
      {
        "reviews": "story:review-openapi-semantic-accounting"
      },
      {
        "reviews": "story:review-output-containment"
      },
      {
        "reviews": "story:review-output-ownership"
      },
      {
        "reviews": "story:review-persisted-delivery-validation"
      },
      {
        "reviews": "story:review-primitive-semantics"
      },
      {
        "reviews": "story:review-public-support-claims"
      },
      {
        "reviews": "story:review-report-reader-validation"
      },
      {
        "reviews": "story:review-rust-target-feasibility"
      },
      {
        "reviews": "story:review-schema-resource-identity"
      },
      {
        "reviews": "story:review-secret-sanitization"
      },
      {
        "reviews": "story:review-semantic-diff-coverage"
      },
      {
        "reviews": "story:review-typed-diagnostics"
      },
      {
        "reviews": "story:review-typescript-root-collision"
      },
      {
        "reviews": "story:scenarios-directory-compiles-nothing"
      }
    ],
    "metadata": {
      "title": "Review remediation acceptance critic, round 1"
    }
  },
  "review-result:review-remediation-acceptance-round-2": {
    "id": "review-result:review-remediation-acceptance-round-2",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/review-remediation-acceptance-round-2.md"
    },
    "relations": [
      {
        "reviews": "epic:review-boundary-remediation"
      },
      {
        "reviews": "obligation:review-contract-rollout-coordination"
      },
      {
        "reviews": "obligation:review-execution-recovery-implementation"
      },
      {
        "reviews": "story:a-skipped-scenario-is-not-a-failed-one"
      },
      {
        "reviews": "story:fuzz-the-specification-surface"
      },
      {
        "reviews": "story:review-authored-discovery"
      },
      {
        "reviews": "story:review-browser-replay-fidelity"
      },
      {
        "reviews": "story:review-cache-origin"
      },
      {
        "reviews": "story:review-composition-contract"
      },
      {
        "reviews": "story:review-conformance-coverage"
      },
      {
        "reviews": "story:review-conformance-format-design"
      },
      {
        "reviews": "story:review-consumer-coverage"
      },
      {
        "reviews": "story:review-delivery-trust-contract"
      },
      {
        "reviews": "story:review-execution-recovery-design"
      },
      {
        "reviews": "story:review-expression-typechecking"
      },
      {
        "reviews": "story:review-format-catalog"
      },
      {
        "reviews": "story:review-glossary-boundaries"
      },
      {
        "reviews": "story:review-infra-ir-invariants"
      },
      {
        "reviews": "story:review-observation-completeness"
      },
      {
        "reviews": "story:review-openapi-semantic-accounting"
      },
      {
        "reviews": "story:review-output-containment"
      },
      {
        "reviews": "story:review-output-ownership"
      },
      {
        "reviews": "story:review-persisted-delivery-validation"
      },
      {
        "reviews": "story:review-primitive-semantics"
      },
      {
        "reviews": "story:review-public-support-claims"
      },
      {
        "reviews": "story:review-report-reader-validation"
      },
      {
        "reviews": "story:review-rust-target-feasibility"
      },
      {
        "reviews": "story:review-schema-resource-identity"
      },
      {
        "reviews": "story:review-secret-sanitization"
      },
      {
        "reviews": "story:review-semantic-diff-coverage"
      },
      {
        "reviews": "story:review-typed-diagnostics"
      },
      {
        "reviews": "story:review-typescript-root-collision"
      },
      {
        "reviews": "story:scenarios-directory-compiles-nothing"
      }
    ],
    "metadata": {
      "title": "Review remediation acceptance critic, round 2"
    }
  },
  "review-result:review-remediation-design-round-1": {
    "id": "review-result:review-remediation-design-round-1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/review-remediation-design-round-1.md"
    },
    "relations": [
      {
        "reviews": "epic:review-boundary-remediation"
      },
      {
        "reviews": "obligation:review-contract-rollout-coordination"
      },
      {
        "reviews": "obligation:review-execution-recovery-implementation"
      },
      {
        "reviews": "story:a-skipped-scenario-is-not-a-failed-one"
      },
      {
        "reviews": "story:fuzz-the-specification-surface"
      },
      {
        "reviews": "story:review-authored-discovery"
      },
      {
        "reviews": "story:review-browser-replay-fidelity"
      },
      {
        "reviews": "story:review-cache-origin"
      },
      {
        "reviews": "story:review-composition-contract"
      },
      {
        "reviews": "story:review-conformance-coverage"
      },
      {
        "reviews": "story:review-conformance-format-design"
      },
      {
        "reviews": "story:review-consumer-coverage"
      },
      {
        "reviews": "story:review-delivery-trust-contract"
      },
      {
        "reviews": "story:review-execution-recovery-design"
      },
      {
        "reviews": "story:review-expression-typechecking"
      },
      {
        "reviews": "story:review-format-catalog"
      },
      {
        "reviews": "story:review-glossary-boundaries"
      },
      {
        "reviews": "story:review-infra-ir-invariants"
      },
      {
        "reviews": "story:review-observation-completeness"
      },
      {
        "reviews": "story:review-openapi-semantic-accounting"
      },
      {
        "reviews": "story:review-output-containment"
      },
      {
        "reviews": "story:review-output-ownership"
      },
      {
        "reviews": "story:review-persisted-delivery-validation"
      },
      {
        "reviews": "story:review-primitive-semantics"
      },
      {
        "reviews": "story:review-public-support-claims"
      },
      {
        "reviews": "story:review-report-reader-validation"
      },
      {
        "reviews": "story:review-rust-target-feasibility"
      },
      {
        "reviews": "story:review-schema-resource-identity"
      },
      {
        "reviews": "story:review-secret-sanitization"
      },
      {
        "reviews": "story:review-semantic-diff-coverage"
      },
      {
        "reviews": "story:review-typed-diagnostics"
      },
      {
        "reviews": "story:review-typescript-root-collision"
      },
      {
        "reviews": "story:scenarios-directory-compiles-nothing"
      }
    ],
    "metadata": {
      "title": "Review remediation design critic, round 1"
    }
  },
  "review-result:review-remediation-design-round-2": {
    "id": "review-result:review-remediation-design-round-2",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/review-remediation-design-round-2.md"
    },
    "relations": [
      {
        "reviews": "epic:review-boundary-remediation"
      },
      {
        "reviews": "obligation:review-contract-rollout-coordination"
      },
      {
        "reviews": "obligation:review-execution-recovery-implementation"
      },
      {
        "reviews": "story:a-skipped-scenario-is-not-a-failed-one"
      },
      {
        "reviews": "story:fuzz-the-specification-surface"
      },
      {
        "reviews": "story:review-authored-discovery"
      },
      {
        "reviews": "story:review-browser-replay-fidelity"
      },
      {
        "reviews": "story:review-cache-origin"
      },
      {
        "reviews": "story:review-composition-contract"
      },
      {
        "reviews": "story:review-conformance-coverage"
      },
      {
        "reviews": "story:review-conformance-format-design"
      },
      {
        "reviews": "story:review-consumer-coverage"
      },
      {
        "reviews": "story:review-delivery-trust-contract"
      },
      {
        "reviews": "story:review-execution-recovery-design"
      },
      {
        "reviews": "story:review-expression-typechecking"
      },
      {
        "reviews": "story:review-format-catalog"
      },
      {
        "reviews": "story:review-glossary-boundaries"
      },
      {
        "reviews": "story:review-infra-ir-invariants"
      },
      {
        "reviews": "story:review-observation-completeness"
      },
      {
        "reviews": "story:review-openapi-semantic-accounting"
      },
      {
        "reviews": "story:review-output-containment"
      },
      {
        "reviews": "story:review-output-ownership"
      },
      {
        "reviews": "story:review-persisted-delivery-validation"
      },
      {
        "reviews": "story:review-primitive-semantics"
      },
      {
        "reviews": "story:review-public-support-claims"
      },
      {
        "reviews": "story:review-report-reader-validation"
      },
      {
        "reviews": "story:review-rust-target-feasibility"
      },
      {
        "reviews": "story:review-schema-resource-identity"
      },
      {
        "reviews": "story:review-secret-sanitization"
      },
      {
        "reviews": "story:review-semantic-diff-coverage"
      },
      {
        "reviews": "story:review-typed-diagnostics"
      },
      {
        "reviews": "story:review-typescript-root-collision"
      },
      {
        "reviews": "story:scenarios-directory-compiles-nothing"
      }
    ],
    "metadata": {
      "title": "Review remediation design critic, round 2"
    }
  },
  "review-result:review-remediation-parallel-safety-round-1": {
    "id": "review-result:review-remediation-parallel-safety-round-1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/review-remediation-parallel-safety-round-1.md"
    },
    "relations": [
      {
        "reviews": "epic:review-boundary-remediation"
      },
      {
        "reviews": "obligation:review-contract-rollout-coordination"
      },
      {
        "reviews": "obligation:review-execution-recovery-implementation"
      },
      {
        "reviews": "story:a-skipped-scenario-is-not-a-failed-one"
      },
      {
        "reviews": "story:fuzz-the-specification-surface"
      },
      {
        "reviews": "story:review-authored-discovery"
      },
      {
        "reviews": "story:review-browser-replay-fidelity"
      },
      {
        "reviews": "story:review-cache-origin"
      },
      {
        "reviews": "story:review-composition-contract"
      },
      {
        "reviews": "story:review-conformance-coverage"
      },
      {
        "reviews": "story:review-conformance-format-design"
      },
      {
        "reviews": "story:review-consumer-coverage"
      },
      {
        "reviews": "story:review-delivery-trust-contract"
      },
      {
        "reviews": "story:review-execution-recovery-design"
      },
      {
        "reviews": "story:review-expression-typechecking"
      },
      {
        "reviews": "story:review-format-catalog"
      },
      {
        "reviews": "story:review-glossary-boundaries"
      },
      {
        "reviews": "story:review-infra-ir-invariants"
      },
      {
        "reviews": "story:review-observation-completeness"
      },
      {
        "reviews": "story:review-openapi-semantic-accounting"
      },
      {
        "reviews": "story:review-output-containment"
      },
      {
        "reviews": "story:review-output-ownership"
      },
      {
        "reviews": "story:review-persisted-delivery-validation"
      },
      {
        "reviews": "story:review-primitive-semantics"
      },
      {
        "reviews": "story:review-public-support-claims"
      },
      {
        "reviews": "story:review-report-reader-validation"
      },
      {
        "reviews": "story:review-rust-target-feasibility"
      },
      {
        "reviews": "story:review-schema-resource-identity"
      },
      {
        "reviews": "story:review-secret-sanitization"
      },
      {
        "reviews": "story:review-semantic-diff-coverage"
      },
      {
        "reviews": "story:review-typed-diagnostics"
      },
      {
        "reviews": "story:review-typescript-root-collision"
      },
      {
        "reviews": "story:scenarios-directory-compiles-nothing"
      }
    ],
    "metadata": {
      "title": "Review remediation parallel-safety critic, round 1"
    }
  },
  "review-result:review-remediation-parallel-safety-round-2": {
    "id": "review-result:review-remediation-parallel-safety-round-2",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/review-remediation-parallel-safety-round-2.md"
    },
    "relations": [
      {
        "reviews": "epic:review-boundary-remediation"
      },
      {
        "reviews": "obligation:review-contract-rollout-coordination"
      },
      {
        "reviews": "obligation:review-execution-recovery-implementation"
      },
      {
        "reviews": "story:a-skipped-scenario-is-not-a-failed-one"
      },
      {
        "reviews": "story:fuzz-the-specification-surface"
      },
      {
        "reviews": "story:review-authored-discovery"
      },
      {
        "reviews": "story:review-browser-replay-fidelity"
      },
      {
        "reviews": "story:review-cache-origin"
      },
      {
        "reviews": "story:review-composition-contract"
      },
      {
        "reviews": "story:review-conformance-coverage"
      },
      {
        "reviews": "story:review-conformance-format-design"
      },
      {
        "reviews": "story:review-consumer-coverage"
      },
      {
        "reviews": "story:review-delivery-trust-contract"
      },
      {
        "reviews": "story:review-execution-recovery-design"
      },
      {
        "reviews": "story:review-expression-typechecking"
      },
      {
        "reviews": "story:review-format-catalog"
      },
      {
        "reviews": "story:review-glossary-boundaries"
      },
      {
        "reviews": "story:review-infra-ir-invariants"
      },
      {
        "reviews": "story:review-observation-completeness"
      },
      {
        "reviews": "story:review-openapi-semantic-accounting"
      },
      {
        "reviews": "story:review-output-containment"
      },
      {
        "reviews": "story:review-output-ownership"
      },
      {
        "reviews": "story:review-persisted-delivery-validation"
      },
      {
        "reviews": "story:review-primitive-semantics"
      },
      {
        "reviews": "story:review-public-support-claims"
      },
      {
        "reviews": "story:review-report-reader-validation"
      },
      {
        "reviews": "story:review-rust-target-feasibility"
      },
      {
        "reviews": "story:review-schema-resource-identity"
      },
      {
        "reviews": "story:review-secret-sanitization"
      },
      {
        "reviews": "story:review-semantic-diff-coverage"
      },
      {
        "reviews": "story:review-typed-diagnostics"
      },
      {
        "reviews": "story:review-typescript-root-collision"
      },
      {
        "reviews": "story:scenarios-directory-compiles-nothing"
      }
    ],
    "metadata": {
      "title": "Review remediation parallel-safety critic, round 2"
    }
  },
  "review-result:review-remediation-scope-round-1": {
    "id": "review-result:review-remediation-scope-round-1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/review-remediation-scope-round-1.md"
    },
    "relations": [
      {
        "reviews": "epic:review-boundary-remediation"
      },
      {
        "reviews": "obligation:review-contract-rollout-coordination"
      },
      {
        "reviews": "obligation:review-execution-recovery-implementation"
      },
      {
        "reviews": "story:a-skipped-scenario-is-not-a-failed-one"
      },
      {
        "reviews": "story:fuzz-the-specification-surface"
      },
      {
        "reviews": "story:review-authored-discovery"
      },
      {
        "reviews": "story:review-browser-replay-fidelity"
      },
      {
        "reviews": "story:review-cache-origin"
      },
      {
        "reviews": "story:review-composition-contract"
      },
      {
        "reviews": "story:review-conformance-coverage"
      },
      {
        "reviews": "story:review-conformance-format-design"
      },
      {
        "reviews": "story:review-consumer-coverage"
      },
      {
        "reviews": "story:review-delivery-trust-contract"
      },
      {
        "reviews": "story:review-execution-recovery-design"
      },
      {
        "reviews": "story:review-expression-typechecking"
      },
      {
        "reviews": "story:review-format-catalog"
      },
      {
        "reviews": "story:review-glossary-boundaries"
      },
      {
        "reviews": "story:review-infra-ir-invariants"
      },
      {
        "reviews": "story:review-observation-completeness"
      },
      {
        "reviews": "story:review-openapi-semantic-accounting"
      },
      {
        "reviews": "story:review-output-containment"
      },
      {
        "reviews": "story:review-output-ownership"
      },
      {
        "reviews": "story:review-persisted-delivery-validation"
      },
      {
        "reviews": "story:review-primitive-semantics"
      },
      {
        "reviews": "story:review-public-support-claims"
      },
      {
        "reviews": "story:review-report-reader-validation"
      },
      {
        "reviews": "story:review-rust-target-feasibility"
      },
      {
        "reviews": "story:review-schema-resource-identity"
      },
      {
        "reviews": "story:review-secret-sanitization"
      },
      {
        "reviews": "story:review-semantic-diff-coverage"
      },
      {
        "reviews": "story:review-typed-diagnostics"
      },
      {
        "reviews": "story:review-typescript-root-collision"
      },
      {
        "reviews": "story:scenarios-directory-compiles-nothing"
      }
    ],
    "metadata": {
      "title": "Review remediation scope critic, round 1"
    }
  },
  "review-result:review-remediation-scope-round-2": {
    "id": "review-result:review-remediation-scope-round-2",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/review-remediation-scope-round-2.md"
    },
    "relations": [
      {
        "reviews": "epic:review-boundary-remediation"
      },
      {
        "reviews": "obligation:review-contract-rollout-coordination"
      },
      {
        "reviews": "obligation:review-execution-recovery-implementation"
      },
      {
        "reviews": "story:a-skipped-scenario-is-not-a-failed-one"
      },
      {
        "reviews": "story:fuzz-the-specification-surface"
      },
      {
        "reviews": "story:review-authored-discovery"
      },
      {
        "reviews": "story:review-browser-replay-fidelity"
      },
      {
        "reviews": "story:review-cache-origin"
      },
      {
        "reviews": "story:review-composition-contract"
      },
      {
        "reviews": "story:review-conformance-coverage"
      },
      {
        "reviews": "story:review-conformance-format-design"
      },
      {
        "reviews": "story:review-consumer-coverage"
      },
      {
        "reviews": "story:review-delivery-trust-contract"
      },
      {
        "reviews": "story:review-execution-recovery-design"
      },
      {
        "reviews": "story:review-expression-typechecking"
      },
      {
        "reviews": "story:review-format-catalog"
      },
      {
        "reviews": "story:review-glossary-boundaries"
      },
      {
        "reviews": "story:review-infra-ir-invariants"
      },
      {
        "reviews": "story:review-observation-completeness"
      },
      {
        "reviews": "story:review-openapi-semantic-accounting"
      },
      {
        "reviews": "story:review-output-containment"
      },
      {
        "reviews": "story:review-output-ownership"
      },
      {
        "reviews": "story:review-persisted-delivery-validation"
      },
      {
        "reviews": "story:review-primitive-semantics"
      },
      {
        "reviews": "story:review-public-support-claims"
      },
      {
        "reviews": "story:review-report-reader-validation"
      },
      {
        "reviews": "story:review-rust-target-feasibility"
      },
      {
        "reviews": "story:review-schema-resource-identity"
      },
      {
        "reviews": "story:review-secret-sanitization"
      },
      {
        "reviews": "story:review-semantic-diff-coverage"
      },
      {
        "reviews": "story:review-typed-diagnostics"
      },
      {
        "reviews": "story:review-typescript-root-collision"
      },
      {
        "reviews": "story:scenarios-directory-compiles-nothing"
      }
    ],
    "metadata": {
      "title": "Review remediation scope critic, round 2"
    }
  },
  "review-result:schema-resource-identity-adversary-pass-1": {
    "id": "review-result:schema-resource-identity-adversary-pass-1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/schema-resource-identity-adversary-pass-1.md"
    },
    "relations": [
      {
        "reviews": "story:review-schema-resource-identity"
      }
    ],
    "metadata": {
      "title": "Schema resource identity independent source adversary"
    }
  },
  "review-result:typescript-equality-binding": {
    "id": "review-result:typescript-equality-binding",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/typescript-equality-binding.md"
    },
    "relations": [
      {
        "reviews": "story:typescript-normalization-target"
      }
    ],
    "metadata": {
      "title": "Qualify frozen numeric equality behavior in the TypeScript binding"
    }
  },
  "review-result:typescript-normalization-adversary-pass1": {
    "id": "review-result:typescript-normalization-adversary-pass1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/typescript-normalization-adversary-pass1.md"
    },
    "relations": [
      {
        "reviews": "story:typescript-normalization-target"
      }
    ],
    "metadata": {
      "title": "Independent TypeScript normalization source and native attack"
    }
  },
  "review-result:typescript-normalization-docs-pass1": {
    "id": "review-result:typescript-normalization-docs-pass1",
    "kind": "review-result",
    "version": "1",
    "status": "active",
    "location": {
      "path": "review-result/typescript-normalization-docs-pass1.md"
    },
    "relations": [
      {
        "reviews": "story:typescript-normalization-target"
      }
    ],
    "metadata": {
      "title": "Review TypeScript normalization adopter and binding documentation"
    }
  },
  "specification:architecture-review-baseline": {
    "id": "specification:architecture-review-baseline",
    "kind": "specification",
    "version": "1",
    "status": "draft",
    "location": {
      "path": "specification/architecture-review-baseline.md"
    },
    "metadata": {
      "title": "ESS architecture review baseline at fd06a4d"
    }
  },
  "specification:construct-provenance-references": {
    "id": "specification:construct-provenance-references",
    "kind": "specification",
    "version": "5",
    "status": "implemented",
    "location": {
      "path": "specification/construct-provenance-references.md"
    },
    "metadata": {
      "title": "Construct provenance references",
      "summary": "Supported ESS constructs retain typed external records explaining why they exist.",
      "owner": "ess",
      "tags": [
        "github-issue-migration"
      ],
      "refs": [
        {
          "provider": "github",
          "reference": "beyond10x/ess#3"
        }
      ]
    }
  },
  "specification:existing-struct-generation": {
    "id": "specification:existing-struct-generation",
    "kind": "specification",
    "version": "1",
    "status": "draft",
    "location": {
      "path": "specification/existing-struct-generation.md"
    },
    "metadata": {
      "title": "Existing structural generation capabilities"
    }
  },
  "specification:static-server-safe-generated-doc-paths": {
    "id": "specification:static-server-safe-generated-doc-paths",
    "kind": "specification",
    "version": "4",
    "status": "implemented",
    "location": {
      "path": "specification/static-server-safe-generated-doc-paths.md"
    },
    "metadata": {
      "title": "Static-server-safe generated documentation paths",
      "summary": "Generated domain documentation uses a stable path that static file servers can serve.",
      "owner": "ess",
      "tags": [
        "github-issue-migration"
      ],
      "refs": [
        {
          "provider": "github",
          "reference": "beyond10x/ess#2"
        }
      ]
    }
  },
  "story:a-skipped-scenario-is-not-a-failed-one": {
    "id": "story:a-skipped-scenario-is-not-a-failed-one",
    "kind": "story",
    "version": "24",
    "status": "implemented",
    "location": {
      "path": "story/a-skipped-scenario-is-not-a-failed-one.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      },
      {
        "serves": "vision:O2"
      },
      {
        "depends_on": "story:review-conformance-format-design"
      },
      {
        "depends_on": "story:review-report-reader-validation"
      }
    ],
    "metadata": {
      "title": "A conformance report counts skipped scenarios as failed"
    }
  },
  "story:align-normalization-v2-format-catalog": {
    "id": "story:align-normalization-v2-format-catalog",
    "kind": "story",
    "version": "6",
    "status": "implemented",
    "location": {
      "path": "story/align-normalization-v2-format-catalog.md"
    },
    "relations": [
      {
        "serves": "vision:O2"
      },
      {
        "informed_by": "story:review-format-catalog"
      },
      {
        "informed_by": "story:source-pinned-data-normalization"
      }
    ],
    "metadata": {
      "title": "Align the internal normalization format catalog"
    }
  },
  "story:architecture-review-and-outlook": {
    "id": "story:architecture-review-and-outlook",
    "kind": "story",
    "version": "6",
    "status": "draft",
    "location": {
      "path": "story/architecture-review-and-outlook.md"
    },
    "relations": [
      {
        "informed_by": "epic:area-layout"
      },
      {
        "informed_by": "epic:oci-component-delivery"
      }
    ],
    "metadata": {
      "title": "Review ESS architecture and document its maturity outlook"
    }
  },
  "story:authored-mermaid-rendering": {
    "id": "story:authored-mermaid-rendering",
    "kind": "story",
    "version": "5",
    "status": "implemented",
    "location": {
      "path": "story/authored-mermaid-rendering.md"
    },
    "relations": [
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Render authored Mermaid fences in generated sites"
    }
  },
  "story:authored-scenarios": {
    "id": "story:authored-scenarios",
    "kind": "story",
    "version": "4",
    "status": "implemented",
    "location": {
      "path": "story/authored-scenarios.md"
    },
    "metadata": {
      "title": "A specification carries the scenarios an author wrote, not only the ones it obliges"
    }
  },
  "story:authored-site-link-resolution": {
    "id": "story:authored-site-link-resolution",
    "kind": "story",
    "version": "5",
    "status": "implemented",
    "location": {
      "path": "story/authored-site-link-resolution.md"
    },
    "relations": [
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Resolve authored documentation links in generated sites"
    }
  },
  "story:authored-site-prose-wrapping": {
    "id": "story:authored-site-prose-wrapping",
    "kind": "story",
    "version": "5",
    "status": "implemented",
    "location": {
      "path": "story/authored-site-prose-wrapping.md"
    },
    "relations": [
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Keep long authored identifiers within narrow viewports"
    }
  },
  "story:binary64-structural-codecs": {
    "id": "story:binary64-structural-codecs",
    "kind": "story",
    "version": "9",
    "status": "implemented",
    "location": {
      "path": "story/binary64-structural-codecs.md"
    },
    "relations": [
      {
        "derived_from": "story:model-binary64-fields"
      },
      {
        "depends_on": "story:model-binary64-fields"
      },
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Preserve finite Binary64 in standalone model data libraries"
    }
  },
  "story:canonical-build-ir-roundtrip": {
    "id": "story:canonical-build-ir-roundtrip",
    "kind": "story",
    "version": "6",
    "status": "implemented",
    "location": {
      "path": "story/canonical-build-ir-roundtrip.md"
    },
    "relations": [
      {
        "decomposes": "epic:oci-component-delivery"
      }
    ],
    "metadata": {
      "title": "Canonical build IR round-trips",
      "summary": "Generated build IR with omitted empty collections remains readable by every ESS release command."
    }
  },
  "story:cli-first-level-is-the-four-areas": {
    "id": "story:cli-first-level-is-the-four-areas",
    "kind": "story",
    "version": "6",
    "status": "implemented",
    "location": {
      "path": "story/cli-first-level-is-the-four-areas.md"
    },
    "metadata": {
      "title": "ess --help shows the four areas; every flat verb stays as a hidden alias",
      "summary": "Group the 20 verbs under specify, generate, verify, infra with hidden flat aliases and a clap-tree test."
    }
  },
  "story:command-line-surface": {
    "id": "story:command-line-surface",
    "kind": "story",
    "version": "12",
    "status": "implemented",
    "location": {
      "path": "story/command-line-surface.md"
    },
    "relations": [
      {
        "decomposes": "epic:command-line-surface"
      }
    ],
    "metadata": {
      "title": "A component declares a command-line surface, and ESS synthesizes its parser"
    }
  },
  "story:component-declares-its-settings": {
    "id": "story:component-declares-its-settings",
    "kind": "story",
    "version": "2",
    "status": "draft",
    "location": {
      "path": "story/component-declares-its-settings.md"
    },
    "relations": [
      {
        "decomposes": "epic:configuration-declared-once"
      }
    ],
    "metadata": {
      "title": "A component declares its settings, and the runtime slots are derived from them"
    }
  },
  "story:component-release-check-toolchains": {
    "id": "story:component-release-check-toolchains",
    "kind": "story",
    "version": "6",
    "status": "implemented",
    "location": {
      "path": "story/component-release-check-toolchains.md"
    },
    "relations": [
      {
        "decomposes": "epic:oci-component-delivery"
      }
    ],
    "metadata": {
      "title": "Provision component-owned release check toolchains",
      "summary": "Let an ESS component release select its Rust, Node, and pnpm check toolchains without forking the OCI release pipeline."
    }
  },
  "story:composite-action-shell": {
    "id": "story:composite-action-shell",
    "kind": "story",
    "version": "5",
    "status": "implemented",
    "location": {
      "path": "story/composite-action-shell.md"
    },
    "relations": [
      {
        "decomposes": "epic:oci-component-delivery"
      }
    ],
    "metadata": {
      "title": "Invoke the component release script portably",
      "summary": "Run the composite action script through its declared Bash interpreter on clean GitHub checkouts."
    }
  },
  "story:crates-under-area-directories": {
    "id": "story:crates-under-area-directories",
    "kind": "story",
    "version": "8",
    "status": "implemented",
    "location": {
      "path": "story/crates-under-area-directories.md"
    },
    "relations": [
      {
        "decomposes": "epic:area-layout"
      }
    ],
    "metadata": {
      "title": "Crates move under area directories; names and consumers unchanged",
      "summary": "git mv the 20 crates into crates/{specify,generate,verify,infra,edge}; fix member paths, workspace dependency paths and the seven Taskfile fixture paths; no crate renamed."
    }
  },
  "story:create-only-command-cannot-refuse": {
    "id": "story:create-only-command-cannot-refuse",
    "kind": "story",
    "version": "10",
    "status": "draft",
    "location": {
      "path": "story/create-only-command-cannot-refuse.md"
    },
    "metadata": {
      "title": "A command that only creates cannot declare a refusal",
      "summary": "wrong_state: is the only refusal form and ESS-COMMAND-012 refuses it where nothing moves, so the claim survives only in a hand-written scenario"
    }
  },
  "story:early-stop-assertion": {
    "id": "story:early-stop-assertion",
    "kind": "story",
    "version": "5",
    "status": "implemented",
    "location": {
      "path": "story/early-stop-assertion.md"
    },
    "metadata": {
      "title": "An authored scenario can claim that a consumer halted an ordered scan"
    }
  },
  "story:elapsed-time-claims": {
    "id": "story:elapsed-time-claims",
    "kind": "story",
    "version": "5",
    "status": "implemented",
    "location": {
      "path": "story/elapsed-time-claims.md"
    },
    "metadata": {
      "title": "An authored scenario can claim a length of time, and a target has to answer for it"
    }
  },
  "story:enum-variant-in-an-entity-invariant": {
    "id": "story:enum-variant-in-an-entity-invariant",
    "kind": "story",
    "version": "6",
    "status": "draft",
    "location": {
      "path": "story/enum-variant-in-an-entity-invariant.md"
    },
    "relations": [
      {
        "informed_by": "story:review-expression-typechecking"
      }
    ],
    "metadata": {
      "title": "An entity invariant may name an enum variant that does not exist, and validate accepts it"
    }
  },
  "story:fit-source-driven-change-summary": {
    "id": "story:fit-source-driven-change-summary",
    "kind": "story",
    "version": "7",
    "status": "implemented",
    "location": {
      "path": "story/fit-source-driven-change-summary.md"
    },
    "relations": [
      {
        "serves": "vision:O2"
      },
      {
        "informed_by": "release-plan:consolidated-ess-019"
      }
    ],
    "metadata": {
      "title": "Fit the source-driven change summary to the publication contract"
    }
  },
  "story:fuzz-the-specification-surface": {
    "id": "story:fuzz-the-specification-surface",
    "kind": "story",
    "version": "12",
    "status": "draft",
    "location": {
      "path": "story/fuzz-the-specification-surface.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      },
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Fuzz the specification surface",
      "summary": "Anything validate accepts, every projection and every synthesis target survives — asserted rather than hoped.",
      "owner": "ess"
    }
  },
  "story:go-normalization-pattern-semantics": {
    "id": "story:go-normalization-pattern-semantics",
    "kind": "story",
    "version": "15",
    "status": "active",
    "location": {
      "path": "story/go-normalization-pattern-semantics.md"
    },
    "relations": [
      {
        "derived_from": "story:source-pinned-data-normalization"
      },
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Qualify bounded ECMA-262 patterns in Go normalization"
    }
  },
  "story:helm-defaults-satisfy-schema": {
    "id": "story:helm-defaults-satisfy-schema",
    "kind": "story",
    "version": "6",
    "status": "implemented",
    "location": {
      "path": "story/helm-defaults-satisfy-schema.md"
    },
    "relations": [
      {
        "derived_from": "epic:oci-component-delivery"
      }
    ],
    "metadata": {
      "title": "Generate Helm defaults that satisfy their schema",
      "summary": "Ensure every configuration-neutral ESS chart passes Helm lint before environment binding."
    }
  },
  "story:helm-secret-slot-defaults": {
    "id": "story:helm-secret-slot-defaults",
    "kind": "story",
    "version": "5",
    "status": "implemented",
    "location": {
      "path": "story/helm-secret-slot-defaults.md"
    },
    "relations": [
      {
        "decomposes": "epic:oci-component-delivery"
      }
    ],
    "metadata": {
      "title": "Render valid Helm defaults for secret slots",
      "summary": "Emit every typed runtime secret slot in default values so generated charts render and lint."
    }
  },
  "story:integrate-source-driven-realizations": {
    "id": "story:integrate-source-driven-realizations",
    "kind": "story",
    "version": "3",
    "status": "draft",
    "location": {
      "path": "story/integrate-source-driven-realizations.md"
    },
    "relations": [
      {
        "depends_on": "story:review-format-catalog"
      },
      {
        "depends_on": "story:review-rust-target-feasibility"
      },
      {
        "depends_on": "story:types-only-realizations"
      },
      {
        "depends_on": "story:source-pinned-data-normalization"
      }
    ],
    "metadata": {
      "title": "Integrate source-driven realizations with the remediation baseline"
    }
  },
  "story:java-conformance-target": {
    "id": "story:java-conformance-target",
    "kind": "story",
    "version": "5",
    "status": "draft",
    "location": {
      "path": "story/java-conformance-target.md"
    },
    "metadata": {
      "title": "A conformance suite can be emitted as a Java test package",
      "summary": "ess verify conform synthesize --target offers ir and go; an adopter in Java reaches no runner"
    }
  },
  "story:model-binary64-fields": {
    "id": "story:model-binary64-fields",
    "kind": "story",
    "version": "14",
    "status": "implemented",
    "location": {
      "path": "story/model-binary64-fields.md"
    },
    "relations": [
      {
        "derived_from": "story:source-pinned-data-normalization"
      },
      {
        "serves": "vision:O2"
      },
      {
        "depends_on": "story:raw-json-normalization-provenance"
      }
    ],
    "metadata": {
      "title": "Represent finite binary64 fields in compiler-owned models"
    }
  },
  "story:native-realization-ci": {
    "id": "story:native-realization-ci",
    "kind": "story",
    "version": "2",
    "status": "draft",
    "location": {
      "path": "story/native-realization-ci.md"
    },
    "relations": [
      {
        "serves": "vision:O2"
      },
      {
        "informed_by": "story:types-only-realizations"
      }
    ],
    "metadata": {
      "title": "Run structural realization compiler checks in CI"
    }
  },
  "story:normalization-equality-eligibility": {
    "id": "story:normalization-equality-eligibility",
    "kind": "story",
    "version": "2",
    "status": "draft",
    "location": {
      "path": "story/normalization-equality-eligibility.md"
    },
    "relations": [
      {
        "derived_from": "story:typescript-normalization-target"
      },
      {
        "informed_by": "review-result:typescript-equality-binding"
      },
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Specify numeric equality eligibility and its format compatibility boundary"
    }
  },
  "story:normalization-followup-publication": {
    "id": "story:normalization-followup-publication",
    "kind": "story",
    "version": "5",
    "status": "implemented",
    "location": {
      "path": "story/normalization-followup-publication.md"
    },
    "relations": [
      {
        "depends_on": "story:binary64-structural-codecs"
      },
      {
        "depends_on": "story:normalize-positional-array-input"
      },
      {
        "derived_from": "story:source-pinned-data-normalization"
      },
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Publish shared Binary64 codec and positional normalization guidance"
    }
  },
  "story:normalize-model-owned-records": {
    "id": "story:normalize-model-owned-records",
    "kind": "story",
    "version": "10",
    "status": "implemented",
    "location": {
      "path": "story/normalize-model-owned-records.md"
    },
    "relations": [
      {
        "derived_from": "story:source-pinned-data-normalization"
      },
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Normalize model-owned records without duplicating their schemas"
    }
  },
  "story:normalize-positional-array-input": {
    "id": "story:normalize-positional-array-input",
    "kind": "story",
    "version": "17",
    "status": "implemented",
    "location": {
      "path": "story/normalize-positional-array-input.md"
    },
    "relations": [
      {
        "derived_from": "story:source-pinned-data-normalization"
      },
      {
        "serves": "vision:O2"
      },
      {
        "depends_on": "story:model-binary64-fields"
      }
    ],
    "metadata": {
      "title": "Normalize declared positional arrays without guessing decoder policy"
    }
  },
  "story:oci-component-release": {
    "id": "story:oci-component-release",
    "kind": "story",
    "version": "7",
    "status": "implemented",
    "location": {
      "path": "story/oci-component-release.md"
    },
    "relations": [
      {
        "decomposes": "epic:oci-component-delivery"
      }
    ],
    "metadata": {
      "title": "Execute and compose OCI component releases",
      "summary": "Make independently released component bundles the reusable ESS deployment unit."
    }
  },
  "story:own-planning-store": {
    "id": "story:own-planning-store",
    "kind": "story",
    "version": "4",
    "status": "implemented",
    "location": {
      "path": "story/own-planning-store.md"
    },
    "relations": [
      {
        "decomposes": "epic:entity-relations"
      }
    ],
    "metadata": {
      "title": "ESS plans in a store of its own",
      "summary": "The .engineering store, pinned to aep 0.42.0, so cross-repository work has a plan that is not a chat.",
      "owner": "ess",
      "tags": [
        "store"
      ]
    }
  },
  "story:provision-wasm-for-gate": {
    "id": "story:provision-wasm-for-gate",
    "kind": "story",
    "version": "5",
    "status": "implemented",
    "location": {
      "path": "story/provision-wasm-for-gate.md"
    },
    "relations": [
      {
        "serves": "vision:O2"
      },
      {
        "informed_by": "story:review-rust-target-feasibility"
      }
    ],
    "metadata": {
      "title": "Provision the mandatory WASM compiler target in CI"
    }
  },
  "story:raw-json-normalization-provenance": {
    "id": "story:raw-json-normalization-provenance",
    "kind": "story",
    "version": "12",
    "status": "implemented",
    "location": {
      "path": "story/raw-json-normalization-provenance.md"
    },
    "relations": [
      {
        "derived_from": "story:source-pinned-data-normalization"
      },
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Preserve declared raw JSON token bytes during normalization"
    }
  },
  "story:relations-design-page": {
    "id": "story:relations-design-page",
    "kind": "story",
    "version": "4",
    "status": "implemented",
    "location": {
      "path": "story/relations-design-page.md"
    },
    "relations": [
      {
        "decomposes": "epic:entity-relations"
      }
    ],
    "metadata": {
      "title": "The relation vocabulary is decided on a design page before code",
      "summary": "docs/design/ess-entity-relations-design-v0.1.md: shape, refusals, projections, rejected alternatives.",
      "owner": "ess",
      "tags": [
        "design",
        "relations"
      ]
    }
  },
  "story:relations-in-the-billing-example": {
    "id": "story:relations-in-the-billing-example",
    "kind": "story",
    "version": "4",
    "status": "implemented",
    "location": {
      "path": "story/relations-in-the-billing-example.md"
    },
    "relations": [
      {
        "decomposes": "epic:entity-relations"
      },
      {
        "depends_on": "story:relations-projected"
      }
    ],
    "metadata": {
      "title": "The billing example shows an ownership relation end to end",
      "summary": "examples/billing carries one owns relation, validated, compiled into the golden fixtures and explained in its README.",
      "owner": "ess",
      "tags": [
        "examples",
        "relations"
      ]
    }
  },
  "story:relations-in-the-domain-model": {
    "id": "story:relations-in-the-domain-model",
    "kind": "story",
    "version": "4",
    "status": "implemented",
    "location": {
      "path": "story/relations-in-the-domain-model.md"
    },
    "relations": [
      {
        "decomposes": "epic:entity-relations"
      },
      {
        "depends_on": "story:relations-design-page"
      }
    ],
    "metadata": {
      "title": "An entity's relations are typed, resolved and refused like its fields",
      "summary": "RelationSpec on EntitySpec, a validation pass beside validate_lifecycle_causes, four refusals each with a test that breaks it.",
      "owner": "ess",
      "tags": [
        "domain",
        "relations"
      ]
    }
  },
  "story:relations-projected": {
    "id": "story:relations-projected",
    "kind": "story",
    "version": "4",
    "status": "implemented",
    "location": {
      "path": "story/relations-projected.md"
    },
    "relations": [
      {
        "decomposes": "epic:entity-relations"
      },
      {
        "depends_on": "story:relations-in-the-domain-model"
      }
    ],
    "metadata": {
      "title": "A relation survives into every projection",
      "summary": "One extension key carries target and cardinality into JSON Schema, OpenAPI and Rust; golden tests per projection.",
      "owner": "ess",
      "tags": [
        "projections",
        "relations"
      ]
    }
  },
  "story:release-status-publication-state": {
    "id": "story:release-status-publication-state",
    "kind": "story",
    "version": "3",
    "status": "draft",
    "location": {
      "path": "story/release-status-publication-state.md"
    },
    "relations": [
      {
        "informed_by": "release-plan:consolidated-ess-019"
      }
    ],
    "metadata": {
      "title": "Release status distinguishes drafts from public releases"
    }
  },
  "story:reusable-component-release-action": {
    "id": "story:reusable-component-release-action",
    "kind": "story",
    "version": "5",
    "status": "implemented",
    "location": {
      "path": "story/reusable-component-release-action.md"
    },
    "relations": [
      {
        "decomposes": "epic:oci-component-delivery"
      }
    ],
    "metadata": {
      "title": "Publish any ESS component through one release action",
      "summary": "Provide an ESS-owned action for building or adopting an image and publishing chart, evidence, and bundle artifacts."
    }
  },
  "story:review-authored-discovery": {
    "id": "story:review-authored-discovery",
    "kind": "story",
    "version": "3",
    "status": "draft",
    "location": {
      "path": "story/review-authored-discovery.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      },
      {
        "serves": "vision:O2"
      },
      {
        "depends_on": "story:scenarios-directory-compiles-nothing"
      }
    ],
    "metadata": {
      "title": "Define predictable discovery for co-located ESS documents",
      "tags": [
        "P1",
        "review-2026-09-05"
      ]
    }
  },
  "story:review-browser-replay-fidelity": {
    "id": "story:review-browser-replay-fidelity",
    "kind": "story",
    "version": "4",
    "status": "draft",
    "location": {
      "path": "story/review-browser-replay-fidelity.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      },
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Make browser replay faithful to its declared semantic subset",
      "tags": [
        "P1",
        "review-2026-09-05"
      ]
    }
  },
  "story:review-cache-origin": {
    "id": "story:review-cache-origin",
    "kind": "story",
    "version": "10",
    "status": "draft",
    "location": {
      "path": "story/review-cache-origin.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      },
      {
        "serves": "vision:O2"
      },
      {
        "depends_on": "story:review-persisted-delivery-validation"
      }
    ],
    "metadata": {
      "title": "Verify cached bundle bytes against their OCI identity",
      "tags": [
        "P1",
        "review-2026-09-05"
      ]
    }
  },
  "story:review-composition-contract": {
    "id": "story:review-composition-contract",
    "kind": "story",
    "version": "10",
    "status": "implemented",
    "location": {
      "path": "story/review-composition-contract.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      },
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "State the composition client plan's actual guarantees",
      "tags": [
        "P2",
        "review-2026-09-05"
      ]
    }
  },
  "story:review-conformance-coverage": {
    "id": "story:review-conformance-coverage",
    "kind": "story",
    "version": "23",
    "status": "implemented",
    "location": {
      "path": "story/review-conformance-coverage.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      },
      {
        "serves": "vision:O2"
      },
      {
        "depends_on": "story:review-report-reader-validation"
      },
      {
        "depends_on": "story:a-skipped-scenario-is-not-a-failed-one"
      },
      {
        "depends_on": "story:review-conformance-format-design"
      }
    ],
    "metadata": {
      "title": "Carry conformance coverage through persisted suites and evidence",
      "tags": [
        "P0",
        "review-2026-09-05"
      ]
    }
  },
  "story:review-conformance-format-design": {
    "id": "story:review-conformance-format-design",
    "kind": "story",
    "version": "8",
    "status": "implemented",
    "location": {
      "path": "story/review-conformance-format-design.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      },
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Design the conformance suite and report migration"
    }
  },
  "story:review-consumer-coverage": {
    "id": "story:review-consumer-coverage",
    "kind": "story",
    "version": "8",
    "status": "draft",
    "location": {
      "path": "story/review-consumer-coverage.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      },
      {
        "serves": "vision:O2"
      },
      {
        "depends_on": "story:review-semantic-diff-coverage"
      }
    ],
    "metadata": {
      "title": "Require explicit consumer coverage for model extensions",
      "tags": [
        "P1",
        "review-2026-09-05"
      ]
    }
  },
  "story:review-delivery-trust-contract": {
    "id": "story:review-delivery-trust-contract",
    "kind": "story",
    "version": "6",
    "status": "draft",
    "location": {
      "path": "story/review-delivery-trust-contract.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      },
      {
        "serves": "vision:O2"
      },
      {
        "depends_on": "story:review-persisted-delivery-validation"
      },
      {
        "depends_on": "story:review-report-reader-validation"
      }
    ],
    "metadata": {
      "title": "Distinguish release consistency from verified evidence",
      "tags": [
        "P1",
        "review-2026-09-05"
      ]
    }
  },
  "story:review-execution-recovery-design": {
    "id": "story:review-execution-recovery-design",
    "kind": "story",
    "version": "8",
    "status": "implemented",
    "location": {
      "path": "story/review-execution-recovery-design.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      },
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Specify the recovery contract for finite deployment execution",
      "tags": [
        "P1",
        "review-2026-09-05"
      ]
    }
  },
  "story:review-expression-typechecking": {
    "id": "story:review-expression-typechecking",
    "kind": "story",
    "version": "16",
    "status": "implemented",
    "location": {
      "path": "story/review-expression-typechecking.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      },
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Resolve complete expression paths during validation",
      "tags": [
        "P1",
        "review-2026-09-05"
      ]
    }
  },
  "story:review-format-catalog": {
    "id": "story:review-format-catalog",
    "kind": "story",
    "version": "14",
    "status": "implemented",
    "location": {
      "path": "story/review-format-catalog.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      },
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Catalog format identities and canonical byte contracts",
      "tags": [
        "P1",
        "review-2026-09-05"
      ]
    }
  },
  "story:review-glossary-boundaries": {
    "id": "story:review-glossary-boundaries",
    "kind": "story",
    "version": "5",
    "status": "draft",
    "location": {
      "path": "story/review-glossary-boundaries.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      },
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Disambiguate ESS logical, interface and delivery concepts",
      "tags": [
        "P2",
        "review-2026-09-05"
      ]
    }
  },
  "story:review-infra-ir-invariants": {
    "id": "story:review-infra-ir-invariants",
    "kind": "story",
    "version": "16",
    "status": "implemented",
    "location": {
      "path": "story/review-infra-ir-invariants.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      },
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Keep resolved infrastructure handles valid for the IR lifetime",
      "tags": [
        "P0",
        "review-2026-09-05"
      ]
    }
  },
  "story:review-kubectl-diagnostic-sanitization": {
    "id": "story:review-kubectl-diagnostic-sanitization",
    "kind": "story",
    "version": "10",
    "status": "implemented",
    "location": {
      "path": "story/review-kubectl-diagnostic-sanitization.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      },
      {
        "serves": "vision:O2"
      },
      {
        "informed_by": "review-result:review-boundaries-1-secret-adversary-pass-1"
      }
    ],
    "metadata": {
      "title": "Keep untrusted kubectl stderr out of ESS diagnostics",
      "tags": [
        "P0",
        "review-2026-09-05"
      ]
    }
  },
  "story:review-observation-completeness": {
    "id": "story:review-observation-completeness",
    "kind": "story",
    "version": "18",
    "status": "draft",
    "location": {
      "path": "story/review-observation-completeness.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      },
      {
        "serves": "vision:O2"
      },
      {
        "depends_on": "story:review-secret-sanitization"
      },
      {
        "depends_on": "story:review-infra-ir-invariants"
      }
    ],
    "metadata": {
      "title": "Preserve observation scope and selector uncertainty",
      "tags": [
        "P1",
        "review-2026-09-05"
      ]
    }
  },
  "story:review-openapi-semantic-accounting": {
    "id": "story:review-openapi-semantic-accounting",
    "kind": "story",
    "version": "21",
    "status": "implemented",
    "location": {
      "path": "story/review-openapi-semantic-accounting.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      },
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Account for every unpreserved OpenAPI constraint",
      "tags": [
        "P1",
        "review-2026-09-05"
      ]
    }
  },
  "story:review-output-containment": {
    "id": "story:review-output-containment",
    "kind": "story",
    "version": "14",
    "status": "implemented",
    "location": {
      "path": "story/review-output-containment.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      },
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Validate output paths and page uniqueness before writing",
      "tags": [
        "P0",
        "review-2026-09-05"
      ]
    }
  },
  "story:review-output-ownership": {
    "id": "story:review-output-ownership",
    "kind": "story",
    "version": "3",
    "status": "draft",
    "location": {
      "path": "story/review-output-ownership.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      },
      {
        "serves": "vision:O2"
      },
      {
        "depends_on": "story:review-output-containment"
      }
    ],
    "metadata": {
      "title": "Make generated output replacement recoverable and ownership-aware",
      "tags": [
        "P1",
        "review-2026-09-05"
      ]
    }
  },
  "story:review-persisted-delivery-validation": {
    "id": "story:review-persisted-delivery-validation",
    "kind": "story",
    "version": "8",
    "status": "implemented",
    "location": {
      "path": "story/review-persisted-delivery-validation.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      },
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Restore delivery IR invariants at persisted read boundaries",
      "tags": [
        "P0",
        "review-2026-09-05"
      ]
    }
  },
  "story:review-primitive-semantics": {
    "id": "story:review-primitive-semantics",
    "kind": "story",
    "version": "6",
    "status": "draft",
    "location": {
      "path": "story/review-primitive-semantics.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      },
      {
        "serves": "vision:O2"
      },
      {
        "depends_on": "story:review-format-catalog"
      }
    ],
    "metadata": {
      "title": "Align primitive admission and exact numeric semantics",
      "tags": [
        "P1",
        "review-2026-09-05"
      ]
    }
  },
  "story:review-public-support-claims": {
    "id": "story:review-public-support-claims",
    "kind": "story",
    "version": "6",
    "status": "draft",
    "location": {
      "path": "story/review-public-support-claims.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      },
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Keep public support claims aligned with shipped evidence",
      "tags": [
        "P1",
        "review-2026-09-05"
      ]
    }
  },
  "story:review-report-reader-validation": {
    "id": "story:review-report-reader-validation",
    "kind": "story",
    "version": "7",
    "status": "implemented",
    "location": {
      "path": "story/review-report-reader-validation.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      },
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Validate standalone conformance report claims on read",
      "tags": [
        "P0",
        "review-2026-09-05"
      ]
    }
  },
  "story:review-rust-target-feasibility": {
    "id": "story:review-rust-target-feasibility",
    "kind": "story",
    "version": "13",
    "status": "implemented",
    "location": {
      "path": "story/review-rust-target-feasibility.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      },
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Check Rust target feasibility before claiming generated output",
      "tags": [
        "P1",
        "review-2026-09-05"
      ]
    }
  },
  "story:review-schema-resource-identity": {
    "id": "story:review-schema-resource-identity",
    "kind": "story",
    "version": "18",
    "status": "implemented",
    "location": {
      "path": "story/review-schema-resource-identity.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      },
      {
        "serves": "vision:O2"
      },
      {
        "depends_on": "story:review-format-catalog"
      }
    ],
    "metadata": {
      "title": "Define the boundary between generated schemas and registry resources",
      "tags": [
        "P2",
        "review-2026-09-05"
      ]
    }
  },
  "story:review-secret-sanitization": {
    "id": "story:review-secret-sanitization",
    "kind": "story",
    "version": "7",
    "status": "implemented",
    "location": {
      "path": "story/review-secret-sanitization.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      },
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Refuse malformed Secret shapes before serialization",
      "tags": [
        "P0",
        "review-2026-09-05"
      ]
    }
  },
  "story:review-semantic-diff-coverage": {
    "id": "story:review-semantic-diff-coverage",
    "kind": "story",
    "version": "26",
    "status": "implemented",
    "location": {
      "path": "story/review-semantic-diff-coverage.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      },
      {
        "serves": "vision:O2"
      },
      {
        "informed_by": "obligation:review-contract-rollout-coordination"
      }
    ],
    "metadata": {
      "title": "Propagate every reviewed semantic change into diff and impact",
      "tags": [
        "P0",
        "review-2026-09-05"
      ]
    }
  },
  "story:review-typed-diagnostics": {
    "id": "story:review-typed-diagnostics",
    "kind": "story",
    "version": "4",
    "status": "draft",
    "location": {
      "path": "story/review-typed-diagnostics.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      },
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Carry diagnostic identity independently of rendered wording",
      "tags": [
        "P2",
        "review-2026-09-05"
      ]
    }
  },
  "story:review-typescript-root-collision": {
    "id": "story:review-typescript-root-collision",
    "kind": "story",
    "version": "11",
    "status": "implemented",
    "location": {
      "path": "story/review-typescript-root-collision.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      },
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Allocate TypeScript roots and definitions in one symbol space",
      "tags": [
        "P1",
        "review-2026-09-05"
      ]
    }
  },
  "story:scenarios-directory-compiles-nothing": {
    "id": "story:scenarios-directory-compiles-nothing",
    "kind": "story",
    "version": "13",
    "status": "implemented",
    "location": {
      "path": "story/scenarios-directory-compiles-nothing.md"
    },
    "relations": [
      {
        "decomposes": "epic:review-boundary-remediation"
      },
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "A --scenarios directory of directories compiles nothing and exits 0",
      "summary": "The flag does not descend, and a corpus root silently yields a suite with none of the corpus"
    }
  },
  "story:schema-bundle-import": {
    "id": "story:schema-bundle-import",
    "kind": "story",
    "version": "5",
    "status": "implemented",
    "location": {
      "path": "story/schema-bundle-import.md"
    },
    "relations": [
      {
        "informed_by": "story:review-openapi-semantic-accounting"
      },
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Import structural component bundles with explicit dialect accounting"
    }
  },
  "story:schema-document-root-import": {
    "id": "story:schema-document-root-import",
    "kind": "story",
    "version": "5",
    "status": "implemented",
    "location": {
      "path": "story/schema-document-root-import.md"
    },
    "relations": [
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Retain the root of an imported JSON Schema document"
    }
  },
  "story:schema-unique-items-signed-zero": {
    "id": "story:schema-unique-items-signed-zero",
    "kind": "story",
    "version": "3",
    "status": "draft",
    "location": {
      "path": "story/schema-unique-items-signed-zero.md"
    },
    "relations": [
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Correct size-dependent schema uniqueness for signed zero"
    }
  },
  "story:source-pinned-data-normalization": {
    "id": "story:source-pinned-data-normalization",
    "kind": "story",
    "version": "15",
    "status": "active",
    "location": {
      "path": "story/source-pinned-data-normalization.md"
    },
    "relations": [
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Source-pinned checked data normalization across Go Rust and TypeScript"
    }
  },
  "story:the-generated-go-runtime-is-gofmt-clean": {
    "id": "story:the-generated-go-runtime-is-gofmt-clean",
    "kind": "story",
    "version": "6",
    "status": "draft",
    "location": {
      "path": "story/the-generated-go-runtime-is-gofmt-clean.md"
    },
    "metadata": {
      "title": "The emitted Go runtime is not gofmt-stable, so an adopter's formatter changes it"
    }
  },
  "story:types-only-realizations": {
    "id": "story:types-only-realizations",
    "kind": "story",
    "version": "5",
    "status": "active",
    "location": {
      "path": "story/types-only-realizations.md"
    },
    "relations": [
      {
        "informed_by": "specification:existing-struct-generation"
      },
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Consistent types-only realizations for Go Rust and TypeScript"
    }
  },
  "story:typescript-normalization-target": {
    "id": "story:typescript-normalization-target",
    "kind": "story",
    "version": "38",
    "status": "implemented",
    "location": {
      "path": "story/typescript-normalization-target.md"
    },
    "relations": [
      {
        "derived_from": "story:source-pinned-data-normalization"
      },
      {
        "serves": "vision:O2"
      },
      {
        "depends_on": "story:normalize-positional-array-input"
      }
    ],
    "metadata": {
      "title": "Execute source-pinned normalization in native TypeScript"
    }
  },
  "story:unique-wire-field-identity": {
    "id": "story:unique-wire-field-identity",
    "kind": "story",
    "version": "4",
    "status": "implemented",
    "location": {
      "path": "story/unique-wire-field-identity.md"
    },
    "relations": [
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Reject colliding wire field names before any projection"
    }
  },
  "task:normalization-followup-publication": {
    "id": "task:normalization-followup-publication",
    "kind": "task",
    "version": "3",
    "status": "archived",
    "location": {
      "path": "task/normalization-followup-publication.md"
    },
    "relations": [
      {
        "derived_from": "story:binary64-structural-codecs"
      },
      {
        "derived_from": "story:normalize-positional-array-input"
      },
      {
        "serves": "vision:O2"
      }
    ],
    "metadata": {
      "title": "Publish shared Binary64 codec and positional normalization guidance"
    }
  },
  "verification-report:composition-cli-example-first-check": {
    "id": "verification-report:composition-cli-example-first-check",
    "kind": "verification-report",
    "version": "1",
    "status": "draft",
    "location": {
      "path": "verification-report/composition-cli-example-first-check.md"
    },
    "relations": [
      {
        "verifies": "story:review-composition-contract"
      }
    ],
    "metadata": {
      "title": "First execution of the composition CLI example"
    }
  },
  "verification-report:consumer-coverage-binding-wording": {
    "id": "verification-report:consumer-coverage-binding-wording",
    "kind": "verification-report",
    "version": "1",
    "status": "draft",
    "location": {
      "path": "verification-report/consumer-coverage-binding-wording.md"
    },
    "relations": [
      {
        "verifies": "story:review-consumer-coverage"
      }
    ],
    "metadata": {
      "title": "Consumer coverage binding Draft7 wording correction"
    }
  },
  "verification-report:coverage-cli-leaf-adaptation": {
    "id": "verification-report:coverage-cli-leaf-adaptation",
    "kind": "verification-report",
    "version": "1",
    "status": "draft",
    "location": {
      "path": "verification-report/coverage-cli-leaf-adaptation.md"
    },
    "relations": [
      {
        "verifies": "story:review-conformance-coverage"
      }
    ],
    "metadata": {
      "title": "Coverage CLI leaf inventory adaptation"
    }
  },
  "verification-report:coverage-go-execution-adaptation": {
    "id": "verification-report:coverage-go-execution-adaptation",
    "kind": "verification-report",
    "version": "1",
    "status": "draft",
    "location": {
      "path": "verification-report/coverage-go-execution-adaptation.md"
    },
    "relations": [
      {
        "verifies": "story:review-conformance-coverage"
      }
    ],
    "metadata": {
      "title": "Coverage Go execution adapter and future-version decisions"
    }
  },
  "verification-report:coverage-refusal-rendering-decision": {
    "id": "verification-report:coverage-refusal-rendering-decision",
    "kind": "verification-report",
    "version": "1",
    "status": "draft",
    "location": {
      "path": "verification-report/coverage-refusal-rendering-decision.md"
    },
    "relations": [
      {
        "verifies": "story:review-conformance-coverage"
      }
    ],
    "metadata": {
      "title": "Coverage refusal rendering D1 decision"
    }
  },
  "verification-report:coverage-writer-actual-correspondence": {
    "id": "verification-report:coverage-writer-actual-correspondence",
    "kind": "verification-report",
    "version": "1",
    "status": "draft",
    "location": {
      "path": "verification-report/coverage-writer-actual-correspondence.md"
    },
    "relations": [
      {
        "verifies": "story:review-conformance-coverage"
      }
    ],
    "metadata": {
      "title": "Coverage writer: integrated gate and 43 actual AEP correspondence cases"
    }
  },
  "verification-report:coverage-writer-correction-and-gate-preparation": {
    "id": "verification-report:coverage-writer-correction-and-gate-preparation",
    "kind": "verification-report",
    "version": "1",
    "status": "draft",
    "location": {
      "path": "verification-report/coverage-writer-correction-and-gate-preparation.md"
    },
    "relations": [
      {
        "verifies": "story:review-conformance-coverage"
      }
    ],
    "metadata": {
      "title": "Coverage writer correction and root gate preparation"
    }
  },
  "verification-report:coverage-writer-final-source-disposition": {
    "id": "verification-report:coverage-writer-final-source-disposition",
    "kind": "verification-report",
    "version": "1",
    "status": "draft",
    "location": {
      "path": "verification-report/coverage-writer-final-source-disposition.md"
    },
    "relations": [
      {
        "verifies": "story:review-conformance-coverage"
      }
    ],
    "metadata": {
      "title": "Coverage writer final source disposition: 648 package cases pass"
    }
  },
  "verification-report:coverage-writer-final-view-test-scope": {
    "id": "verification-report:coverage-writer-final-view-test-scope",
    "kind": "verification-report",
    "version": "1",
    "status": "draft",
    "location": {
      "path": "verification-report/coverage-writer-final-view-test-scope.md"
    },
    "relations": [
      {
        "verifies": "story:review-conformance-coverage"
      }
    ],
    "metadata": {
      "title": "Coverage writer final test scope: full admission and current unused view projection"
    }
  },
  "verification-report:fuzz-seed-baseline-go-panic": {
    "id": "verification-report:fuzz-seed-baseline-go-panic",
    "kind": "verification-report",
    "version": "1",
    "status": "draft",
    "location": {
      "path": "verification-report/fuzz-seed-baseline-go-panic.md"
    },
    "relations": [
      {
        "verifies": "story:fuzz-the-specification-surface"
      }
    ],
    "metadata": {
      "title": "Fuzz candidate seed baseline finds Go synthesis panic"
    }
  },
  "verification-report:fuzz-seed-baseline-refusal-stdout": {
    "id": "verification-report:fuzz-seed-baseline-refusal-stdout",
    "kind": "verification-report",
    "version": "1",
    "status": "draft",
    "location": {
      "path": "verification-report/fuzz-seed-baseline-refusal-stdout.md"
    },
    "relations": [
      {
        "verifies": "story:fuzz-the-specification-surface"
      }
    ],
    "metadata": {
      "title": "Read back the actual target-refusal stdout from the fuzz seed baseline"
    }
  },
  "verification-report:normalization-binary64-integrated": {
    "id": "verification-report:normalization-binary64-integrated",
    "kind": "verification-report",
    "version": "2",
    "status": "draft",
    "location": {
      "path": "verification-report/normalization-binary64-integrated.md"
    },
    "relations": [
      {
        "verifies": "story:model-binary64-fields"
      }
    ],
    "metadata": {
      "title": "Binary64 model and normalization integrated verification"
    }
  },
  "verification-report:normalization-main-reconciliation": {
    "id": "verification-report:normalization-main-reconciliation",
    "kind": "verification-report",
    "version": "1",
    "status": "draft",
    "location": {
      "path": "verification-report/normalization-main-reconciliation.md"
    },
    "relations": [
      {
        "verifies": "story:typescript-normalization-target"
      }
    ],
    "metadata": {
      "title": "Preserve published composition and normalization planning history"
    }
  },
  "verification-report:normalization-native-positional-integrated": {
    "id": "verification-report:normalization-native-positional-integrated",
    "kind": "verification-report",
    "version": "3",
    "status": "draft",
    "location": {
      "path": "verification-report/normalization-native-positional-integrated.md"
    },
    "relations": [
      {
        "verifies": "story:binary64-structural-codecs"
      },
      {
        "verifies": "story:normalize-positional-array-input"
      },
      {
        "verifies": "story:normalization-followup-publication"
      }
    ],
    "metadata": {
      "title": "Integrated native codecs and positional normalization verification"
    }
  },
  "verification-report:normalization-raw-json-integrated": {
    "id": "verification-report:normalization-raw-json-integrated",
    "kind": "verification-report",
    "version": "4",
    "status": "draft",
    "location": {
      "path": "verification-report/normalization-raw-json-integrated.md"
    },
    "relations": [
      {
        "verifies": "story:raw-json-normalization-provenance"
      }
    ],
    "metadata": {
      "title": "Raw JSON normalization integrated verification"
    }
  },
  "verification-report:normalization-recovery-20260906": {
    "id": "verification-report:normalization-recovery-20260906",
    "kind": "verification-report",
    "version": "3",
    "status": "draft",
    "location": {
      "path": "verification-report/normalization-recovery-20260906.md"
    },
    "relations": [
      {
        "verifies": "story:go-normalization-pattern-semantics"
      }
    ],
    "metadata": {
      "title": "Recover interrupted normalization qualification"
    }
  },
  "verification-report:review-boundaries-1-integrated": {
    "id": "verification-report:review-boundaries-1-integrated",
    "kind": "verification-report",
    "version": "1",
    "status": "draft",
    "location": {
      "path": "verification-report/review-boundaries-1-integrated.md"
    },
    "relations": [
      {
        "verifies": "story:review-secret-sanitization"
      },
      {
        "verifies": "story:review-report-reader-validation"
      }
    ],
    "metadata": {
      "title": "Boundary wave 1 integrated verification"
    }
  },
  "verification-report:review-boundaries-10-integrated": {
    "id": "verification-report:review-boundaries-10-integrated",
    "kind": "verification-report",
    "version": "1",
    "status": "draft",
    "location": {
      "path": "verification-report/review-boundaries-10-integrated.md"
    },
    "relations": [
      {
        "verifies": "story:review-schema-resource-identity"
      }
    ],
    "metadata": {
      "title": "Schema resource identity integrated verification"
    }
  },
  "verification-report:review-boundaries-10-versioned-source": {
    "id": "verification-report:review-boundaries-10-versioned-source",
    "kind": "verification-report",
    "version": "1",
    "status": "draft",
    "location": {
      "path": "verification-report/review-boundaries-10-versioned-source.md"
    },
    "relations": [
      {
        "verifies": "story:review-schema-resource-identity"
      }
    ],
    "metadata": {
      "title": "Schema workflow final versioned source gate"
    }
  },
  "verification-report:review-boundaries-11-producer-expectations": {
    "id": "verification-report:review-boundaries-11-producer-expectations",
    "kind": "verification-report",
    "version": "1",
    "status": "draft",
    "location": {
      "path": "verification-report/review-boundaries-11-producer-expectations.md"
    },
    "relations": [
      {
        "verifies": "story:review-conformance-coverage"
      }
    ],
    "metadata": {
      "title": "Independent coverage producer expectations and empty-policy baseline"
    }
  },
  "verification-report:review-boundaries-2-integrated": {
    "id": "verification-report:review-boundaries-2-integrated",
    "kind": "verification-report",
    "version": "2",
    "status": "draft",
    "location": {
      "path": "verification-report/review-boundaries-2-integrated.md"
    },
    "relations": [
      {
        "verifies": "story:review-kubectl-diagnostic-sanitization"
      },
      {
        "verifies": "story:review-output-containment"
      }
    ],
    "metadata": {
      "title": "Boundary wave 2 integrated verification"
    }
  },
  "verification-report:review-boundaries-3-integrated": {
    "id": "verification-report:review-boundaries-3-integrated",
    "kind": "verification-report",
    "version": "1",
    "status": "draft",
    "location": {
      "path": "verification-report/review-boundaries-3-integrated.md"
    },
    "relations": [
      {
        "verifies": "story:review-persisted-delivery-validation"
      },
      {
        "verifies": "story:review-typescript-root-collision"
      },
      {
        "verifies": "story:review-conformance-format-design"
      }
    ],
    "metadata": {
      "title": "Boundary wave 3 integrated verification"
    }
  },
  "verification-report:review-boundaries-4-integrated": {
    "id": "verification-report:review-boundaries-4-integrated",
    "kind": "verification-report",
    "version": "2",
    "status": "draft",
    "location": {
      "path": "verification-report/review-boundaries-4-integrated.md"
    },
    "relations": [
      {
        "verifies": "story:review-semantic-diff-coverage"
      },
      {
        "verifies": "story:review-infra-ir-invariants"
      }
    ],
    "metadata": {
      "title": "Boundary wave 4 integrated verification"
    }
  },
  "verification-report:review-boundaries-7-integrated": {
    "id": "verification-report:review-boundaries-7-integrated",
    "kind": "verification-report",
    "version": "2",
    "status": "draft",
    "location": {
      "path": "verification-report/review-boundaries-7-integrated.md"
    },
    "relations": [
      {
        "verifies": "story:review-expression-typechecking"
      },
      {
        "verifies": "story:review-openapi-semantic-accounting"
      }
    ],
    "metadata": {
      "title": "Boundary wave 7 integrated verification"
    }
  },
  "verification-report:review-boundaries-8-integrated": {
    "id": "verification-report:review-boundaries-8-integrated",
    "kind": "verification-report",
    "version": "1",
    "status": "draft",
    "location": {
      "path": "verification-report/review-boundaries-8-integrated.md"
    },
    "relations": [
      {
        "verifies": "story:a-skipped-scenario-is-not-a-failed-one"
      }
    ],
    "metadata": {
      "title": "Count writer integrated verification"
    }
  },
  "verification-report:review-boundaries-9-integrated": {
    "id": "verification-report:review-boundaries-9-integrated",
    "kind": "verification-report",
    "version": "1",
    "status": "draft",
    "location": {
      "path": "verification-report/review-boundaries-9-integrated.md"
    },
    "relations": [
      {
        "verifies": "story:review-composition-contract"
      }
    ],
    "metadata": {
      "title": "Composition contract integrated verification"
    }
  },
  "verification-report:typescript-normalization-integrated": {
    "id": "verification-report:typescript-normalization-integrated",
    "kind": "verification-report",
    "version": "2",
    "status": "draft",
    "location": {
      "path": "verification-report/typescript-normalization-integrated.md"
    },
    "metadata": {
      "title": "Qualified and published TypeScript normalization integration"
    }
  },
  "vision:O2": {
    "id": "vision:O2",
    "kind": "vision",
    "version": "1",
    "status": "draft",
    "location": {
      "path": "vision/O2.md"
    },
    "metadata": {
      "title": "O2 — Decisions as data, with evidence"
    }
  }
}
```

## graph stderr

```text
```

## blocked stdout

```text
nothing is blocked
```

## blocked stderr

```text
```

## waves stdout

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
          "id": "story:review-cache-origin",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-cache-origin.md"
            },
            {
              "confidence": "cited",
              "path": "website/docs/concepts/component-delivery.md"
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
      "wave": 5,
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
      "wave": 6,
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
      "b": "story:review-cache-origin",
      "path": "crates/edge/ess-cli",
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

## waves stderr

```text
```

## lifecycle stdout

```text
story starts at draft
  active -> implemented, archived
  archived -> nothing
  draft -> proposed, archived
  implemented -> archived
  proposed -> draft, rejected, active
  rejected -> archived
```

## lifecycle stderr

```text
```

## validate stdout

```text
185 file(s) in /home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/.engineering/planning: 185 artifact(s)
31 review(s) recorded no findings block:
  - review-result:binary64-structural-codecs-adversary-pass1 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:cache-origin-binding-pass1 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:composition-contract-adversary-pass-1 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:composition-contract-adversary-pass-2 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:coverage-writer-source-pass2 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:fuzz-specification-binding-pass2 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
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

## validate stderr

```text
```

## After applying candidate scopes

The exact wave computation is unchanged: six waves, 55 collisions, no unassessed artifacts or cycles. The original output above is byte-identical to the fresh result after applying the cache and fuzz Scope sections. Scope mutations used AEP; machine reservations remain unchanged.

```text
185 file(s) in /home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/.engineering/planning: 185 artifact(s)
31 review(s) recorded no findings block:
  - review-result:binary64-structural-codecs-adversary-pass1 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:cache-origin-binding-pass1 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:composition-contract-adversary-pass-1 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:composition-contract-adversary-pass-2 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:coverage-writer-source-pass2 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:fuzz-specification-binding-pass2 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
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
