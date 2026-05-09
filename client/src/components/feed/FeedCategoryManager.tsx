import { useState } from "react";
import { $api } from "#/lib/api";
import { Button } from "../ui/button";
import { Plus, X, Settings2, Trash2 } from "lucide-react";
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from "../ui/dialog";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../ui/select";
import { useQueryClient } from "@tanstack/react-query";
import { Input } from "../ui/input";
import { Label } from "../ui/label";

export function FeedCategoryManager({ feedId }: { feedId: string }) {
    const queryClient = useQueryClient();
    const [open, setOpen] = useState(false);

    const { data: feedCategories, refetch } = $api.useQuery("get", "/feed/{feed_id}/categories", {
        params: { path: { feed_id: feedId } }
    });

    const { data: allCategories } = $api.useQuery("get", "/category");

    const { mutate: addCategory } = $api.useMutation("post", "/feed/{feed_id}/categories");
    const { mutate: removeCategory } = $api.useMutation("delete", "/feed/{feed_id}/categories/{category_id}");
    const { mutate: updateCategory } = $api.useMutation("put", "/feed/{feed_id}/categories/{category_id}");
    const { mutate: rescheduleFeed, isPending: isRescheduling } = $api.useMutation("post", "/workers/reschedule");

    const [selectedCategoryId, setSelectedCategoryId] = useState<string>("");
    const [override, setOverride] = useState<string>("");

    const handleAdd = () => {
        if (!selectedCategoryId) return;
        addCategory({
            params: { path: { feed_id: feedId } },
            body: {
                feedId,
                categoryId: selectedCategoryId,
                modelDescriptionOverride: override || null
            }
        }, {
            onSuccess: () => {
                setSelectedCategoryId("");
                setOverride("");
                refetch();
                queryClient.invalidateQueries({ queryKey: ["get", "/articles/categories"] });
            }
        });
    };

    const handleRemove = (categoryId: string) => {
        if (confirm("Remove this category from the feed?")) {
            removeCategory({
                params: { path: { feed_id: feedId, category_id: categoryId } }
            }, {
                onSuccess: () => {
                    refetch();
                    queryClient.invalidateQueries({ queryKey: ["get", "/articles/categories"] });
                }
            });
        }
    };

    const unusedCategories = allCategories?.filter(
        cat => !feedCategories?.some(fc => fc.categoryId === cat.id)
    ) || [];

    return (
        <Dialog open={open} onOpenChange={setOpen}>
            <DialogTrigger asChild>
                <Button variant="outline" size="sm">
                    <Settings2 className="size-4 mr-2" />
                    Manage Categories
                </Button>
            </DialogTrigger>
            <DialogContent className="sm:max-w-[500px] max-h-[90vh] flex flex-col">
                <DialogHeader>
                    <DialogTitle>Manage Feed Categories</DialogTitle>
                </DialogHeader>
                
                <div className="space-y-6 py-4 overflow-y-auto pr-2 custom-scrollbar">
                    <div className="space-y-4">
                        <Label>Add Category</Label>
                        <div className="flex gap-2">
                            <Select value={selectedCategoryId} onValueChange={setSelectedCategoryId}>
                                <SelectTrigger className="flex-1">
                                    <SelectValue placeholder="Select a category" />
                                </SelectTrigger>
                                <SelectContent>
                                    {unusedCategories.map(cat => (
                                        <SelectItem key={cat.id} value={cat.id!}>
                                            {cat.humanName}
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </div>
                        <div className="space-y-2">
                            <Label className="text-xs">Model Description Override (Optional)</Label>
                            <Input 
                                placeholder="Custom instructions for ML model" 
                                value={override}
                                onChange={(e) => setOverride(e.target.value)}
                            />
                        </div>
                        <Button onClick={handleAdd} disabled={!selectedCategoryId} className="w-full">
                            <Plus className="size-4 mr-2" /> Add to Feed
                        </Button>
                    </div>

                    <div className="space-y-2">
                        <Label>Current Categories</Label>
                        <div className="border rounded-md divide-y">
                            {feedCategories?.map(fc => {
                                const cat = allCategories?.find(c => c.id === fc.categoryId);
                                return (
                                    <div key={fc.categoryId} className="p-3 flex justify-between items-center">
                                        <div className="flex-1 mr-4">
                                            <div className="font-medium text-sm">{cat?.humanName}</div>
                                            <div className="flex flex-col gap-1 mt-1">
                                                <Label className="text-[10px] text-muted-foreground">Model Override (Saves on blur/Enter)</Label>
                                                <Input 
                                                    className="h-7 text-xs" 
                                                    placeholder={cat?.modelDescription || "Model override"} 
                                                    defaultValue={fc.modelDescriptionOverride || ""}
                                                    onKeyDown={(e) => {
                                                        if (e.key === 'Enter') {
                                                            e.currentTarget.blur();
                                                        }
                                                    }}
                                                    onBlur={(e) => {
                                                        const newVal = e.target.value || null;
                                                        if (newVal !== fc.modelDescriptionOverride) {
                                                            updateCategory({
                                                                params: { path: { feed_id: feedId, category_id: fc.categoryId } },
                                                                body: { ...fc, modelDescriptionOverride: newVal }
                                                            }, { onSuccess: () => refetch() });
                                                        }
                                                    }}
                                                />
                                            </div>
                                        </div>
                                        <Button 
                                            variant="ghost" 
                                            size="icon-xs" 
                                            onClick={() => handleRemove(fc.categoryId)}
                                            className="text-destructive hover:text-destructive hover:bg-destructive/10"
                                        >
                                            <Trash2 className="size-4" />
                                        </Button>
                                    </div>
                                );
                            })}
                            {feedCategories?.length === 0 && (
                                <div className="p-4 text-center text-sm text-muted-foreground italic">
                                    No categories assigned to this feed.
                                </div>
                            )}
                        </div>
                    </div>
                </div>

                <div className="pt-4 border-t mt-auto">
                    <Button 
                        variant="secondary" 
                        className="w-full"
                        disabled={isRescheduling}
                        onClick={() => {
                            if (confirm("This will reschedule all articles in this feed for categorization and sentiment analysis. Continue?")) {
                                rescheduleFeed({
                                    body: {
                                        feed_uuid: feedId,
                                        tasks: ["Categorize", "SentimentalAnalysis"],
                                        missing_only: false
                                    }
                                }, {
                                    onSuccess: () => {
                                        alert("Rescheduling started successfully.");
                                    }
                                });
                            }
                        }}
                    >
                        {isRescheduling ? "Rescheduling..." : "Reschedule all feed articles"}
                    </Button>
                </div>
            </DialogContent>
        </Dialog>
    );
}
