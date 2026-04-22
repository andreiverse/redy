import { $api } from "#/lib/api";
import { type components } from "#/lib/api/v1";
import { Link } from "@tanstack/react-router";
import { Card, CardDescription, CardHeader, CardTitle } from "../ui/card";
export function ArticleCard({
    article,
    sentimentScore
}: {
    sentimentScore?: number | null,
    article: components["schemas"]["ArticleDto"]
}) {
    return <>
        <Card className={`${(sentimentScore ?? 0) < 0 ? 'border-l-solid border-l-red-500 border-l-2' : ''} ${(sentimentScore ?? 0) > 0 ? "border-l-solid border-l-green-500 border-l-2" : ""}`}>
            <CardHeader>
                <CardTitle>{article.title}</CardTitle>
                <CardDescription>Published at {article.publishedAt}</CardDescription>
                {article.feedDescription && <CardDescription>{article.feedDescription}</CardDescription>}
                <Link to={"/reader?article_uuid=" + article.id}>Read</Link>
            </CardHeader>
        </Card>
    </>
}

export function FeedArticleList({ feedUuid }: { feedUuid: string | null }) {
    const feedsQuery = $api.useQuery("get", "/articles", {
        params: {
            query: { feed_uuid: feedUuid }
        }
    });

    if (feedsQuery.isLoading) {
        return <>Loading...</>;
    }

    if (feedsQuery.isError || !feedsQuery.isSuccess) {
        return <>Error: {feedsQuery.error}</>;
    }

    const sorted = [...feedsQuery.data].sort(
        (a, b) => new Date(b.article.publishedAt ?? Date.now()).getTime() - new Date(a.article.publishedAt ?? Date.now()).getTime()
    );

    return <>
        <div>
            <div className="space-y-2">
                {
                    sorted.map(article => <ArticleCard key={article.article.link} sentimentScore={article.sentimentScore} article={article.article} />)
                }
            </div>
        </div>
    </>;
}