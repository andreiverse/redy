import { Link } from '@tanstack/react-router'
import ThemeToggle from './ThemeToggle'
import { $api, baseUrl } from '../lib/api'
import { Button } from './ui/button'

export default function Header() {
  const { data: user, isLoading } = $api.useQuery('get', '/auth/me', undefined, {
    retry: false
  })

  const handleLogin = () => {
   window.location.href = `${baseUrl}/auth/login?redirect_to_frontend=true`;
  }

  return (
    <header className="sticky top-0 z-50 border-b border-[var(--line)] bg-[var(--header-bg)] px-4 backdrop-blur-lg">
      <nav className="page-wrap flex flex-wrap items-center gap-x-3 gap-y-2 py-3 sm:py-4">
        <h2 className="m-0 flex-shrink-0 text-base font-semibold tracking-tight">
          <Link
            to="/"
            className="inline-flex items-center gap-2 rounded-full border border-[var(--chip-line)] bg-[var(--chip-bg)] px-3 py-1.5 text-sm text-[var(--sea-ink)] no-underline shadow-[0_8px_24px_rgba(30,90,72,0.08)] sm:px-4 sm:py-2"
          >
            Redy 
          </Link>
        </h2>

 
        <div className="order-3 flex w-full flex-wrap items-center gap-x-4 gap-y-1 pb-1 text-sm font-semibold sm:order-2 sm:w-auto sm:flex-nowrap sm:pb-0">
          <Link
            to="/"
            className="nav-link"
            activeProps={{ className: 'nav-link is-active' }}
          >
            Home
          </Link>
          {user && (
            <Link
              to="/profile"
              className="nav-link"
              activeProps={{ className: 'nav-link is-active' }}
            >
              {user.username}
            </Link>
          )}
          {user?.isAdmin && (
            <Link
              to="/admin"
              className="nav-link text-[var(--sea-ink)] hover:text-[var(--sea-ink-hover)]"
              activeProps={{ className: 'nav-link is-active' }}
            >
              Admin
            </Link>
          )}
        </div>

        <div className="ml-auto flex items-center gap-4">
          {!isLoading && (
            !user && (
              <Button 
                variant="outline" 
                size="sm" 
                onClick={handleLogin}
                className="rounded-full border-[var(--chip-line)] text-[var(--sea-ink)] hover:bg-[var(--chip-bg)]"
              >
                Login
              </Button>
            )
          )}
          <ThemeToggle />
        </div>

      </nav>
    </header>
  )
}
