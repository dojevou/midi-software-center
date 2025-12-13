/** @type {import('tailwindcss').Config} */
export default {
  content: [
    './index.html',
    './src/**/*.{js,ts,jsx,tsx,svelte}',
  ],
  darkMode: 'class',
  theme: {
    extend: {
      // Color Palette - Enterprise Design System
      colors: {
        // Gray Scale
        gray: {
          50: '#fafafa',
          100: '#f4f4f5',
          200: '#e4e4e7',
          300: '#d4d4d8',
          400: '#a1a1aa',
          500: '#71717a',
          600: '#52525b',
          700: '#3f3f46',
          800: '#27272a',
          900: '#18181b',
          950: '#09090b',
        },

        // Primary - Electric Blue
        primary: {
          50: '#eff6ff',
          100: '#dbeafe',
          200: '#bfdbfe',
          300: '#93c5fd',
          400: '#60a5fa',
          500: '#3b82f6',
          600: '#2563eb',
          700: '#1d4ed8',
          800: '#1e40af',
          900: '#1e3a8a',
          950: '#172554',
          DEFAULT: '#3b82f6',
        },

        // Success - Emerald
        success: {
          50: '#ecfdf5',
          100: '#d1fae5',
          200: '#a7f3d0',
          300: '#6ee7b7',
          400: '#34d399',
          500: '#10b981',
          600: '#059669',
          700: '#047857',
          800: '#065f46',
          900: '#064e3b',
          DEFAULT: '#10b981',
        },

        // Warning - Amber
        warning: {
          50: '#fffbeb',
          100: '#fef3c7',
          200: '#fde68a',
          300: '#fcd34d',
          400: '#fbbf24',
          500: '#f59e0b',
          600: '#d97706',
          700: '#b45309',
          800: '#92400e',
          900: '#78350f',
          DEFAULT: '#f59e0b',
        },

        // Error - Rose
        error: {
          50: '#fff1f2',
          100: '#ffe4e6',
          200: '#fecdd3',
          300: '#fda4af',
          400: '#fb7185',
          500: '#f43f5e',
          600: '#e11d48',
          700: '#be123c',
          800: '#9f1239',
          900: '#881337',
          DEFAULT: '#f43f5e',
        },

        // Accent - Violet
        accent: {
          50: '#f5f3ff',
          100: '#ede9fe',
          200: '#ddd6fe',
          300: '#c4b5fd',
          400: '#a78bfa',
          500: '#8b5cf6',
          600: '#7c3aed',
          700: '#6d28d9',
          800: '#5b21b6',
          900: '#4c1d95',
          DEFAULT: '#8b5cf6',
        },

        // Semantic Colors
        'app-bg': '#09090b',
        'app-text': '#fafafa',
        menu: '#27272a',
        window: '#27272a',
        'window-border': '#3f3f46',
        'window-subtle': '#18181b',
        surface: '#27272a',
        input: '#18181b',
        hover: 'rgba(59, 130, 246, 0.1)',

        // FROGSKIN CAMO THEME - Gemstone Colors
        // Ruby - Deep Crimson Red
        ruby: {
          50: '#fef2f2',
          100: '#fee2e2',
          200: '#fecaca',
          300: '#fca5a5',
          400: '#f87171',
          500: '#dc2626',
          600: '#b91c1c',
          700: '#991b1b',
          800: '#7f1d1d',
          900: '#450a0a',
          950: '#2a0505',
          DEFAULT: '#dc2626',
        },

        // Emerald - Rich Forest Green (custom gem variant)
        'emerald-gem': {
          50: '#ecfdf5',
          100: '#d1fae5',
          200: '#a7f3d0',
          300: '#6ee7b7',
          400: '#34d399',
          500: '#059669',
          600: '#047857',
          700: '#065f46',
          800: '#064e3b',
          900: '#022c22',
          950: '#011613',
          DEFAULT: '#059669',
        },

        // Sapphire - Deep Royal Blue
        sapphire: {
          50: '#eff6ff',
          100: '#dbeafe',
          200: '#bfdbfe',
          300: '#93c5fd',
          400: '#60a5fa',
          500: '#1d4ed8',
          600: '#1e40af',
          700: '#1e3a8a',
          800: '#172554',
          900: '#0c1a3d',
          950: '#060d1f',
          DEFAULT: '#1d4ed8',
        },

        // Camo Pattern Base Colors
        camo: {
          base: '#0d1117',
          'ruby-dark': '#450a0a',
          'ruby-mid': '#7f1d1d',
          'ruby-light': '#dc2626',
          'emerald-dark': '#022c22',
          'emerald-mid': '#065f46',
          'emerald-light': '#059669',
          'sapphire-dark': '#0c1a3d',
          'sapphire-mid': '#1e3a8a',
          'sapphire-light': '#1d4ed8',
        },
      },

      // Font Family
      fontFamily: {
        sans: [
          'ui-sans-serif',
          'system-ui',
          '-apple-system',
          'BlinkMacSystemFont',
          '"Segoe UI"',
          'Roboto',
          '"Helvetica Neue"',
          'Arial',
          'sans-serif',
        ],
        mono: [
          'ui-monospace',
          '"SF Mono"',
          '"Cascadia Code"',
          '"Roboto Mono"',
          'Menlo',
          'Monaco',
          'Consolas',
          'monospace',
        ],
      },

      // Font Size
      fontSize: {
        '2xs': ['0.625rem', { lineHeight: '0.875rem' }],
        xs: ['0.75rem', { lineHeight: '1rem' }],
        sm: ['0.875rem', { lineHeight: '1.25rem' }],
        base: ['1rem', { lineHeight: '1.5rem' }],
        lg: ['1.125rem', { lineHeight: '1.75rem' }],
        xl: ['1.25rem', { lineHeight: '1.75rem' }],
        '2xl': ['1.5rem', { lineHeight: '2rem' }],
        '3xl': ['1.875rem', { lineHeight: '2.25rem' }],
        '4xl': ['2.25rem', { lineHeight: '2.5rem' }],
        '5xl': ['3rem', { lineHeight: '1' }],
        '6xl': ['3.75rem', { lineHeight: '1' }],
      },

      // Spacing
      spacing: {
        px: '1px',
        0: '0',
        0.5: '0.125rem',
        1: '0.25rem',
        1.5: '0.375rem',
        2: '0.5rem',
        2.5: '0.625rem',
        3: '0.75rem',
        3.5: '0.875rem',
        4: '1rem',
        5: '1.25rem',
        6: '1.5rem',
        7: '1.75rem',
        8: '2rem',
        9: '2.25rem',
        10: '2.5rem',
        11: '2.75rem',
        12: '3rem',
        14: '3.5rem',
        16: '4rem',
        20: '5rem',
        24: '6rem',
        28: '7rem',
        32: '8rem',
        36: '9rem',
        40: '10rem',
        44: '11rem',
        48: '12rem',
        52: '13rem',
        56: '14rem',
        60: '15rem',
        64: '16rem',
        72: '18rem',
        80: '20rem',
        96: '24rem',
      },

      // Border Radius
      borderRadius: {
        none: '0',
        sm: '0.125rem',
        DEFAULT: '0.25rem',
        md: '0.375rem',
        lg: '0.5rem',
        xl: '0.75rem',
        '2xl': '1rem',
        '3xl': '1.5rem',
        full: '9999px',
      },

      // Box Shadow - Elevation System
      boxShadow: {
        xs: '0 1px 2px 0 rgb(0 0 0 / 0.05)',
        sm: '0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1)',
        DEFAULT: '0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1)',
        md: '0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)',
        lg: '0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)',
        xl: '0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1)',
        '2xl': '0 25px 50px -12px rgb(0 0 0 / 0.25)',
        inner: 'inset 0 2px 4px 0 rgb(0 0 0 / 0.05)',
        none: '0 0 #0000',
        // Focus rings
        'primary-ring': '0 0 0 3px rgb(59 130 246 / 0.3)',
        'error-ring': '0 0 0 3px rgb(244 63 94 / 0.3)',
        'success-ring': '0 0 0 3px rgb(16 185 129 / 0.3)',
        // Frogskin Camo Glows
        'ruby-glow': '0 0 20px rgba(220, 38, 38, 0.4), 0 0 40px rgba(220, 38, 38, 0.4)',
        'emerald-glow': '0 0 20px rgba(5, 150, 105, 0.4), 0 0 40px rgba(5, 150, 105, 0.4)',
        'sapphire-glow': '0 0 20px rgba(29, 78, 216, 0.4), 0 0 40px rgba(29, 78, 216, 0.4)',
        'frogskin-glow': '0 0 15px rgba(220, 38, 38, 0.4), 0 0 30px rgba(5, 150, 105, 0.4), 0 0 45px rgba(29, 78, 216, 0.4)',
      },

      // Z-Index Scale
      zIndex: {
        auto: 'auto',
        0: '0',
        10: '10',
        20: '20',
        30: '30',
        40: '40',
        50: '50',
        dropdown: '100',
        sticky: '200',
        fixed: '300',
        'modal-backdrop': '400',
        modal: '500',
        popover: '600',
        tooltip: '700',
        toast: '800',
        max: '9999',
      },

      // Transition
      transitionDuration: {
        75: '75ms',
        100: '100ms',
        150: '150ms',
        200: '200ms',
        300: '300ms',
        500: '500ms',
        700: '700ms',
        1000: '1000ms',
      },

      transitionTimingFunction: {
        DEFAULT: 'cubic-bezier(0.4, 0, 0.2, 1)',
        linear: 'linear',
        in: 'cubic-bezier(0.4, 0, 1, 1)',
        out: 'cubic-bezier(0, 0, 0.2, 1)',
        'in-out': 'cubic-bezier(0.4, 0, 0.2, 1)',
        bounce: 'cubic-bezier(0.68, -0.55, 0.265, 1.55)',
      },

      // Animation
      animation: {
        none: 'none',
        spin: 'spin 1s linear infinite',
        ping: 'ping 1s cubic-bezier(0, 0, 0.2, 1) infinite',
        pulse: 'pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        bounce: 'bounce 1s infinite',
        'fade-in': 'fadeIn 0.2s ease-out',
        'fade-out': 'fadeOut 0.2s ease-in',
        'slide-in-up': 'slideInUp 0.2s ease-out',
        'slide-in-down': 'slideInDown 0.2s ease-out',
        'slide-in-left': 'slideInLeft 0.2s ease-out',
        'slide-in-right': 'slideInRight 0.2s ease-out',
        'scale-in': 'scaleIn 0.2s ease-out',
        shimmer: 'shimmer 1.5s infinite',
        // Frogskin Camo Animations
        'frogskin-drift': 'frogskinDrift 60s linear infinite',
        'frogskin-pulse': 'frogskinPulse 3s ease-in-out infinite',
        'frogskin-border': 'frogskinBorder 4s linear infinite',
        'gem-glow': 'gemGlow 2s ease-in-out infinite',
      },

      keyframes: {
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        fadeOut: {
          '0%': { opacity: '1' },
          '100%': { opacity: '0' },
        },
        slideInUp: {
          '0%': { transform: 'translateY(10px)', opacity: '0' },
          '100%': { transform: 'translateY(0)', opacity: '1' },
        },
        slideInDown: {
          '0%': { transform: 'translateY(-10px)', opacity: '0' },
          '100%': { transform: 'translateY(0)', opacity: '1' },
        },
        slideInLeft: {
          '0%': { transform: 'translateX(-10px)', opacity: '0' },
          '100%': { transform: 'translateX(0)', opacity: '1' },
        },
        slideInRight: {
          '0%': { transform: 'translateX(10px)', opacity: '0' },
          '100%': { transform: 'translateX(0)', opacity: '1' },
        },
        scaleIn: {
          '0%': { transform: 'scale(0.95)', opacity: '0' },
          '100%': { transform: 'scale(1)', opacity: '1' },
        },
        shimmer: {
          '0%': { backgroundPosition: '200% 0' },
          '100%': { backgroundPosition: '-200% 0' },
        },
        // Frogskin Camo Keyframes
        frogskinDrift: {
          '0%': { backgroundPosition: '0 0' },
          '100%': { backgroundPosition: '400px 400px' },
        },
        frogskinPulse: {
          '0%, 100%': {
            boxShadow: '0 0 5px rgba(220, 38, 38, 0.4), 0 0 10px rgba(5, 150, 105, 0.4), 0 0 15px rgba(29, 78, 216, 0.4)',
          },
          '33%': {
            boxShadow: '0 0 15px rgba(220, 38, 38, 0.4), 0 0 5px rgba(5, 150, 105, 0.4), 0 0 10px rgba(29, 78, 216, 0.4)',
          },
          '66%': {
            boxShadow: '0 0 10px rgba(220, 38, 38, 0.4), 0 0 15px rgba(5, 150, 105, 0.4), 0 0 5px rgba(29, 78, 216, 0.4)',
          },
        },
        frogskinBorder: {
          '0%': { backgroundPosition: '0% 50%' },
          '100%': { backgroundPosition: '300% 50%' },
        },
        gemGlow: {
          '0%, 100%': {
            boxShadow: '0 0 10px rgba(220, 38, 38, 0.3), 0 0 20px rgba(5, 150, 105, 0.3), 0 0 30px rgba(29, 78, 216, 0.3)',
          },
          '50%': {
            boxShadow: '0 0 20px rgba(220, 38, 38, 0.5), 0 0 40px rgba(5, 150, 105, 0.5), 0 0 60px rgba(29, 78, 216, 0.5)',
          },
        },
      },

      // Width/Height
      width: {
        'window-min': '320px',
        'window-sm': '400px',
        'window-md': '600px',
        'window-lg': '800px',
        'window-xl': '1000px',
      },
      height: {
        'window-min': '200px',
        'window-sm': '300px',
        'window-md': '400px',
        'window-lg': '600px',
        header: '40px',
        'status-bar': '28px',
        'menu-bar': '36px',
      },
      minWidth: {
        window: '320px',
      },
      minHeight: {
        window: '200px',
      },

      // Opacity
      opacity: {
        0: '0',
        5: '0.05',
        10: '0.1',
        20: '0.2',
        25: '0.25',
        30: '0.3',
        40: '0.4',
        50: '0.5',
        60: '0.6',
        70: '0.7',
        75: '0.75',
        80: '0.8',
        90: '0.9',
        95: '0.95',
        100: '1',
      },
    },
  },

  plugins: [],
};
