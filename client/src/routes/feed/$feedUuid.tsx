import { FeedArticleList } from '#/components/rss/FeedArticles';
import { FeedList } from '#/components/rss/FeedList';
import { Button } from '#/components/ui/button';
import { Input } from '#/components/ui/input';
import { createFileRoute } from '@tanstack/react-router'
import { useState } from 'react'

export const Route = createFileRoute('/feed/$feedUuid')({ component: RssFeed })

function RssFeed() {
  const { feedUuid } = Route.useParams()

  return (
    <>
     <FeedArticleList rssFeedUuid={feedUuid} /> 
    </>
  )
}
