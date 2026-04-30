import { $api } from "#/lib/api";
import { type components } from "#/lib/api/v1";
import { Link, useNavigate } from "@tanstack/react-router";
import { Card, CardDescription, CardHeader, CardTitle } from "../ui/card";
import { Button } from "../ui/button";
import { Trash2, AlertCircle, Clock } from "lucide-react";
import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { Badge } from "../ui/badge";

export function ArticleCard({
    article,
    sentimentScore,
    category
}: {
    sentimentScore?: number | null,
    category?: string | null,
    article: components["schemas"]["ArticleDto"]
}) {
    const now = new Date();
    const fetchedDate = new Date(article.fetchedAt);
    const pubDate = article.publishedAt ? new Date(article.publishedAt) : null;

    const getDaysAgo = (date: Date) => {
        const diffInMs = now.getTime() - date.getTime();
        return Math.floor(diffInMs / (1000 * 60 * 60 * 24));
    };

    const formatDaysAgo = (days: number) => {
        if (days <= 0) return "today";
        if (days === 1) return "yesterday";
        return `${days} days ago`;
    };

    const fetchedDaysAgo = getDaysAgo(fetchedDate);
    const pubDaysAgo = pubDate ? getDaysAgo(pubDate) : null;

    const isStale = (pubDaysAgo !== null ? pubDaysAgo : fetchedDaysAgo) > 2;

    return <>
        <Card className={`relative ${(sentimentScore ?? 0) < 0 ? 'border-l-solid border-l-red-500 border-l-2' : ''} ${(sentimentScore ?? 0) > 0 ? "border-l-solid border-l-green-500 border-l-2" : ""}`}>
            <CardHeader>
                <div className="flex justify-between items-start gap-2">
                    <div className="flex-1 space-y-1">
                        <CardTitle className="text-lg leading-snug">{article.title}</CardTitle>
                        <div className="flex items-center gap-x-2 gap-y-1 flex-wrap text-xs text-muted-foreground">
                            {pubDate && (
                                <div className="flex items-center gap-1">
                                    <Clock className="size-3" />
                                    <span>Published {formatDaysAgo(pubDaysAgo!)}</span>
                                </div>
                            )}
                            {(pubDate && fetchedDaysAgo > 0) && <span>•</span>}
                            <div className={`flex items-center gap-1 ${isStale && !pubDate ? 'text-destructive font-medium' : ''}`}>
                                {isStale && !pubDate && <AlertCircle className="size-3" />}
                                <span>Found {formatDaysAgo(fetchedDaysAgo)}</span>
                            </div>
                            {isStale && pubDate && (
                                <div className="flex items-center gap-1 text-destructive font-medium">
                                    <span>•</span>
                                    <AlertCircle className="size-3" />
                                    <span>Old content ({">"}2 days)</span>
                                </div>
                            )}
                        </div>
                    </div>
                    {category && (
                        <Badge variant="secondary" className="shrink-0">
                            {category}
                        </Badge>
                    )}
                </div>
                {article.feedDescription && <CardDescription className="line-clamp-2">{article.feedDescription}</CardDescription>}
                <Link to={"/reader?article_uuid=" + article.id} className="text-sm text-primary hover:underline mt-2 inline-block font-medium">Read Article</Link>
            </CardHeader>
        </Card>
    </>
}

export function FeedArticleList({ feedUuid, initialCategory }: { feedUuid: string | null, initialCategory?: string }) {
    const queryClient = useQueryClient();
    const navigate = useNavigate();
    const [selectedCategory, setSelectedCategory] = useState<string>(initialCategory || "all");

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

    const categoriesQuery = $api.useQuery("get", "/articles/categories", {
        params: {
            query: {
                feed_uuid: feedUuid ? feedUuid : undefined
            }
        }
    });

    useEffect(() => {
        if (initialCategory) {
            setSelectedCategory(initialCategory);
        } else {
            setSelectedCategory("all");
        }
    }, [initialCategory]);

    const feedsQuery = $api.useQuery("get", "/articles", {
        params: {
            query: {
                feed_uuid: feedUuid ? feedUuid : undefined,
                category: selectedCategory === "all" ? undefined : selectedCategory
            }
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

    const canManage = user && feedQuery.data && (feedQuery.data.ownerUuid === user.id || user.isAdmin);

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

    const handleCategoryChange = (category: string) => {
        setSelectedCategory(category);
        navigate({
            search: (prev: any) => ({
                ...prev,
                category: category === "all" ? undefined : category
            })
        } as any);
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
                    {canManage && (
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

            {
                (categoriesQuery.data?.length ?? 0) > 0 &&
                <div className="mb-6 flex flex-wrap gap-2 items-center">
                    <span className="text-sm font-medium mr-2">Categories:</span>
                    <Button
                        variant={selectedCategory === "all" ? "default" : "outline"}
                        size="sm"
                        onClick={() => handleCategoryChange("all")}
                        className="rounded-full h-7 px-3 text-xs"
                    >
                        All
                    </Button>
                    {categoriesQuery.data?.map(category => (
                        <Button
                            key={category}
                            variant={selectedCategory === category ? "default" : "outline"}
                            size="sm"
                            onClick={() => handleCategoryChange(category)}
                            className="rounded-full h-7 px-3 text-xs"
                        >
                            {category}
                        </Button>
                    ))}
                </div>}

            <div className="space-y-2">
                {
                    sorted.map(article => (
                        <ArticleCard
                            key={article.article.link}
                            sentimentScore={article.sentimentScore}
                            category={article.category}
                            article={article.article}
                        />
                    ))
                }
            </div>
        </div>
    </>;
}
