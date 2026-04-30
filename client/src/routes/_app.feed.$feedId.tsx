import { FeedArticleList } from '#/components/feed/FeedArticles';
import { createFileRoute } from '@tanstack/react-router'
import { z } from 'zod'

const searchSchema = z.object({
  category: z.string().optional()
})

export const Route = createFileRoute('/_app/feed/$feedId')({
  component: FeedView,
  validateSearch: (search) => searchSchema.parse(search)
})

function FeedView() {
  const { feedId } = Route.useParams()
  const { category } = Route.useSearch()

  return (
    <div className="space-y-4">
      <FeedArticleList feedUuid={feedId} initialCategory={category} />
    </div>
  )
}
