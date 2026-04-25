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
        // Primary - Deep Teal
        'primary': {
          DEFAULT: 'var(--color-primary)',
          light: 'var(--color-primary-light)',
          dark: 'var(--color-primary-dark)',
        },
        
        // Background
        'bg': 'var(--color-bg)',
        'surface': 'var(--color-surface)',
        'surface-hover': 'var(--color-surface-hover)',
        'border': {
          DEFAULT: 'var(--color-border)',
        },
        
        // Text
        'text': {
          primary: 'var(--color-text-primary)',
          secondary: 'var(--color-text-secondary)',
          muted: 'var(--color-text-muted)',
        },
        
        // Semantic
        'success': 'var(--color-success)',
        'error': 'var(--color-error)',
        'warning': 'var(--color-warning)',
        'info': 'var(--color-info)',
        
        // Accents
        'accent': {
          pink: 'var(--color-accent-pink)',
          orange: 'var(--color-accent-orange)',
          purple: 'var(--color-accent-purple)',
        },
      },
      fontFamily: {
        mono: ['JetBrains Mono', 'Consolas', 'monospace'],
        sans: ['Inter', 'system-ui', 'sans-serif'],
      },
      fontSize: {
        'display': ['3rem', { lineHeight: '1.1' }],
        'h1': ['2rem', { lineHeight: '1.1' }],
        'h2': ['1.5rem', { lineHeight: '1.1' }],
        'h3': ['1.25rem', { lineHeight: '1.1' }],
        'micro': ['0.75rem', { lineHeight: '1.4' }],
      },
      spacing: {
        'xs': '0.25rem',
        'sm': '0.5rem',
        'md': '1rem',
        'lg': '1.5rem',
        'xl': '2rem',
        '2xl': '3rem',
        '3xl': '4rem',
      },
      maxWidth: {
        'content': '1400px',
      },
    },
  },
  plugins: [],
}
