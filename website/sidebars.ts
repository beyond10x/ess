import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  docsSidebar: [
    {
      type: 'category',
      label: 'Start here',
      items: [
        'index',
        'getting-started',
        'start/install',
        'start/first-specification',
        'start/first-conformance-run',
        {
          type: 'category',
          label: 'Runners',
          items: ['start/runners/typescript', 'start/runners/go', 'start/runners/rust'],
        },
        'start/use-with-an-agent',
        'start/explore-the-example',
      ],
    },
    {
      type: 'category',
      label: 'Concepts',
      items: [
        'concepts/overview',
        'concepts/ess',
        'concepts/component-delivery',
        'concepts/test-pyramid',
      ],
    },
    {
      type: 'category',
      label: 'Guides',
      link: {type: 'generated-index', slug: '/guides'},
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
        'guides/deliver/build-and-chart',
        'guides/deliver/resolve-a-stack',
        'guides/deliver/deploy-an-environment',
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
      label: 'Releases',
      items: [
        {type: 'link', label: 'Release posts', href: '/releases'},
        'releases/what-changed',
      ],
    },
    {
      type: 'category',
      label: 'Status',
      items: ['status/where-this-stands', 'status/limitations', 'status/roadmap', 'status/outlook'],
    },
  ],
};

export default sidebars;
