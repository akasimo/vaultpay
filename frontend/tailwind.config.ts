import type { Config } from "tailwindcss";

const config: Config = {
  content: [
    "./pages/**/*.{js,ts,jsx,tsx,mdx}",
    "./components/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/app/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  theme: {
    extend: {
      colors: {
        background: "var(--background)",
        foreground: "var(--foreground)",
        cardBackground: "var(--card-background)",
        accent: "var(--accent)",
        primary: "#10b981", // Standardizing to green
      },
      borderRadius: {
        xl: "1rem",
      },
      boxShadow: {
        card: "0 4px 12px rgba(0, 0, 0, 0.3)",
      },
      fontFamily: {
        sans: ['"Geist Sans"', 'Arial', 'Helvetica', 'sans-serif'],
        mono: ['"Geist Mono"', 'Courier New', 'Courier', 'monospace'],
      },
    },
  },
  plugins: [],
};

export default config;