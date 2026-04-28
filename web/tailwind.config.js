/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        // Primary - Deep Teal (50-900 scale)
        'primary': {
          DEFAULT: 'var(--color-primary-600)',
          50: 'var(--color-primary-50)',
          100: 'var(--color-primary-100)',
          200: 'var(--color-primary-200)',
          300: 'var(--color-primary-300)',
          400: 'var(--color-primary-400)',
          500: 'var(--color-primary-500)',
          600: 'var(--color-primary-600)',
          700: 'var(--color-primary-700)',
          800: 'var(--color-primary-800)',
          900: 'var(--color-primary-900)',
        },

        // Surface (0-300 scale)
        'surface': {
          0: 'var(--color-surface-0)',
          50: 'var(--color-surface-50)',
          100: 'var(--color-surface-100)',
          200: 'var(--color-surface-200)',
          300: 'var(--color-surface-300)',
        },

        // Text
        'text': {
          primary: 'var(--color-text-primary)',
          secondary: 'var(--color-text-secondary)',
          tertiary: 'var(--color-text-tertiary)',
          inverse: 'var(--color-text-inverse)',
        },

        // Semantic (Status)
        'success': 'var(--color-success)',
        'error': 'var(--color-error)',
        'warning': 'var(--color-warning)',
        'info': 'var(--color-info)',

        // Metric
        'metric': {
          positive: 'var(--color-metric-positive)',
          negative: 'var(--color-metric-negative)',
          neutral: 'var(--color-metric-neutral)',
          threshold: 'var(--color-metric-threshold)',
        },
      },
      fontFamily: {
        mono: ['JetBrains Mono', 'Fira Code', 'Cascadia Code', 'ui-monospace', 'monospace'],
        sans: ['Inter', 'system-ui', '-apple-system', 'sans-serif'],
      },
      fontSize: {
        'micro':   ['0.625rem', { lineHeight: '1.4' }],   // 10px
        'xs':      ['0.75rem',  { lineHeight: '1.4' }],   // 12px
        'sm':      ['0.875rem', { lineHeight: '1.5' }],   // 14px
        'base':    ['1rem',     { lineHeight: '1.5' }],   // 16px
        'lg':      ['1.125rem', { lineHeight: '1.5' }],   // 18px
        'xl':      ['1.25rem',  { lineHeight: '1.4' }],   // 20px
        '2xl':     ['1.5rem',   { lineHeight: '1.3' }],   // 24px
        'display': ['2rem',     { lineHeight: '1.1' }],   // 32px
      },
      spacing: {
        '0':   '0',
        '1':   '0.25rem',   //  4px
        '2':   '0.5rem',    //  8px — base unit
        '3':   '0.75rem',   // 12px
        '4':   '1rem',      // 16px
        '5':   '1.25rem',   // 20px
        '6':   '1.5rem',    // 24px
        '8':   '2rem',      // 32px
        '10':  '2.5rem',    // 40px
        '12':  '3rem',      // 48px
        '16':  '4rem',      // 64px
      },
      borderRadius: {
        'sm': '0.25rem',   //  4px
        'md': '0.5rem',    //  8px
        'lg': '0.75rem',   // 12px
        'xl': '1rem',      // 16px
      },
      maxWidth: {
        'content': '1400px',
      },
    },
  },
  plugins: [],
}
