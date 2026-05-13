import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// Tailwind runs through PostCSS (see postcss.config.js + tailwind.config.js),
// so no extra Vite plugin is needed here.
export default defineConfig({
  plugins: [react()],
})
