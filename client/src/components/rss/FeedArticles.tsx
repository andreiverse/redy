import { $api } from "#/lib/api";
import { type components } from "#/lib/api/v1";
import { Link } from "@tanstack/react-router";
import { Card, CardDescription, CardHeader, CardTitle } from "../ui/card";
export function ArticleCard({
    article
}: {
    article: components["schemas"]["ArticleDto"]
}) {
    return <>
        <Card>
            <CardHeader>
                <CardTitle>{article.title}</CardTitle>
                {article.feedDescription && <CardDescription>{article.feedDescription}</CardDescription>}
                <Link to={"/reader?url=" + article.link}>Read</Link>
            </CardHeader>
        </Card>
    </>
}

export function FeedArticleList({ feedUuid }: { feedUuid: string }) {
    const feedsQuery = $api.useQuery("get", "/feed/{feed_uuid}/fetch", {
        params: {
            path: { feed_uuid: feedUuid }
        }
    });


    if (feedsQuery.isLoading) {
        return <>Loading...</>;
    }

    if (feedsQuery.isError || !feedsQuery.isSuccess) {
        return <>Error: {feedsQuery.error}</>;
    }

    const sorted = [...feedsQuery.data].sort(
        (a, b) => new Date(b.publishedAt ?? Date.now()).getTime() - new Date(a.publishedAt ?? Date.now()).getTime()
    );

    return <>
        <div className="space-y-2">
            {
                sorted.map(article => <ArticleCard key={article.link} article={article} />)
            }
        </div>
    </>;
}