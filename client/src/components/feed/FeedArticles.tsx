import { $api } from "#/lib/api";
import { type components } from "#/lib/api/v1";
import { Link, useNavigate } from "@tanstack/react-router";
import { Card, CardDescription, CardHeader, CardTitle } from "../ui/card";
import { Button } from "../ui/button";
import { Trash2 } from "lucide-react";
import { useQueryClient } from "@tanstack/react-query";

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
    const queryClient = useQueryClient();
    const navigate = useNavigate();

    const { data: user } = $api.useQuery('get', '/auth/me', undefined, {
        retry: false
    });

    const feedQuery = $api.useQuery("get", "/feed/{feed_uuid}", {
        params: {
            path: {
                feed_uuid: feedUuid || ""
            }
        }
    }, {
        enabled: !!feedUuid
    });

    const feedsQuery = $api.useQuery("get", "/articles", {
        params: {
            query: { feed_uuid: feedUuid }
        }
    });

    const deleteFeedMutation = $api.useMutation("delete", "/feed/{feed_uuid}", {
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["get", "/feed"] });
            queryClient.invalidateQueries({ queryKey: ["get", "/favorites"] });
            navigate({ to: "/" });
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

    const isOwner = user && feedQuery.data && feedQuery.data.ownerUuid === user.id;

    const handleDelete = () => {
        if (feedUuid && window.confirm("Are you sure you want to delete this feed?")) {
            deleteFeedMutation.mutate({
                params: {
                    path: {
                        feed_uuid: feedUuid
                    }
                }
            });
        }
    };

    return <>
        <div>
            {
                feedQuery.data && <div className="mb-6 flex justify-between items-start bg-card border rounded-lg p-4">
                    <div className="space-y-1">
                        <h2 className="text-xl font-bold">{feedQuery.data.name}</h2>
                        <p className="text-sm text-muted-foreground break-all">{feedQuery.data.url}</p>
                        <p className="text-xs text-muted-foreground mt-2">
                            Last fetch: {feedQuery.data.fetchedSecondsAgo ? feedQuery.data.fetchedSecondsAgo.toFixed(2) + ' seconds ago' : "never"}
                        </p>
                    </div>
                    {isOwner && (
                        <Button 
                            variant="destructive" 
                            size="icon" 
                            onClick={handleDelete}
                            disabled={deleteFeedMutation.isPending}
                            title="Delete feed"
                        >
                            <Trash2 className="size-4" />
                        </Button>
                    )}
                </div>
            }

            <div className="space-y-2">
                {
                    sorted.map(article => <ArticleCard key={article.article.link} sentimentScore={article.sentimentScore} article={article.article} />)
                }
            </div>
        </div>
    </>;
}