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
        {
          type: 'category',
          label: 'Getting Started',
          collapsible: true,
          collapsed: true,
          link: {type: 'doc', id: 'getting-started/index'},
          items: [
            'getting-started/terminal-basics',
            'getting-started/install-git',
            'getting-started/install-github-cli',
          ],
        },
        {
          type: 'category',
          label: 'Working with Claude',
          collapsible: true,
          collapsed: true,
          link: {type: 'doc', id: 'working-with-claude/index'},
          items: [
            'working-with-claude/plan',
            'working-with-claude/build',
            'working-with-claude/autonomous-loops',
            'working-with-claude/verify',
            'working-with-claude/review',
          ],
        },
        'onboarding-project',
      ],
    },
    {
      type: 'category',
      label: 'Deep Dives',
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
