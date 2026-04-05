import { $api } from '#/lib/api'
import { createFileRoute } from '@tanstack/react-router'
import { z } from "zod";

const searchSchema = z.object({
  article_uuid: z.string(),
})

export const Route = createFileRoute('/reader')({
  component: RouteComponent,
  validateSearch: searchSchema
})

function RouteComponent() {
  const { article_uuid } = Route.useSearch();

  const articleQuery = $api.useQuery("get", "/articles/{article_uuid}", {
    params: {
      path: {
        article_uuid
      }
    }
  })

  if (articleQuery.isLoading) {
    return <>Loading...</>
  }

  if (articleQuery.isError || !articleQuery.isSuccess) {
    return <>Error: {articleQuery.error}</>
  }

  if (articleQuery.data.htmlContent == null) {
    return <>Article could not be fetched, go to link: <a target='_blank' href={articleQuery.data.link}>click</a></>
  }

  return <div>
    <div>
      <div className='text-justify news-content' dangerouslySetInnerHTML={{ __html: articleQuery.data.htmlContent }} />
    </div>
  </div>
}
