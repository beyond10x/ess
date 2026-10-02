# Multiple independent observation windows

The shared Service executes this admitted suite with WindowHealthy and DelayedEvent modes. At the
same command, the two declared events require immediate windows and positive 350ms/650ms windows.
Native Runner reads the operation start once, shared by all its event windows. Each positive window
must additionally call the target's mark/elapsed capability and scan after measured completion.
The pinned actual native callback trace therefore discriminates restarting each window's runner
clock from sharing the operation start. Both deadlines exceed the 100ms deterministic clock tick.
The healthy log stays available and clean throughout; the delayed-log mutant leaks on its third
observation and fails. Counts, code, trace and plaintext-free evidence are checked against actual
stateful callbacks. The original twenty live controls remain unchanged.
