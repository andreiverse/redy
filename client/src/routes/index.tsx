import { createFileRoute, Link } from '@tanstack/react-router'
import { $api } from '#/lib/api'
import { Heart } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { FeedList } from '#/components/feed/FeedList'

export const Route = createFileRoute('/')({
  component: Index
})

function Index() {
  const { data: user } = $api.useQuery('get', '/auth/me', undefined, { retry: false })
  const { data: favorites } = $api.useQuery('get', '/favorites', undefined, { enabled: !!user })

  return (
    <div className="flex flex-col items-center justify-center min-h-[60vh] text-center space-y-6 py-8">
      <div className="space-y-2">
        <h1 className="text-4xl font-bold tracking-tighter sm:text-5xl">Welcome to Redy</h1>
        <p className="text-muted-foreground max-w-[600px] md:text-xl">
          Your personal RSS reader. Stay updated with your favorite feeds in one place.
        </p>
      </div>

      <div className="flex flex-col sm:flex-row gap-4">
        {user ? (
          favorites && favorites.length > 0 ? (
            <Button asChild size="lg" className="hidden md:flex">
              <Link to="/feed/$feedId" params={{ feedId: favorites[0].id }}>
                <Heart className="mr-2 size-4 fill-current" />
                Go to your first favorite
              </Link>
            </Button>
          ) : (
            <div className="text-muted-foreground italic hidden md:block">
              Select a feed from the sidebar to start reading
            </div>
          )
        ) : (
          <div className="text-muted-foreground hidden md:block">
            Login to see your favorites and personalized feeds
          </div>
        )}
      </div>

      <div className="w-full md:hidden text-left bg-card border rounded-lg shadow-sm p-4 mt-8">
        <h2 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground mb-4 px-2">Your Feeds</h2>
        <FeedList className="max-h-[60vh]" />
      </div>
    </div>
  )
}
