import { useEffect } from 'react'
import { Outlet, useLocation } from 'react-router-dom'
import Header from './Header.jsx'
import Footer from './Footer.jsx'

export default function AppLayout() {
  const { pathname } = useLocation()

  // Reset scroll on route change so each page lands at the top.
  useEffect(() => {
    window.scrollTo({ top: 0, behavior: 'instant' })
  }, [pathname])

  return (
    <div className="min-h-screen flex flex-col bg-surface text-on-surface">
      <Header />
      <main className="pt-16 flex-1">
        <Outlet />
      </main>
      <Footer />
    </div>
  )
}
