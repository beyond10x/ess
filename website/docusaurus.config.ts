import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';
import {withProductSite} from '@beyond10x/docs-system/product-site';
import canonicalToUnifiedDocs from './src/canonical';

const config: Config = {
  title: 'ESS',
  tagline:
    'Turn system intent into validated typed models, deterministic artifacts, and executable conformance checks.',
  favicon: 'img/mark.svg',

  future: {v4: true},
  url: 'https://beyond10x.github.io',
  baseUrl: '/ess/',
  organizationName: 'beyond10x',
  projectName: 'ess',
  deploymentBranch: 'gh-pages',
  trailingSlash: false,
  onBrokenLinks: 'throw',

  markdown: {
    hooks: {onBrokenMarkdownLinks: 'throw'},
    mermaid: true,
  },
  themes: ['@docusaurus/theme-mermaid'],
  plugins: [
    canonicalToUnifiedDocs,
    [
      '@docusaurus/plugin-client-redirects',
      {
        redirects: [{from: '/lab', to: '/docs/visualise'}],
      },
    ],
  ],
  i18n: {defaultLocale: 'en', locales: ['en']},

  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          routeBasePath: 'docs',
          editUrl: 'https://github.com/beyond10x/ess/tree/main/website/',
        },
        blog: {
          routeBasePath: 'releases',
          blogTitle: 'ESS releases, in practice',
          blogDescription: 'Worked records of what each ESS capability added.',
          blogSidebarTitle: 'All releases',
          blogSidebarCount: 'ALL',
          showReadingTime: true,
          onUntruncatedBlogPosts: 'throw',
          editUrl: 'https://github.com/beyond10x/ess/tree/main/website/',
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    image: 'img/social-card.png',
    navbar: {
      title: 'ESS',
      items: [
        {
          to: '/docs',
          label: 'Docs',
          position: 'left',
          activeBaseRegex: '^/ess/docs(?!/(examples/|visualise|reference/cli|status/))',
        },
        {to: '/docs/examples/specification-to-contracts', label: 'Example', position: 'left'},
        {to: '/docs/visualise', label: 'Visualise', position: 'left'},
        {to: '/docs/reference/cli', label: 'CLI', position: 'left'},
        {to: '/releases', label: 'Releases', position: 'left'},
        {to: '/docs/status/where-this-stands', label: 'Status', position: 'left'},
        {href: 'https://github.com/beyond10x/ess', label: 'GitHub', position: 'right'},
      ],
    },
    footer: {
      links: [
        {
          title: 'Documentation',
          items: [
            {label: 'Introduction', to: '/docs'},
            {label: 'Getting started', to: '/docs/getting-started'},
            {label: 'Architecture', to: '/docs/concepts/overview'},
            {label: 'CLI reference', to: '/docs/reference/cli'},
          ],
        },
        {
          title: 'Build and verify',
          items: [
            {label: 'Specification to contracts', to: '/docs/examples/specification-to-contracts'},
            {label: 'Visualise the billing model', to: '/docs/visualise'},
            {label: 'Generate artifacts', to: '/docs/guides/generate-artifacts'},
            {label: 'Verify conformance', to: '/docs/guides/verify-conformance'},
          ],
        },
        {
          title: 'Project',
          items: [
            {label: 'Status', to: '/docs/status/where-this-stands'},
            {label: 'Limitations', to: '/docs/status/limitations'},
            {label: 'Roadmap', to: '/docs/status/roadmap'},
            {label: 'Releases', to: '/releases'},
            {label: 'Source', href: 'https://github.com/beyond10x/ess'},
          ],
        },
        {
          title: 'Family',
          items: [
            {label: 'Canon', href: 'https://beyond10x.github.io/canon/'},
            {label: 'ELS', href: 'https://beyond10x.github.io/els/'},
            {label: 'Loom', href: 'https://beyond10x.github.io/loom/'},
            {label: 'Commission', href: 'https://beyond10x.github.io/commission/'},
          ],
        },
      ],
      copyright: 'A beyond10x project · Apache-2.0 · built with Docusaurus and the docs-system product template.',
    },
  } satisfies Preset.ThemeConfig,
};

export default withProductSite(config, {landing: './product.json', product: 'ess', mark: 'Es'});
