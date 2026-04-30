import { createFileRoute, Outlet, Link } from '@tanstack/react-router'
import { $api } from '../lib/api'
import { LayoutDashboard, Rss, Users, Hash } from 'lucide-react'

export const Route = createFileRoute('/admin')({
  beforeLoad: async () => {
    // We can't easily check on the server without server functions, 
    // so we'll let the component handle the redirect if needed, 
    // or just rely on the API failing if not authorized.
  },
  component: AdminLayout,
})


function AdminLayout() {
  const { data: user, isLoading } = $api.useQuery('get', '/auth/me', undefined, {
    retry: false
  })

  if (isLoading) return <div>Loading...</div>
  
  if (!user?.isAdmin) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[50vh] space-y-4">
        <h1 className="text-2xl font-bold">Unauthorized</h1>
        <p className="text-muted-foreground">You do not have permission to access the admin panel.</p>
        <Link to="/" className="text-primary hover:underline">Return to Home</Link>
      </div>
    )
  }

  return (
    <div className="flex gap-6">
      <aside className="w-64 shrink-0 hidden md:block">
        <div className="sticky top-20 h-[calc(100vh-7rem)] overflow-y-auto pr-2 space-y-1 custom-scrollbar">
          <h2 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-4 px-2">
            Admin Panel
          </h2>
          <Link
            to="/admin"
            className="flex items-center gap-2 px-2 py-1.5 rounded-md text-sm transition-colors hover:bg-accent hover:text-accent-foreground"
            activeProps={{ className: "bg-primary/10 text-primary font-medium" }}
          >
            <LayoutDashboard className="size-4" />
            Dashboard
          </Link>
          <Link
            to="/admin/feeds"
            className="flex items-center gap-2 px-2 py-1.5 rounded-md text-sm transition-colors hover:bg-accent hover:text-accent-foreground"
            activeProps={{ className: "bg-primary/10 text-primary font-medium" }}
          >
            <Rss className="size-4" />
            Feeds
          </Link>
          <Link
            to="/admin/users"
            className="flex items-center gap-2 px-2 py-1.5 rounded-md text-sm transition-colors hover:bg-accent hover:text-accent-foreground"
            activeProps={{ className: "bg-primary/10 text-primary font-medium" }}
          >
            <Users className="size-4" />
            Users
          </Link>
          <Link
            to="/admin/categories"
            className="flex items-center gap-2 px-2 py-1.5 rounded-md text-sm transition-colors hover:bg-accent hover:text-accent-foreground"
            activeProps={{ className: "bg-primary/10 text-primary font-medium" }}
          >
            <Hash className="size-4" />
            Categories
          </Link>
        </div>
      </aside>
      <main className="flex-1 min-w-0">
        <Outlet />
      </main>
    </div>
  )
}
