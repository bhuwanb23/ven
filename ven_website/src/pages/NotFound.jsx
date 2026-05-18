import { Link } from 'react-router-dom'
import Brand from '../components/ui/Brand.jsx'
import Icon from '../components/ui/Icon.jsx'
import Button from '../components/ui/Button.jsx'

export default function NotFound({ title = 'Page not found', sub }) {
  return (
    <div className="min-h-[60vh] flex flex-col items-center justify-center text-center px-margin-mobile py-24">
      <Link to="/" aria-label="ven — home" className="mb-6 opacity-80 hover:opacity-100 transition-opacity">
        <Brand size="lg" wordmark={false} />
      </Link>
      <div className="font-mono text-[120px] leading-none font-bold text-primary-fixed-dim/30 mb-4 tracking-tighter">
        404
      </div>
      <h1 className="font-display-lg text-display-lg text-primary mb-4">{title}</h1>
      <p className="text-on-surface-variant max-w-md mb-8">
        {sub ?? "We couldn't find that page. Maybe one of these will help."}
      </p>

      <div className="flex flex-wrap justify-center gap-3 mb-12">
        <Button to="/">
          <Icon name="home" /> Home
        </Button>
        <Button to="/docs" variant="ghost">
          <Icon name="menu_book" /> Docs
        </Button>
        <Button to="/install" variant="ghost">
          <Icon name="rocket_launch" /> Install
        </Button>
      </div>

      <div className="font-mono text-xs text-on-surface-variant opacity-60">
        Or try{' '}
        <Link to="/changelog" className="text-primary-fixed-dim hover:underline">
          /changelog
        </Link>
        ,{' '}
        <Link to="/languages" className="text-primary-fixed-dim hover:underline">
          /languages
        </Link>
        ,{' '}
        <Link to="/playground" className="text-primary-fixed-dim hover:underline">
          /playground
        </Link>
        .
      </div>
    </div>
  )
}
