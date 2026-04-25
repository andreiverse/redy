import { $api } from "#/lib/api";
import { Button } from "../ui/button";
import { CreateFeedDialog } from "./CreateFeedDialog";
import { useMemo } from "react";
import { Star, Compass, Heart, Hash } from "lucide-react";
import { useQueryClient } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { cn } from "#/lib/utils";

export function FeedList({ className }: { className?: string }) {
  const queryClient = useQueryClient();

  const { data: user } = $api.useQuery('get', '/auth/me', undefined, {
    retry: false
  });

  const favoritesQuery = $api.useQuery("get", "/favorites", undefined, {
    enabled: !!user,
  });

  const allFeedsQuery = $api.useQuery("get", "/feed");

  const favoriteMutation = $api.useMutation("post", "/favorites/{feed_uuid}", {
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["get", "/favorites"] });
    }
  });

  const unfavoriteMutation = $api.useMutation("delete", "/favorites/{feed_uuid}", {
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["get", "/favorites"] });
    }
  });

  const favorites = favoritesQuery.data || [];
  const allFeeds = allFeedsQuery.data || [];
  
  const favoriteIds = useMemo(() => {
    return new Set(favorites.map(f => f.id));
  }, [favorites]);

  const toggleFavorite = (e: React.MouseEvent, feedId: string) => {
    e.preventDefault();
    e.stopPropagation();
    if (favoriteIds.has(feedId)) {
      unfavoriteMutation.mutate({ params: { path: { feed_uuid: feedId } } });
    } else {
      favoriteMutation.mutate({ params: { path: { feed_uuid: feedId } } });
    }
  };

  const otherFeeds = allFeeds.filter(f => !favoriteIds.has(f.id));

  return (
    <nav className={cn("flex flex-col gap-6 overflow-y-auto pr-2 custom-scrollbar", className)}>
      {user && (
        <div>
          <div className="flex items-center justify-between mb-2 px-2">
            <h2 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground flex items-center gap-2">
              <Heart className="size-3 text-red-500" /> Favorites
            </h2>
            <CreateFeedDialog />
          </div>
          <div className="space-y-1">
            {favorites.length > 0 ? (
              favorites.map((feed) => (
                <FeedItem 
                  key={feed.id} 
                  feed={feed} 
                  isFavorite={true} 
                  onToggleFavorite={toggleFavorite} 
                  showFavorite={true}
                />
              ))
            ) : (
              <p className="text-xs text-muted-foreground px-2 py-1">No favorites yet</p>
            )}
          </div>
        </div>
      )}

      <div>
        <div className="flex items-center justify-between mb-2 px-2">
          <h2 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground flex items-center gap-2">
            <Compass className="size-3" /> Explore
          </h2>
        </div>
        <div className="space-y-1">
          {otherFeeds.map((feed) => (
            <FeedItem 
              key={feed.id} 
              feed={feed} 
              isFavorite={false} 
              onToggleFavorite={toggleFavorite} 
              showFavorite={!!user}
            />
          ))}
        </div>
      </div>
    </nav>
  );
}

function FeedItem({ 
  feed, 
  isFavorite, 
  onToggleFavorite,
  showFavorite
}: { 
  feed: any, 
  isFavorite: boolean, 
  onToggleFavorite: (e: React.MouseEvent, id: string) => void,
  showFavorite: boolean
}) {
  return (
    <Link
      to="/feed/$feedId"
      params={{ feedId: feed.id }}
      className="flex items-center justify-between group px-2 py-1.5 rounded-md text-sm transition-colors hover:bg-accent hover:text-accent-foreground active:bg-accent/80"
      activeProps={{ className: "bg-primary/10 text-primary font-medium" }}
    >
      <div className="flex items-center gap-2 overflow-hidden">
        <Hash className="size-3.5 shrink-0 opacity-50" />
        <span className="truncate">{feed.name}</span>
      </div>
      {showFavorite && (
        <Button
          variant="ghost"
          size="icon-xs"
          onClick={(e) => onToggleFavorite(e, feed.id)}
          className={`opacity-0 group-hover:opacity-100 rounded-full h-6 w-6 shrink-0 transition-opacity ${isFavorite ? "text-yellow-500 opacity-100" : "text-muted-foreground hover:text-yellow-500"}`}
        >
          <Star className={`size-3 ${isFavorite ? "fill-current" : ""}`} />
        </Button>
      )}
    </Link>
  );
}
