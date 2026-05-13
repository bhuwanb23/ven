/** @type {import('tailwindcss').Config} */
import forms from '@tailwindcss/forms'
import containerQueries from '@tailwindcss/container-queries'

export default {
  darkMode: 'class',
  content: ['./index.html', './src/**/*.{js,jsx,ts,tsx}'],
  theme: {
    extend: {
      // Material-3 palette ported from website_design/design.md.
      // Every token name matches the inline tailwind.config in the
      // hand-built HTML so classes like `bg-surface-container-lowest`
      // and `text-primary-fixed-dim` Just Work in the React port.
      colors: {
        background: '#131313',
        surface: '#131313',
        'surface-dim': '#131313',
        'surface-bright': '#3a3939',
        'surface-container-lowest': '#0e0e0e',
        'surface-container-low': '#1c1b1b',
        'surface-container': '#201f1f',
        'surface-container-high': '#2a2a2a',
        'surface-container-highest': '#353534',
        'surface-variant': '#353534',
        'surface-tint': '#00dbe7',
        'on-surface': '#e5e2e1',
        'on-surface-variant': '#b9cacb',
        'on-background': '#e5e2e1',
        'inverse-surface': '#e5e2e1',
        'inverse-on-surface': '#313030',
        outline: '#849495',
        'outline-variant': '#3a494b',
        primary: '#e1fdff',
        'on-primary': '#00363a',
        'primary-container': '#00f2ff',
        'on-primary-container': '#006a71',
        'inverse-primary': '#00696f',
        'primary-fixed': '#74f5ff',
        'primary-fixed-dim': '#00dbe7',
        'on-primary-fixed': '#002022',
        'on-primary-fixed-variant': '#004f54',
        secondary: '#ecffe3',
        'on-secondary': '#003907',
        'secondary-container': '#13ff43',
        'on-secondary-container': '#007117',
        'secondary-fixed': '#72ff70',
        'secondary-fixed-dim': '#00e639',
        'on-secondary-fixed': '#002203',
        'on-secondary-fixed-variant': '#00530e',
        tertiary: '#fff5f4',
        'on-tertiary': '#690003',
        'tertiary-container': '#ffd0ca',
        'on-tertiary-container': '#c3000a',
        'tertiary-fixed': '#ffdad5',
        'tertiary-fixed-dim': '#ffb4aa',
        'on-tertiary-fixed': '#410001',
        'on-tertiary-fixed-variant': '#930005',
        error: '#ffb4ab',
        'on-error': '#690005',
        'error-container': '#93000a',
        'on-error-container': '#ffdad6',
      },
      borderRadius: {
        DEFAULT: '0.125rem',
        lg: '0.25rem',
        xl: '0.5rem',
        full: '0.75rem',
      },
      spacing: {
        unit: '4px',
        gutter: '16px',
        'margin-mobile': '16px',
        'margin-desktop': '32px',
      },
      maxWidth: {
        // Used as `max-w-max-width` in the existing HTML.
        'max-width': '1280px',
      },
      fontFamily: {
        sans: ['Geist', 'system-ui', 'sans-serif'],
        mono: ['"JetBrains Mono"', 'ui-monospace', 'monospace'],
        'display-lg': ['Geist'],
        'headline-md': ['Geist'],
        'body-base': ['Geist'],
        'code-sm': ['"JetBrains Mono"'],
        'terminal-output': ['"JetBrains Mono"'],
      },
      fontSize: {
        'display-lg': ['48px', { lineHeight: '1.1', letterSpacing: '-0.04em', fontWeight: '700' }],
        'headline-md': ['24px', { lineHeight: '1.2', fontWeight: '600' }],
        'body-base': ['16px', { lineHeight: '1.6', fontWeight: '400' }],
        'code-sm': ['14px', { lineHeight: '1.5', fontWeight: '400' }],
        'terminal-output': ['13px', { lineHeight: '1.4', letterSpacing: '0.02em', fontWeight: '500' }],
      },
      boxShadow: {
        'cyan-glow': '0 0 20px rgba(0, 219, 231, 0.15)',
        'cyan-glow-strong': '0 0 20px rgba(0, 219, 231, 0.4)',
        'red-glow': '0 0 20px rgba(255, 59, 48, 0.3)',
        'nav-glow': '0 0 15px rgba(0, 219, 231, 0.1)',
      },
      keyframes: {
        'caret-blink': {
          '0%, 49%, 100%': { opacity: '1' },
          '50%, 99%': { opacity: '0' },
        },
      },
      animation: {
        'caret-blink': 'caret-blink 1s steps(1) infinite',
      },
    },
  },
  plugins: [forms, containerQueries],
}
