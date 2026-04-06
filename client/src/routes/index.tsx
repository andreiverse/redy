import { FeedArticleList } from '#/components/rss/FeedArticles';
import { FeedList } from '#/components/rss/FeedList';
import { createFileRoute } from '@tanstack/react-router'
import { useState } from 'react'

export const Route = createFileRoute('/')({ component: App })

function App() {
  const [selectedFeedUuid, setSelectedFeedUuid] = useState<string | null>(null);

  return (
    <>
      <FeedList selected={selectedFeedUuid} setSelected={setSelectedFeedUuid} /> 
      <FeedArticleList feedUuid={selectedFeedUuid} />
    </>
  )
}
