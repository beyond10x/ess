---
format: aep.planning-md/3
id: approval-record:ess-d338-findings-triage
kind: approval-record
status: approved
title: The 2026-10-31 findings triage is decided (D338)
summary: Adopted rows pickable; rows 23, 24, 26, 28, 29, 30 declined and row 16 a duplicate, archived; the wave lead decides out-of-scope rows from now on
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-10-31T23:45:20Z", actor: "human:timo", revision: 2}
---
## Decision

The 2026-10-31 findings triage of draft and proposed stories filed from adversary and security
passes is decided (orchestrator decision D338; the operator's approval relayed by host-monitor D53,
2026-10-31 23:4xZ: stories may be created for anything inside the organization).

- Adopted, pickable for the next wave: the ess rows of the sheet marked adopt. Row 4
  (story:an-authored-text-literal-outside-its-alphabet-is-refused) is narrowed to the
  entity-invariant half; row 9 (story:a-rust-runner-compares-a-decimal-by-exact-value) stays
  blocked on decision-blocker:decimal-canonical-serialization-crosses-repositories.
- Declined with the sheet's reason and archived (draft stories cannot be rejected; archived is
  their closing status): rows 23, 24, 26, 28, 29 and 30.
- Duplicate, archived: row 16, story:a-tied-input-also-read-by-a-row-set-guard-keeps-its-tie, of
  story:a-caller-swap-redraw-keeps-unread-members-and-related-ties.

Each closed story carries a `## Triage (D338)` section with its reason.

## Standing rule from this decision

From wave D335 on, the wave lead decides each out-of-scope review row at wave close (adopt as a
story, decline with a reason, or duplicate) and reports the counts to the orchestrator; no triage
sheet goes to the operator.
