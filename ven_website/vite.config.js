import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// Tailwind runs through PostCSS (see postcss.config.js + tailwind.config.js),
// so no extra Vite plugin is needed here.
//
// `base` controls the public path the bundle is served from. The site is
// hosted at https://bhuwanb23.github.io/ven/ (a GitHub Pages *project* site),
// so production assets must be served under `/ven/`. In dev we keep `/` so
// `npm run dev` continues to work at http://localhost:5173/.
export default defineConfig(({ command }) => ({
  base: command === 'build' ? '/ven/' : '/',
  plugins: [react()],
}))
