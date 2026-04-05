import { $api } from "#/lib/api";
import { type components } from "#/lib/api/v1";
import { Link } from "@tanstack/react-router";
import { Card, CardHeader, CardTitle } from "../ui/card";
import { Button } from "../ui/button";

export function FeedList({
    selected, setSelected
}: {
    selected: string | null,
    setSelected: (selected: string | null) => void
}) {
    const feedsQuery = $api.useQuery("get", "/feed");

    if (feedsQuery.isLoading) {
        return <>Loading...</>;
    }

    if (feedsQuery.isError || !feedsQuery.isSuccess) {
        return <>Error: {feedsQuery.error}</>;
    }

    return <>
        <div className="grid grid-cols-3 space gap-2 mb-2">
            {
                feedsQuery.data.map(feed =>
                    <Card key={feed.id} className="h-full">
                        <CardHeader>
                            <CardTitle>{feed.url}</CardTitle>
                            {
                                selected == feed.id ? <Button onClick={() => setSelected(null)}>Remove filter</Button> :
                                    <Button onClick={() => setSelected(feed.id)}>Filter by this</Button>
                            }

                        </CardHeader>
                    </Card>

                )
            }
        </div>
    </>;
}