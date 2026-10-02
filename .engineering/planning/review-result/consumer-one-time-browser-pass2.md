---
format: aep.planning-md/3
id: review-result:consumer-one-time-browser-pass2
kind: review-result
status: active
title: Independent browser policy projection review, corrected source
relations:
- reviews: story:feature-request-389
revision: 1
---
approve

Independent source-only rereview, own test/build executions0. Exact corrected frozen patch one-time-browser-review-2.patch SHA256 fd1705c15921aac6400fc94eae2e6f9882f0d1f3da95db950fd16e7f5306d434 and new one_time_browser.rs SHA256 d73390733b727735ca1cae9ce4893b22f20efc0012fe9b03bca57070b3740bca verified in ess-backlog-next-20261002. Runtime verification of this corrected snapshot remains coordinator-owned and pending at review time.

Both first-pass findings are resolved in source. web.rs outcome projection conditionally retains nonempty one_time_response; player.js derives the complete source obligation catalog from model.commands independently of selected scenarios. The shared DOM renders exact command/outcome/fields and explicitly states that it observes no values and verifies no non-disclosure guarantee. An empty suite cannot erase source obligations. Unmarked model serialization retains its prior keys and values.

The named UnsupportedVocabulary refusal now lives in public AdmittedReplay::new immediately after model admission, before reduced model construction. Both direct construction and web::emit_input use that boundary. Coverage replay's player exposes an empty source-policy list under the unchanged shared template; it does not claim private policy execution or expand replay/1. Browser command catalog still refuses marked dispatch while retaining location and obligations.

The added actual-browser empty-suite test checks both persisted model policy and rendered source obligations with zero declared steps/origins. The expanded coverage test calls both public entry points. Together these are appropriate regression seams for the observed omissions. No additional concrete defect found in this projection-only scope. This approval is not a claim of conformance34/35 execution, full runtime parity, or completed validation of the corrected snapshot.

```findings
[]
```
