import { $api } from "#/lib/api";
import { type components } from "#/lib/api/v1";
import { Link } from "@tanstack/react-router";
import { Card, CardHeader, CardTitle } from "../ui/card";
export function RssFeedCard({
    feed
}: {
    feed: components["schemas"]["RssFeedDto"]
}) {
    return <>
        <Card>
            <CardHeader>
                <CardTitle>{feed.url}</CardTitle>
                <Link to={"/feed/" + feed.id}>Read</Link>
            </CardHeader>
        </Card>
    </>
}

export function FeedList() {
    const feedsQuery = $api.useQuery("get", "/rss_feed");

    if (feedsQuery.isLoading) {
        return <>Loading...</>;
    }

    if (feedsQuery.isError || !feedsQuery.isSuccess) {
        return <>Error: {feedsQuery.error}</>;   
    }

    return <>
        <div>
            {
                feedsQuery.data.map(feed => <RssFeedCard feed={feed} />)
            }
        </div>
    </>;
}