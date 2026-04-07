import { $api } from "#/lib/api";
import { Card, CardHeader, CardTitle } from "../ui/card";

export function FeedList({
  selected,
  setSelected
}: {
  selected: string | null,
  setSelected: (selected: string | null) => void
}) {
  const feedsQuery = $api.useQuery("get", "/feed");

  if (feedsQuery.isLoading) {
    return <div className="text-center py-8">Loading...</div>;
  }

  if (feedsQuery.isError || !feedsQuery.isSuccess) {
    return <div className="text-red-500 text-center py-8">Error: {feedsQuery.error}</div>;
  }

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 mb-4">
      {feedsQuery.data.map((feed) => {
        const isSelected = selected === feed.id;

        return (
          <Card
            key={feed.id}
            className={`h-full cursor-pointer transition-transform duration-150
              ${isSelected ? "bg-blue-100 shadow-lg" : "hover:shadow-md hover:scale-105"}`}
            onClick={() => setSelected(isSelected ? null : feed.id)}
          >
            <CardHeader className="flex flex-col gap-2">
              <CardTitle className={isSelected ? "text-black" : "text-white"}>{feed.name}</CardTitle>
              <div className="text-sm text-gray-500">
                {isSelected ? "Click to remove filter" : "Click to filter by this"}
              </div>
            </CardHeader>
          </Card>
        );
      })}
    </div>
  );
}