import { $api } from '#/lib/api'
import { createFileRoute, useParams } from '@tanstack/react-router'

export const Route = createFileRoute('/reader')({
  component: RouteComponent,
})

function RouteComponent() {
  const search = Route.useSearch()

  const articleQuery = $api.useQuery("get", "/reader", {
    params: {
      query: {
        url: search.url
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
