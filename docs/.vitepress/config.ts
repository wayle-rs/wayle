import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Wayle',
  description: 'A configurable desktop shell for Wayland.',
  cleanUrls: true,
  lastUpdated: true,

  head: [
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/wayle.svg' }],
  ],

  markdown: {
    theme: {
      light: 'github-light',
      dark: 'tokyo-night',
    },
  },

  themeConfig: {
    logo: '/wayle.svg',

    nav: [
      { text: 'Guide', link: '/guide/getting-started', activeMatch: '^/guide/' },
      { text: 'Config', link: '/config/', activeMatch: '^/config/' },
    ],

    sidebar: {
      '/guide/': [
        {
          text: 'Guide',
          items: [
            {
              text: 'Getting started',
              link: '/guide/getting-started',
              collapsed: false,
              items: [
                { text: 'Arch Linux', link: '/guide/getting-started-arch' },
                { text: 'Debian / Ubuntu', link: '/guide/getting-started-debian' },
                { text: 'Fedora', link: '/guide/getting-started-fedora' },
                { text: 'NixOS', link: '/guide/getting-started-nixos' },
              ],
            },
            { text: 'Editing config', link: '/guide/editing-config' },
            { text: 'Bars and layouts', link: '/guide/bars-and-layouts' },
            { text: 'Themes', link: '/guide/themes' },
            { text: 'Custom icons', link: '/guide/custom-icons' },
            { text: 'Custom modules', link: '/guide/custom-modules' },
            { text: 'CLI', link: '/guide/cli' },
          ],
        },
        {
          text: 'Also see',
          items: [
            { text: 'Config reference', link: '/config/' },
          ],
        },
      ],

      '/config/': [
        {
          text: 'Config',
          items: [
            { text: 'Overview', link: '/config/' },
            { text: 'Types', link: '/config/types' },
          ],
        },
        {
          text: 'Top-level',
          items: [
            { text: 'bar', link: '/config/bar' },
            { text: 'styling', link: '/config/styling' },
            { text: 'general', link: '/config/general' },
            { text: 'osd', link: '/config/osd' },
            { text: 'wallpaper', link: '/config/wallpaper' },
          ],
        },
        {
          text: 'Modules',
          items: [
            { text: 'battery', link: '/config/modules/battery' },
            { text: 'bluetooth', link: '/config/modules/bluetooth' },
            { text: 'cava', link: '/config/modules/cava' },
            { text: 'clock', link: '/config/modules/clock' },
            { text: 'cpu', link: '/config/modules/cpu' },
            { text: 'custom', link: '/config/modules/custom' },
            { text: 'dashboard', link: '/config/modules/dashboard' },
            { text: 'hyprland-workspaces', link: '/config/modules/hyprland-workspaces' },
            { text: 'hyprsunset', link: '/config/modules/hyprsunset' },
            { text: 'idle-inhibit', link: '/config/modules/idle-inhibit' },
            { text: 'keybind-mode', link: '/config/modules/keybind-mode' },
            { text: 'keyboard-input', link: '/config/modules/keyboard-input' },
            { text: 'media', link: '/config/modules/media' },
            { text: 'microphone', link: '/config/modules/microphone' },
            { text: 'netstat', link: '/config/modules/netstat' },
            { text: 'network', link: '/config/modules/network' },
            { text: 'notification', link: '/config/modules/notification' },
            { text: 'power', link: '/config/modules/power' },
            { text: 'ram', link: '/config/modules/ram' },
            { text: 'separator', link: '/config/modules/separator' },
            { text: 'storage', link: '/config/modules/storage' },
            { text: 'systray', link: '/config/modules/systray' },
            { text: 'volume', link: '/config/modules/volume' },
            { text: 'weather', link: '/config/modules/weather' },
            { text: 'window-title', link: '/config/modules/window-title' },
            { text: 'world-clock', link: '/config/modules/world-clock' },
          ],
        },
      ],
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/wayle-rs/wayle' },
    ],

    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright 2026 Wayle contributors',
    },

    editLink: {
      pattern: 'https://github.com/wayle-rs/wayle/edit/master/docs/:path',
      text: 'Edit this page on GitHub',
    },

    outline: {
      level: [2, 3],
    },

    search: {
      provider: 'local',
    },
  },
})
