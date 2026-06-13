import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

const sidebars: SidebarsConfig = {
  handbookSidebar: [
    {
      type: 'category',
      label: 'Get started',
      collapsible: true,
      collapsed: false,
      items: [
        'getting-started',
        'working-with-claude',
        'onboarding-project',
      ],
    },
    {
      type: 'category',
      label: 'Reference',
      collapsible: true,
      collapsed: false,
      items: [
        'rules-explained',
        'repo-setup',
        'play-testing',
        'tooling-tour',
      ],
    },
  ],
};

export default sidebars;
