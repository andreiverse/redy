import { createFileRoute, Outlet } from '@tanstack/react-router'
import { FeedList } from '../components/feed/FeedList'

export const Route = createFileRoute('/_app')({
  component: AppLayout,
})

function AppLayout() {
  return (
    <div className="flex gap-6">
      <aside className="w-64 shrink-0 hidden md:block">
        <div className="sticky top-24 h-[calc(100vh-8rem)]">
          <FeedList className="h-full" />
        </div>
      </aside>
      <main className="flex-1 min-w-0">
        <Outlet />
      </main>
    </div>
  )
}
