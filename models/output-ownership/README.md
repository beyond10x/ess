# Generated output ownership

This model gives the enrolled anchor and its current transaction typed identities, an explicit
ownership relation, and the durable decision transitions selected by the
[output ownership design](../../docs/design/review-output-ownership.md).

The anchor owns at most one current transaction. The transaction has no meaning outside that
anchor; its `anchor_id` carries the relation. The anchor remains enrolled after final-file
retirement. Committed and Restored decisions survive cleanup; only the subsequent settled anchor
checkpoint omits the transaction. Cleanup and filesystem effects are specified by the design,
not inferred from a model transition.

This is the entity and decision model, not the complete JSON admission schema or a filesystem
proof. The runtime reader must also enforce canonical native-byte encoding, digest syntax,
closed checkpoint variants, complete file inventories, path ownership, preimage integrity and
synchronization ordering. Model validation alone does not establish those properties or execute
any I/O failure or process-interruption witness.
