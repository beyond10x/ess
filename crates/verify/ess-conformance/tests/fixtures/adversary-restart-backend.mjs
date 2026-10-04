// The backend process of `adversary-restart-target.mjs`: `restart-backend.mjs`, plus a count of the
// processes that have started over the state directory, so `counter-reset-second` can lose its
// counter on the third start only.

import { readFileSync, renameSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { createInterface } from 'node:readline';

const mode = process.env.ESS_RESTART_BACKEND ?? '';
const path = join(process.env.ESS_RESTART_DIR ?? '.', 'state.json');

let state = { next: 0, rows: [], starts: 0 };
try {
  state = JSON.parse(readFileSync(path, 'utf8'));
} catch (error) {
  if (error.code !== 'ENOENT') throw error;
}
state.starts += 1;
if (mode === 'counter-reset') state.next = 0;
if (mode === 'counter-reset-second' && state.starts === 3) state.next = 0;
const persist = () => {
  writeFileSync(`${path}.next`, JSON.stringify(state));
  renameSync(`${path}.next`, path);
};
persist();

const send = (value) => process.stdout.write(`${JSON.stringify(value)}\n`);
send({ ready: process.pid });

for await (const line of createInterface({ input: process.stdin })) {
  const request = JSON.parse(line);
  if (request.op === 'record') {
    if (mode === 'constant') state.next = 0;
    state.next += 1;
    const id = `00000000-0000-4000-8000-${String(state.next).padStart(12, '0')}`;
    const row = { entry_id: id, amount: request.amount };
    const index = state.rows.findIndex((existing) => existing.entry_id === id);
    if (index >= 0) state.rows[index] = row;
    else state.rows.push(row);
    persist();
    send({ entry_id: id });
  } else if (request.op === 'list') {
    send({ rows: state.rows });
  } else {
    throw new Error(`unknown request ${request.op}`);
  }
}
