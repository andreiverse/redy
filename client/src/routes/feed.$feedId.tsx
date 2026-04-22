import { FeedArticleList } from '#/components/feed/FeedArticles';
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/feed/$feedId')({
  component: FeedView
})

function FeedView() {
  const { feedId } = Route.useParams()

  return (
    <div className="space-y-4">
      <FeedArticleList feedUuid={feedId} />
    </div>
  )
}
