import { useEffect } from 'react'
import { Outlet, useLocation } from 'react-router-dom'
import Header from './Header.jsx'
import Footer from './Footer.jsx'
import AnimatedBackground from '../effects/AnimatedBackground.jsx'
import useScrollY from '../../hooks/useScrollY.js'

export default function AppLayout() {
  const { pathname } = useLocation()

  // Publish `--scroll-y` / `--scroll-progress` on <html> so the ambient
  // background (and any other scroll-reactive decorations) can read them
  // straight from CSS without each component wiring up its own listener.
  // Mounted once here at the layout root.
  useScrollY()

  // Reset scroll on route change so each page lands at the top.
  useEffect(() => {
    window.scrollTo({ top: 0, behavior: 'instant' })
  }, [pathname])

  return (
    <div className="relative min-h-screen flex flex-col bg-surface text-on-surface">
      {/* Mounted at layout root so every route gets the same ambient
          background without each page re-mounting it on navigation. */}
      <AnimatedBackground />
      <Header />
      <main className="pt-16 flex-1">
        <Outlet />
      </main>
      <Footer />
    </div>
  )
}
