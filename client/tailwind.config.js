/** @type {import('tailwindcss').Config} */
export default {
  darkMode: ["class", '[data-kb-theme="dark"]'],
  content: ["./src/**/*.{html,js,jsx,md,mdx,ts,tsx}"],
  presets: [require("./ui.preset.js")],
};
