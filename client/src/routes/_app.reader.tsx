import { Button } from '#/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '#/components/ui/card';
import { $api } from '#/lib/api'
import { createFileRoute } from '@tanstack/react-router'
import { z } from "zod";
import { RescheduleArticleDialog } from '#/components/article/RescheduleArticleDialog';

const searchSchema = z.object({
  article_uuid: z.string(),
})

export const Route = createFileRoute('/_app/reader')({
  component: RouteComponent,
  validateSearch: searchSchema
})

function RouteComponent() {
  const { article_uuid } = Route.useSearch();

  const { data: user } = $api.useQuery('get', '/auth/me', undefined, {
    retry: false
  });
  const articleQuery = $api.useQuery("get", "/articles/{article_uuid}", {
    params: {
      path: {
        article_uuid
      }
    }
  })

  const allCategoriesQuery = $api.useQuery("get", "/category", undefined, {
    staleTime: Infinity,
  });

  const categoryMap = allCategoriesQuery.data?.reduce((acc, cat) => {
    if (cat.id) acc[cat.id] = cat.humanName;
    return acc;
  }, {} as Record<string, string>) || {};

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

      <Card className='flex flex-col mt-2'>
        <CardHeader>

          <CardTitle className='text-3xl'>{articleQuery.data.article.title}</CardTitle>
          <CardDescription className='flex flex-col gap-2'>
            {
              articleQuery.data.sentimentScore && <span>Sentimental score: {articleQuery.data.sentimentScore}</span>
            }
            {
              articleQuery.data.categoryId && categoryMap[articleQuery.data.categoryId] && <span>Category: {categoryMap[articleQuery.data.categoryId]}</span>
            }
            {
              <span>Language: {articleQuery.data.article.language}</span>
            }
            <div>
              {user?.isAdmin && <RescheduleArticleDialog articleUuid={article_uuid} />}
            </div>
          </CardDescription>
        </CardHeader>
        <CardContent>

          <div className='text-justify news-content' dangerouslySetInnerHTML={{ __html: articleQuery.data.article.htmlContent }} />
        </CardContent>
      </Card>
    </div>
  </div>
}
