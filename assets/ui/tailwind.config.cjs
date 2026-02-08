/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "../../apps/desktop/src/**/*.{rs,html}",
    "../../apps/web/src/**/*.{rs,html}",
    "../../crates/ui/**/src/**/*.{rs,html}",
  ],
  theme: {
    extend: {
      colors: {
        // Brand colors from GOOD CONTROL branding
        brand: {
          navy: {
            DEFAULT: '#1e293b',
            light: '#334155',
            dark: '#0f172a',
          },
          mint: {
            DEFAULT: '#6ee7b7',
            light: '#a7f3d0',
            dark: '#5eead4',
          },
          coral: {
            DEFAULT: '#fb923c',
            light: '#fdba74',
            dark: '#f97316',
          },
        },
      },
    },
  },
  plugins: [],
  // Ensure all Tailwind utilities are included
  safelist: [
    // Common utility classes used in components
    'bg-gray-50', 'bg-gray-100', 'bg-gray-200', 'bg-white',
    'text-gray-600', 'text-gray-700', 'text-gray-800', 'text-gray-900',
    'bg-blue-600', 'bg-green-50', 'bg-yellow-50', 'bg-red-50',
    'text-blue-800', 'text-green-800', 'text-yellow-800', 'text-red-800',
    'border-blue-200', 'border-green-200', 'border-yellow-200', 'border-red-200',
    // Brand colors
    'bg-brand-navy', 'bg-brand-navy-light', 'bg-brand-navy-dark',
    'bg-brand-mint', 'bg-brand-mint-light', 'bg-brand-mint-dark',
    'bg-brand-coral', 'bg-brand-coral-light', 'bg-brand-coral-dark',
    'text-brand-navy', 'text-brand-navy-light', 'text-brand-navy-dark',
    'text-brand-mint', 'text-brand-mint-light', 'text-brand-mint-dark',
    'text-brand-coral', 'text-brand-coral-light', 'text-brand-coral-dark',
    'border-brand-navy', 'border-brand-navy-light', 'border-brand-navy-dark',
    'border-brand-mint', 'border-brand-mint-light', 'border-brand-mint-dark',
    'border-brand-coral', 'border-brand-coral-light', 'border-brand-coral-dark',
  ],
};

