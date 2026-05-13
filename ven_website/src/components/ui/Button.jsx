import clsx from 'clsx'
import { Link } from 'react-router-dom'

const VARIANTS = {
  primary:
    'bg-primary-fixed-dim text-on-primary-fixed font-bold shadow-[0_0_10px_rgba(0,219,231,0.4)] hover:shadow-[0_0_20px_rgba(0,219,231,0.5)] active:scale-95',
  ghost:
    'bg-transparent text-primary-fixed-dim border border-primary-fixed-dim/50 hover:bg-primary-fixed-dim/10 active:scale-95',
  subtle:
    'bg-surface-container-high text-on-surface hover:bg-surface-container-highest active:scale-95',
  link: 'text-primary-fixed-dim hover:text-primary underline-offset-4 hover:underline',
}

const SIZES = {
  sm: 'px-3 py-1.5 text-xs',
  md: 'px-4 py-2 text-sm',
  lg: 'px-8 py-3 text-base',
}

// Renders a <Link> when `to` is set, an <a> when `href` is set, and a <button>
// otherwise. All three share the same visual surface.
export default function Button({
  children,
  variant = 'primary',
  size = 'md',
  to,
  href,
  className,
  type = 'button',
  ...rest
}) {
  const cn = clsx(
    'inline-flex items-center justify-center gap-2 rounded-lg transition-all',
    'font-body-base',
    VARIANTS[variant],
    SIZES[size],
    className
  )
  if (to) {
    return (
      <Link to={to} className={cn} {...rest}>
        {children}
      </Link>
    )
  }
  if (href) {
    return (
      <a href={href} className={cn} target={href.startsWith('http') ? '_blank' : undefined} rel={href.startsWith('http') ? 'noreferrer' : undefined} {...rest}>
        {children}
      </a>
    )
  }
  return (
    <button type={type} className={cn} {...rest}>
      {children}
    </button>
  )
}
