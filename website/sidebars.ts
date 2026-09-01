import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  docsSidebar: [
    {
      type: 'category',
      label: 'Start here',
      collapsed: false,
      items: ['index', 'getting-started'],
    },
    {
      type: 'category',
      label: 'Concepts',
      collapsed: false,
      items: ['concepts/overview', 'concepts/ess'],
    },
    {
      type: 'category',
      label: 'Guides',
      collapsed: false,
      items: [
        'guides/write-a-specification',
        'guides/generate-artifacts',
        'guides/verify-conformance',
        'guides/track-change',
        'guides/synthesize',
        'guides/check-infrastructure',
      ],
    },
    {
      type: 'category',
      label: 'Reference',
      collapsed: false,
      items: ['reference/cli'],
    },
    {
      type: 'category',
      label: 'Examples',
      collapsed: false,
      items: ['examples/specification-to-contracts'],
    },
    {
      type: 'category',
      label: 'Project status',
      collapsed: false,
      items: ['status/where-this-stands', 'status/limitations', 'status/roadmap'],
    },
  ],
};

export default sidebars;
