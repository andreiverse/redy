import { $api } from '#/lib/api'
import { createFileRoute } from '@tanstack/react-router'
import { z } from "zod";

const searchSchema = z.object({
  url: z.string(),
})

export const Route = createFileRoute('/reader')({
  component: RouteComponent,
  validateSearch: searchSchema
})

function RouteComponent() {
  const { url } = Route.useSearch();

  const articleQuery = $api.useQuery("get", "/reader", {
    params: {
      query: {
        url
      }
    }
  })

  if (articleQuery.isLoading) {
    return <>Loading...</>
  }

  if (articleQuery.isError || !articleQuery.isSuccess) {
    return <>Error: {articleQuery.error}</>
  }

  return <div>
    <div>
      <div className='text-justify news-content' dangerouslySetInnerHTML={{ __html: articleQuery.data.html_content }} />
    </div>
  </div>
}
