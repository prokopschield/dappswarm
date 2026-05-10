import type { Config } from 'tailwindcss';

const config: Config = {
  content: ['./src/**/*.{html,svelte,ts,js}'],
  theme: {
    extend: {
      colors: {
        ink: {
          DEFAULT: '#0a0807',
          surface: '#171109',
          raised: '#1f1810',
          border: '#2a2014',
        },
        honey: {
          DEFAULT: '#fbbf24',
          soft: '#f59e0b',
          deep: '#b45309',
          glow: 'rgba(251, 191, 36, 0.18)',
        },
        cream: {
          DEFAULT: '#fef3c7',
          mute: '#d6d3cb',
        },
        mute: '#a8a29e',
      },
      fontFamily: {
        display: ['"Inter Variable"', 'system-ui', 'sans-serif'],
        sans: ['"Inter Variable"', 'system-ui', 'sans-serif'],
        mono: ['"JetBrains Mono Variable"', 'ui-monospace', 'monospace'],
      },
      letterSpacing: {
        tightest: '-0.06em',
      },
      maxWidth: {
        prose: '64ch',
        page: '76rem',
      },
      keyframes: {
        rise: {
          '0%': { opacity: '0', transform: 'translateY(20px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        glow: {
          '0%, 100%': { filter: 'drop-shadow(0 0 6px rgba(251, 191, 36, 0.3))' },
          '50%': { filter: 'drop-shadow(0 0 24px rgba(251, 191, 36, 0.7))' },
        },
        drift: {
          '0%, 100%': { transform: 'translate(0, 0)' },
          '50%': { transform: 'translate(8px, -12px)' },
        },
        scan: {
          '0%': { transform: 'translateX(-100%)' },
          '100%': { transform: 'translateX(100%)' },
        },
      },
      animation: {
        rise: 'rise 600ms cubic-bezier(0.2, 0.8, 0.2, 1) both',
        glow: 'glow 4s ease-in-out infinite',
        drift: 'drift 8s ease-in-out infinite',
        scan: 'scan 2.4s ease-in-out infinite',
      },
    },
  },
  plugins: [],
};

export default config;
