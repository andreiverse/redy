import { FeedArticleList } from '#/components/feed/FeedArticles';
import { FeedList } from '#/components/feed/FeedList';
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
