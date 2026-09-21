# Observed subject history and silent preservation

An internal calling application's AnswerCall consults the session's successful-answer record, not the Call's lifecycle.
A backend Bridged update and a successful client answer can leave equal lifecycle states with
different answer histories. Only a successful ClientAnswered call establishes Answered.

The ess/6 source addition is bounded: `when_subject: {field: answer_history, equals: Answered}`
reads a declared enum field of an existing subject, independently of an optional `when` input
predicate. An ordinary default handles the remaining known enum values. Unknown or unobservable
facts cannot select the default. Existing lifecycle and input guards retain their meaning.

`preserves: Entity`, with an input identity, declares a silent accepted branch. It permits no
assignments, emitted events or error. Synthesis must observe identity, lifecycle and relevant
fields before and after; an explicit no-error assertion is required in addition to the existing
no-event assertions. This new assertion needs a new conformance format; legacy ExpectOutcome must
not silently change meaning.

Arrangement search keeps lifecycle and the bounded subject enum facts in its visited key. A
same-state answer can change history, so update steps and same-state transitions cannot disappear
from the route. Each selected fact must be checked through an immediate, unfiltered declared view.
The implementation's observed value, never the expected outcome or injected label, is authority.

Preservation must also handle fields whose initial value the model does not determine (timestamps,
for example). Source `sets` alone cannot prove it. Suite/10 adds a subject snapshot before the
command and a same-subject comparison afterwards, alongside `expect_no_error`; suite/11 is its
coverage-bearing counterpart. Snapshot steps select exactly one row by the declared identity.
The chosen immediate views must collectively expose every declared subject field. A filtered
view is usable only when its predicate is determined true for the arranged subject; unknown
filter facts refuse that observation.
If that observation is unavailable, synthesis/execution refuses instead of claiming preservation.
Legacy outcome and view assertions keep their existing meanings.

The internal calling application exposes this session-owned fact through its existing opt-in conformance observer.
It is not a public protocol field and adds no business behavior. Remote endpoints that cannot read
that observer remain explicitly unsupported for the new history checks.

Compatibility: ess/1 through ess/5 reject both additions. Unchanged legacy inputs keep canonical
bytes. A conforming runtime passes the two histories; mutations that suppress the first answer,
repeat the media answer, emit an error/event, or change a preserved field must fail.
