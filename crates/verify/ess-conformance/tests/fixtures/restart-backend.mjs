// The backend process of `restart-target.mjs`: one JSON request per input line, one JSON answer per
// output line, over the state directory `ESS_RESTART_DIR` names. `ESS_RESTART_BACKEND` is the mode
// `restart-target.mjs` documents. `TestRestartBackend` in `restart_target.go` is the same backend in
// Go.

import { readFileSync, renameSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { createInterface } from 'node:readline';

const mode = process.env.ESS_RESTART_BACKEND ?? '';
const path = join(process.env.ESS_RESTART_DIR ?? '.', 'state.json');

let state = { next: 0, rows: [] };
if (mode !== 'volatile') {
  try {
    state = JSON.parse(readFileSync(path, 'utf8'));
  } catch (error) {
    if (error.code !== 'ENOENT') throw error;
  }
}
if (mode === 'counter-reset') state.next = 0;

const send = (value) => process.stdout.write(`${JSON.stringify(value)}\n`);
send({ ready: process.pid });

for await (const line of createInterface({ input: process.stdin })) {
  const request = JSON.parse(line);
  if (request.op === 'record') {
    state.next += 1;
    const id = `00000000-0000-4000-8000-${String(state.next).padStart(12, '0')}`;
    const row = { entry_id: id, amount: request.amount };
    const index = state.rows.findIndex((existing) => existing.entry_id === id);
    if (index >= 0) state.rows[index] = row;
    else state.rows.push(row);
    if (mode !== 'volatile') {
      writeFileSync(`${path}.next`, JSON.stringify(state));
      renameSync(`${path}.next`, path);
    }
    send({ entry_id: id });
  } else if (request.op === 'list') {
    send({ rows: state.rows });
  } else {
    throw new Error(`unknown request ${request.op}`);
  }
}
