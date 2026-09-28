// An in-memory implementation of `explore-preconditions.yaml` (ess/15, beyond10x/ess#152): every
// client command runs inside an open session, and opening one creates a user row.
//
// `mode` selects the double:
//
//   (empty)          opens sessions, and refuses every other command outside one
//   refuses-session  answers no declared branch for `OpenSession`
//   drops-accounts   opens sessions, and stores no account whatever the input carries
//
// `explore_preconditions_target.go` is the same target in Go, line for line.

import { unsupported } from './dist/index.js';

export function newTarget(mode = '') {
  let users = [];
  let version = 0;
  const reset = () => {
    users = [];
    version = 0;
  };
  const answered = (outcome, event, payload) => {
    version += 1;
    return { outcome, consistency: `v${version}`, directEvents: [{ event, payload }] };
  };

  return {
    identity: () => ({ name: 'explore-preconditions-target', version: '1' }),
    beginScenario: () => reset(),
    endScenario: () => reset(),

    executeCommand({ command, input }) {
      switch (command) {
        case 'explorepre.desk.OpenSession': {
          if (mode === 'refuses-session') return { outcome: '', directEvents: [] };
          const accounts = mode === 'drops-accounts' ? [] : input.accounts;
          users.push({ user_id: input.user_id, accounts });
          return answered('opened', 'explorepre.desk.SessionOpened', { user_id: input.user_id });
        }
        case 'explorepre.desk.ClearWrapUp': {
          if (users.length === 0) return { outcome: '', directEvents: [] };
          return answered('cleared', 'explorepre.desk.Cleared', {});
        }
        default:
          throw unsupported(`${command} is not a command of explore-preconditions.yaml`);
      }
    },

    queryView({ view }) {
      if (view === 'explorepre.desk.Users') {
        return { rows: users.map((user) => ({ ...user, state: 'Active' })) };
      }
      throw unsupported(`${view} is not a view of explore-preconditions.yaml`);
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
