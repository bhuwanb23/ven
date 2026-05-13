import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// Tailwind runs through PostCSS (see postcss.config.js + tailwind.config.js),
// so no Vite plugin is needed beyond @vitejs/plugin-react.
export default defineConfig({
  plugins: [react()],
})
