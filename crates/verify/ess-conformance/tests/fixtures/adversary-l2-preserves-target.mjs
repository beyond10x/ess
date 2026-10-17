// An in-memory implementation of `adversary-l2-preserves.yaml` (beyond10x/ess#536, adversary pass 1).
//
// `mode` selects the double:
//
//   (empty)            the specification
//   same-title-writes  `Retitle` answers `same-title` and appends `!` to the stored title
//   dry-run-writes     `Touch` answers `dry-run` and writes the input title with `!` appended
//   already-archives   `Publish` answers `already-published` and moves the document to `Archived`
//
// `adversary_l2_preserves_target.go` is the same target in Go.

import { unsupported } from './dist/index.js';

export function newTarget(mode = '') {
  let docs = new Map();
  let created = 0;
  let version = 0;

  const reset = () => {
    docs = new Map();
    created = 0;
    version = 0;
  };
  const refused = (outcome, error) => ({ outcome, error, directEvents: [] });
  const answered = (outcome, event, id) => {
    version += 1;
    return { outcome, consistency: `v${version}`, directEvents: [{ event, payload: { doc_id: id } }] };
  };
  const kept = (outcome) => {
    version += 1;
    return { outcome, consistency: `v${version}`, directEvents: [] };
  };

  return {
    identity: () => ({ name: 'adversary-l2-preserves-target', version: '1' }),
    beginScenario: () => reset(),
    endScenario: () => reset(),

    executeCommand({ command, input }) {
      const doc = docs.get(input.doc_id);
      if (command !== 'advkeep.docs.Create' && doc === undefined)
        throw new Error(`no document ${input.doc_id}`);
      switch (command) {
        case 'advkeep.docs.Create': {
          created += 1;
          const id = `00000000-0000-4000-8000-${String(created).padStart(12, '0')}`;
          docs.set(id, { state: 'Draft', title: input.title });
          return answered('created', 'advkeep.docs.Created', id);
        }
        case 'advkeep.docs.Retitle': {
          if (doc.title === input.title) {
            if (mode === 'same-title-writes') doc.title = `${input.title}!`;
            return kept('same-title');
          }
          doc.title = input.title;
          return answered('retitled', 'advkeep.docs.Retitled', input.doc_id);
        }
        case 'advkeep.docs.Touch': {
          if (input.dry === true) {
            if (mode === 'dry-run-writes') doc.title = `${input.title}!`;
            return kept('dry-run');
          }
          doc.title = input.title;
          return answered('touched', 'advkeep.docs.Touched', input.doc_id);
        }
        case 'advkeep.docs.Publish': {
          if (doc.state === 'Published') {
            if (mode === 'already-archives') doc.state = 'Archived';
            return kept('already-published');
          }
          if (doc.state === 'Draft') {
            doc.state = 'Published';
            return answered('published', 'advkeep.docs.Published', input.doc_id);
          }
          return refused('not-draft', 'advkeep.docs.NotDraft');
        }
        case 'advkeep.docs.Archive': {
          if (doc.state !== 'Published') return refused('not-published', 'advkeep.docs.NotPublished');
          doc.state = 'Archived';
          return answered('archived', 'advkeep.docs.Archived', input.doc_id);
        }
        default:
          throw unsupported(`${command} is not a command of adversary-l2-preserves.yaml`);
      }
    },

    queryView({ view }) {
      const all = [...docs];
      switch (view) {
        case 'advkeep.docs.Docs':
          return {
            rows: all.map(([id, doc]) => ({ doc_id: id, state: doc.state, title: doc.title })),
          };
        case 'advkeep.docs.PublishedDocs':
          return {
            rows: all
              .filter(([, doc]) => doc.state === 'Published')
              .map(([id, doc]) => ({ doc_id: id, title: doc.title })),
          };
        default:
          throw unsupported(`${view} is not a view of adversary-l2-preserves.yaml`);
      }
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
