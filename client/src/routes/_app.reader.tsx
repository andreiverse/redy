import { $api } from '#/lib/api'
import { createFileRoute } from '@tanstack/react-router'
import { z } from "zod";

const searchSchema = z.object({
  article_uuid: z.string(),
})

export const Route = createFileRoute('/_app/reader')({
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

  if (articleQuery.data.article.htmlContent == null) {
    return <>Article could not be fetched, go to link: <a target='_blank' href={articleQuery.data.article.link}>click</a></>
  }

  return <div>
    <div>
      <div className='text-3xl'>{articleQuery.data.article.title}</div>
      <div className='flex flex-col mt-2'>
        {
          articleQuery.data.sentimentScore && <span>Sentimental score: {articleQuery.data.sentimentScore}</span>
        }
        {
          <span>Language: {articleQuery.data.article.language}</span>
        }
      </div>
      <div className='text-justify news-content' dangerouslySetInnerHTML={{ __html: articleQuery.data.article.htmlContent }} />
    </div>
  </div>
}
