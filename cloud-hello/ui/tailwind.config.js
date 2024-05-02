/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./**/*.ts",
  ],
  theme: {
    extend: {
      fontFamily: {
        script: ['Permanent Marker', 'cursive']
      },
      animation: {
        'enter': 'enter 1s ease-in-out both',
        'exit': 'exit 1s ease-in-out both',
      },
      keyframes: {
        enter: {
          '0%': { transform: 'translateY(2rem)', opacity: 0 },
          '100%': { transform: 'translateY(0rem)', opacity: 1 },
        },
        exit: {
          '0%': { transform: 'translateY(0rem)', opacity: 1 },
          '100%': { transform: 'translateY(-2rem)', opacity: 0 },
        }
      }
    },
  },
  plugins: [],
}

