import { Route, Routes } from 'react-router-dom'
import AppLayout from './components/layout/AppLayout.jsx'
import Landing from './pages/Landing.jsx'
import Install from './pages/Install.jsx'
import Languages from './pages/Languages.jsx'
import Changelog from './pages/Changelog.jsx'
import Playground from './pages/Playground.jsx'
import NotFound from './pages/NotFound.jsx'
import DocsLayout from './pages/docs/DocsLayout.jsx'
import DocsHub from './pages/docs/DocsHub.jsx'
import DocPage from './pages/docs/DocPage.jsx'

export default function App() {
  return (
    <Routes>
      <Route element={<AppLayout />}>
        <Route index element={<Landing />} />
        <Route path="install" element={<Install />} />
        <Route path="languages" element={<Languages />} />
        <Route path="changelog" element={<Changelog />} />
        <Route path="playground" element={<Playground />} />
        <Route path="docs" element={<DocsLayout />}>
          <Route index element={<DocsHub />} />
          <Route path=":slug" element={<DocPage />} />
        </Route>
        <Route path="*" element={<NotFound />} />
      </Route>
    </Routes>
  )
}
