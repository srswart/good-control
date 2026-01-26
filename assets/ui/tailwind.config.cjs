/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "../../apps/desktop/src/**/*.{rs,html}",
    "../../crates/ui/**/src/**/*.{rs,html}",
  ],
  theme: {
    extend: {},
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
  ],
};

