import { FeedArticleList } from '#/components/rss/FeedArticles';
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/feed/$feedUuid')({ component: RssFeed })

function RssFeed() {
  const { feedUuid } = Route.useParams()

  return (
    <>
      <FeedArticleList feedUuid={feedUuid} />
    </>
  )
}
