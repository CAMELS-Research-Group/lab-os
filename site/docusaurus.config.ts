import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

// "Edit this page" targets this branch on GitHub. Production builds use main;
// override for pre-merge review builds, e.g.:
//   $env:LAB_OS_EDIT_BRANCH = 'feat/site-content'; npm run build
const editBranch = process.env.LAB_OS_EDIT_BRANCH ?? 'main';
const editUrl = `https://github.com/WatsonWBlair/lab-os/edit/${editBranch}/site/`;

// LAB_OS_EDIT_LOCAL=1 makes "Edit this page" open the local source file in
// VS Code (vscode:// protocol) instead of GitHub — for local review builds
// only; saved edits land in the working tree. Never set in CI.
const localEdit = process.env.LAB_OS_EDIT_LOCAL === '1';
const vscodeFile = (relFromSiteDir: string) =>
  `vscode://file/${`${__dirname}/${relFromSiteDir}`.replace(/\\/g, '/')}`;

const config: Config = {
  title: 'lab-os',
  tagline: 'CAMELS Research Group — lab handbook',
  favicon: 'img/favicon.ico',

  future: {
    v4: true, // Improve compatibility with the upcoming Docusaurus v4
    faster: true, // Enable rspack/SWC toolchain
  },

  url: 'https://watsonwblair.github.io',
  baseUrl: '/lab-os/',

  // GitHub pages deployment config.
  organizationName: 'WatsonWBlair',
  projectName: 'lab-os',

  onBrokenLinks: 'throw',

  markdown: {
    hooks: {
      onBrokenMarkdownLinks: 'throw',
    },
  },

  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          // "Edit this page" → GitHub's web editor; saving commits via the normal PR flow
          editUrl: localEdit
            ? ({docPath}) => vscodeFile(`docs/${docPath}`)
            : editUrl,
        },
        blog: false,
        pages: {
          editUrl: localEdit
            ? ({pagesPath}) => vscodeFile(`src/pages/${pagesPath}`)
            : editUrl,
        },
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    colorMode: {
      respectPrefersColorScheme: true,
    },
    navbar: {
      title: 'lab-os',
      logo: {
        alt: 'lab-os logo',
        src: 'img/logo.svg',
      },
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'handbookSidebar',
          position: 'left',
          label: 'Handbook',
        },
        {
          href: 'https://github.com/WatsonWBlair/lab-os',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          label: 'GitHub',
          href: 'https://github.com/WatsonWBlair/lab-os',
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} CAMELS Research Group`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
