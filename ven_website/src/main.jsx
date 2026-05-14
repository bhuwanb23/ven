import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { BrowserRouter } from 'react-router-dom'
import './index.css'
import App from './App.jsx'

// Vite injects `import.meta.env.BASE_URL` based on the `base` option in
// vite.config.js (`/ven/` in production, `/` in dev). React Router needs the
// same value as `basename` so deep links and `<Link>` URLs round-trip cleanly
// when hosted at https://bhuwanb23.github.io/ven/.
const basename = import.meta.env.BASE_URL.replace(/\/$/, '') || '/'

createRoot(document.getElementById('root')).render(
  <StrictMode>
    <BrowserRouter basename={basename}>
      <App />
    </BrowserRouter>
  </StrictMode>
)
