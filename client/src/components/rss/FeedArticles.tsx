import { $api } from "#/lib/api";
import { type components } from "#/lib/api/v1";
import { Link } from "@tanstack/react-router";
import { Button } from "../ui/button";
import { Card, CardAction, CardDescription, CardHeader, CardTitle } from "../ui/card";
export function ArticleCard({
    article
}: {
    article: components["schemas"]["RssNews"]
}) {
    return <>
        <Card>
            <CardHeader>
                <CardTitle>{article.title}</CardTitle>
                {article.description && <CardDescription>{article.description}</CardDescription>}
                <Link to={"/reader?url=" + article.link}>Read</Link>
            </CardHeader>
        </Card>
    </>
}

export function FeedArticleList({ rssFeedUuid }: { rssFeedUuid: string }) {
    const feedsQuery = $api.useQuery("get", "/rss_feed/{rss_feed_uuid}/fetch", {
        params: {
            path: { rss_feed_uuid: rssFeedUuid }
        }
    });

    if (feedsQuery.isLoading) {
        return <>Loading...</>;
    }

    if (feedsQuery.isError || !feedsQuery.isSuccess) {
        return <>Error: {feedsQuery.error}</>;
    }

    const sorted = [...feedsQuery.data].sort(
        (a, b) => new Date(b.published_at ?? Date.now()).getTime() - new Date(a.published_at ?? Date.now()).getTime()
    );

    return <>
        <div className="space-y-2">
            {
                sorted.map(article => <ArticleCard key={article.link} article={article} />)
            }
        </div>
    </>;
}