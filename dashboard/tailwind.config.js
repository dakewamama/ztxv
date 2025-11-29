/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'zcash-green': '#00FF94',
        'deep-black': '#0A0A0A',
      },
    },
  },
  plugins: [],
}
