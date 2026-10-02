# One-time disclosure payload resource vectors

These nine additional controls reuse the exact `one-time-execution/view.json` admitted suite and
live command service. `support_one_time/resources.rs` supplies deterministic query rows; the runner
must actually issue the command and query. The JSON manifest pins actual native counts, callback
trace and compact encoded payload size. This does not replace the nineteen original controls.

Bound each response object and declared-error fields object separately. Bound each query's complete
rows array and each direct-event or independently observed event payload array as an aggregate.
Protocol identifiers/correlation are outside these payload surfaces. Check resources before looking
for plaintext; exceeding a bound yields Unsupported with a value-free ESS-CF-TARGET diagnostic.

Limits are inclusive: 1,048,576 bytes of canonical compact JSON UTF-8, 65,536 members summed over
all object entries and array elements, and value depth 128 with the root at zero. Keys count once as
part of their object entry, not as another member or depth level. Empty containers have zero members.
Traversal establishes member/depth bounds before a bounded serializer counts bytes. Quotes,
separators, escapes and number spellings all count. The canonical serializer is the existing native
Node/Number JSON writer; lossless Integer values use its exact integer spelling. This fixture's
numeric value is 1234567890123456789 (nineteen digits), never rounded through a binary64 adapter.

The text payload is `[{"data":"..."}]` with thirteen bytes of framing. The plain and escaped pairs
encode to exactly 1,048,576 and 1,048,577 bytes. Escaped text repeats U+0000 (six JSON bytes each),
then enough ASCII `x` to reach the boundary. The numeric overflow has 65,534 numbers inside `data`,
so the outer array, object entry and elements total exactly 65,536 members, but bytes overflow.
The member pair contains 65,536 or 65,537 empty objects. The depth pair wraps null in 126 or 127
singleton arrays below the outer rows array and row object, placing null at depth 128 or 129.

Expected per-case counts are native live execution evidence, not fixture replay evidence. The
original nine test bytes were red 4 pass / 5 fail before the correction and green 9 / 0 afterward;
the additional manifest reproduction test verifies all nine actual count reports and encoded sizes.

Four additive numeric controls put `number` and `padding` in one row. `number` is native Number
1.0 or 0.125, whose canonical JSON spelling is respectively `1.0` and `0.125`; the adapter must not
silently count the integral binary64 value as `1`. ASCII padding makes each pair exactly 1,048,576
or 1,048,577 bytes. These controls bring the resource manifest to thirteen cases (fourteen tests
including manifest reproduction), leaving the original nine entries unchanged.
