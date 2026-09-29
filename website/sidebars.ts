import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  docsSidebar: [
    {
      type: 'category',
      label: 'Start here',
      items: ['index', 'getting-started'],
    },
    {
      type: 'category',
      label: 'Concepts',
      items: ['concepts/overview', 'concepts/ess', 'concepts/component-delivery'],
    },
    {
      type: 'category',
      label: 'Guides',
      items: [
        {
          type: 'category',
          label: 'Write a specification',
          link: {type: 'doc', id: 'guides/write-a-specification'},
          items: [
            'guides/specify/layout-and-validation',
            'guides/specify/fields-and-invariants',
            'guides/specify/guards-and-predicates',
            'guides/specify/commands-and-outcomes',
            'guides/specify/values-and-views',
            'guides/specify/bindings-and-components',
          ],
        },
        {
          type: 'category',
          label: 'Verify conformance',
          link: {type: 'doc', id: 'guides/verify-conformance'},
          items: [
            'guides/verify/synthesize-a-suite',
            'guides/verify/author-scenarios',
            'guides/verify/runners',
            'guides/verify/mutation-audit',
            'guides/verify/explore',
          ],
        },
        'guides/track-change',
        'guides/generate-artifacts',
        'guides/synthesize',
        'guides/check-infrastructure',
        'guides/record-realization',
      ],
    },
    {
      type: 'category',
      label: 'Reference',
      items: [
        'reference/cli',
        'reference/diagnostics',
        'reference/formats',
        'reference/spec-versions',
        'reference/predicates',
        'reference/glossary',
      ],
    },
    {
      type: 'category',
      label: 'Examples',
      items: ['examples/specification-to-contracts'],
    },
    {
      type: 'category',
      label: 'Status',
      items: ['status/where-this-stands', 'status/limitations', 'status/roadmap', 'status/outlook'],
    },
  ],
};

export default sidebars;
