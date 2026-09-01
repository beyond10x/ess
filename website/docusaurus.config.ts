import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';
import {themes as prismThemes} from 'prism-react-renderer';
import docsSystemPlugin from '@beyond10x/docs-system/docusaurus';

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
  plugins: [docsSystemPlugin],
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
        theme: {customCss: './src/css/custom.css'},
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    image: 'img/social-card.png',
    colorMode: {respectPrefersColorScheme: true},
    navbar: {
      title: 'ESS',
      logo: {alt: 'ESS', src: 'img/mark.svg', width: 26, height: 26},
      items: [
        {href: 'https://beyond10x.github.io/getting-started/', label: 'beyond10x', position: 'left'},
        {href: 'https://beyond10x.github.io/getting-started/ecosystem', label: 'Ecosystem', position: 'left'},
        {type: 'docSidebar', sidebarId: 'docsSidebar', position: 'left', label: 'Documentation'},
        {to: '/docs/examples/specification-to-contracts', label: 'See it work', position: 'left'},
        {to: '/lab', label: 'Browser lab', position: 'left'},
        {to: '/releases', label: 'Releases', position: 'left'},
        {
          href: 'https://github.com/beyond10x/ess',
          label: 'GitHub',
          position: 'right',
          className: 'navbar-github-link',
          'aria-label': 'GitHub repository',
        },
      ],
    },
    footer: {
      style: 'dark',
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
            {label: 'Generate artifacts', to: '/docs/guides/generate-artifacts'},
            {label: 'Verify conformance', to: '/docs/guides/verify-conformance'},
            {label: 'Browser lab', to: '/lab'},
          ],
        },
        {
          title: 'Project',
          items: [
            {label: 'Status', to: '/docs/status/where-this-stands'},
            {label: 'Limitations', to: '/docs/status/limitations'},
            {label: 'Roadmap', to: '/docs/status/roadmap'},
            {label: 'Source', href: 'https://github.com/beyond10x/ess'},
          ],
        },
      ],
      logo: {alt: 'ESS', src: 'img/mark.svg', href: '/', width: 22, height: 22},
      copyright:
        '<span class="footer__claim">System intent that compiles, projects, and proves itself.</span>' +
        'ESS · Apache-2.0 · built with Docusaurus.',
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ['rust', 'yaml', 'json', 'bash'],
    },
    mermaid: {theme: {light: 'neutral', dark: 'dark'}},
  } satisfies Preset.ThemeConfig,
};

export default config;
