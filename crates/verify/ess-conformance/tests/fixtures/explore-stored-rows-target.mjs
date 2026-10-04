// An in-memory implementation of `explore-stored-rows.yaml` (beyond10x/ess#221, #223), deciding
// each command as the specification does.
//
// `mode` selects the double:
//
//   (empty)            the specification
//   existing-ignored   `Bind` over a key that exists overwrites it rather than answering
//                      `already-bound`
//   state-ignored      `Revoke` of a revoked key answers `revoked` again
//   changes-ignored    `Resume` of an active key answers `resumed`, not `restated`
//   field-ignored      `Upgrade` of a `Pro` key answers `upgraded`
//   predicate-ignored  `Use` never answers `exhausted`
//   count-inclusive    `Bind` refuses a secret of exactly 12 characters
//   uses-seven         `Bind` refuses `uses: 7`, the input's `example:`
//   label-plain        `Bind` throws on a label other than "", "a" or "b"
//   bulk               `Use` of more than 100 answers `bulk` unless the key is exhausted, for the
//                      fixture variant that declares a `bulk` branch
//
// `explore_stored_rows_target.go` is the same target in Go, line for line.

import { unsupported } from './dist/index.js';

export function newTarget(mode = '') {
  let keys = new Map();
  let owners = [];
  let version = 0;

  const reset = () => {
    keys = new Map();
    owners = [];
    version = 0;
  };
  const refused = (outcome, error) => ({ outcome, error, directEvents: [] });
  const answered = (outcome, event, field, id) => {
    version += 1;
    return {
      outcome,
      consistency: `v${version}`,
      directEvents: [{ event, payload: { [field]: id } }],
    };
  };
  const existing = (id) => {
    const key = keys.get(id);
    if (key === undefined) throw new Error(`no key ${id}`);
    return key;
  };

  return {
    identity: () => ({ name: 'explore-stored-rows-target', version: '1' }),
    beginScenario: () => reset(),
    endScenario: () => reset(),

    executeCommand({ command, input }) {
      switch (command) {
        case 'exploredraw.keys.Bind': {
          const length = [...String(input.secret)].length;
          if (mode === 'label-plain' && !['', 'a', 'b'].includes(input.label)) {
            throw new Error(`label ${JSON.stringify(input.label)} is not one of the plain texts`);
          }
          if (length < 12 || (mode === 'count-inclusive' && length === 12)) {
            return refused('too-short', 'exploredraw.keys.TooShort');
          }
          if (mode === 'uses-seven' && Number(input.uses) === 7) {
            return refused('too-short', 'exploredraw.keys.TooShort');
          }
          const id = input.key_id;
          if (keys.has(id) && mode !== 'existing-ignored') {
            return refused('already-bound', 'exploredraw.keys.AlreadyBound');
          }
          const state = keys.get(id)?.state ?? 'Active';
          keys.set(id, { state, secret: input.secret, label: input.label, plan: 'Basic', uses: input.uses });
          return answered('bound', 'exploredraw.keys.Bound', 'key_id', id);
        }
        case 'exploredraw.keys.Rotate': {
          const key = keys.get(input.key_id);
          if (key === undefined) return refused('unknown-key', 'exploredraw.keys.UnknownKey');
          if (key.secret === input.secret) {
            return refused('same-secret', 'exploredraw.keys.SameSecret');
          }
          key.secret = input.secret;
          return answered('rotated', 'exploredraw.keys.Rotated', 'key_id', input.key_id);
        }
        case 'exploredraw.keys.Upgrade': {
          const key = existing(input.key_id);
          if (key.plan === 'Basic' || mode === 'field-ignored') {
            key.plan = 'Pro';
            return answered('upgraded', 'exploredraw.keys.Upgraded', 'key_id', input.key_id);
          }
          return answered('kept', 'exploredraw.keys.Kept', 'key_id', input.key_id);
        }
        case 'exploredraw.keys.Revoke': {
          const key = existing(input.key_id);
          if (key.state === 'Revoked' && mode !== 'state-ignored') {
            return refused('already-revoked', 'exploredraw.keys.AlreadyRevoked');
          }
          key.state = 'Revoked';
          return answered('revoked', 'exploredraw.keys.Revoked', 'key_id', input.key_id);
        }
        case 'exploredraw.keys.Resume': {
          const key = existing(input.key_id);
          if (key.state === 'Suspended' || (key.state === 'Active' && mode === 'changes-ignored')) {
            key.state = 'Active';
            return answered('resumed', 'exploredraw.keys.Resumed', 'key_id', input.key_id);
          }
          if (key.state === 'Active') {
            return answered('restated', 'exploredraw.keys.Restated', 'key_id', input.key_id);
          }
          return refused('stuck', 'exploredraw.keys.Stuck');
        }
        case 'exploredraw.keys.Suspend': {
          const key = existing(input.key_id);
          if (key.state !== 'Active') return refused('not-active', 'exploredraw.keys.NotActive');
          key.state = 'Suspended';
          return answered('suspended', 'exploredraw.keys.Suspended', 'key_id', input.key_id);
        }
        case 'exploredraw.keys.Use': {
          const key = existing(input.key_id);
          if (Number(key.uses) < Number(input.amount) && mode !== 'predicate-ignored') {
            return refused('exhausted', 'exploredraw.keys.Exhausted');
          }
          if (mode === 'bulk' && Number(input.amount) > 100) {
            return answered('bulk', 'exploredraw.keys.Used', 'key_id', input.key_id);
          }
          return answered('used', 'exploredraw.keys.Used', 'key_id', input.key_id);
        }
        case 'exploredraw.keys.Enrol': {
          const id = `00000000-0000-4000-8000-${String(owners.length + 1).padStart(12, '0')}`;
          owners.push(id);
          return answered('enrolled', 'exploredraw.keys.Enrolled', 'owner_id', id);
        }
        default:
          throw unsupported(`${command} is not a command this target serves`);
      }
    },

    queryView({ view }) {
      if (view === 'exploredraw.keys.Keys') {
        return {
          rows: [...keys].map(([id, key]) => ({
            key_id: id,
            state: key.state,
            secret: key.secret,
            label: key.label,
            plan: key.plan,
            uses: key.uses,
          })),
        };
      }
      if (view === 'exploredraw.keys.Owners') {
        return { rows: owners.map((id) => ({ owner_id: id, state: 'Enrolled' })) };
      }
      throw unsupported(`${view} is not a view of explore-stored-rows.yaml`);
    },

    observeEvents: () => [],
    configureExternalOutcome: () => {
      throw unsupported('no external outcomes');
    },
    redeliverEvent: () => {
      throw unsupported('no redelivery');
    },
    observeInvocations: () => {
      throw unsupported('no bindings');
    },
  };
}
