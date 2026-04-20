import { useState } from "react";
import { $api } from "#/lib/api";

import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "../ui/dialog";

import { Button } from "../ui/button";
import { Input } from "../ui/input";
import { Label } from "../ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../ui/select";
import { useQueryClient } from "@tanstack/react-query";

export function CreateFeedDialog() {
  const queryClient = useQueryClient();

  const [open, setOpen] = useState(false);

  const [form, setForm] = useState({
    url: "",
    name: "",
    defaultLanguage: "en",
    feedType: "rss",
  });

  const createFeedMutation = $api.useMutation("post", "/feed", {
    onSuccess: () => {
      setOpen(false);
      setForm({
        url: "",
        name: "",
        defaultLanguage: "en",
        feedType: "rss",
      });
    },
  });

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();

    await createFeedMutation.mutateAsync({
      body: {
        url: form.url,
        name: form.name,
        defaultLanguage: form.defaultLanguage,
        feedType: form.feedType as any,
      },
    });

    await queryClient.invalidateQueries(
      $api.queryOptions("get", "/feed")
    );
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button>Create Feed</Button>
      </DialogTrigger>

      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create Feed</DialogTitle>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4">
          {/* URL */}
          <div className="space-y-2">
            <Label>URL</Label>
            <Input
              value={form.url}
              onChange={(e) => setForm((p) => ({ ...p, url: e.target.value }))}
              placeholder="https://example.com/rss"
            />
          </div>

          {/* Name */}
          <div className="space-y-2">
            <Label>Name</Label>
            <Input
              value={form.name}
              onChange={(e) => setForm((p) => ({ ...p, name: e.target.value }))}
              placeholder="My Feed"
            />
          </div>

          {/* Language */}
          <div className="space-y-2">
            <Label>Default Language</Label>
            <Input
              value={form.defaultLanguage}
              onChange={(e) =>
                setForm((p) => ({ ...p, defaultLanguage: e.target.value }))
              }
            />
          </div>

          {/* Feed Type */}
          <div className="space-y-2">
            <Label>Feed Type</Label>

            <Select
              value={form.feedType}
              onValueChange={(value) =>
                setForm((p) => ({ ...p, feedType: value }))
              }
            >
              <SelectTrigger>
                <SelectValue placeholder="Select type" />
              </SelectTrigger>

              <SelectContent>
                <SelectItem value="rss">RSS</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <Button
            type="submit"
            disabled={createFeedMutation.isPending}
            className="w-full"
          >
            {createFeedMutation.isPending ? "Creating..." : "Create"}
          </Button>
        </form>
      </DialogContent>
    </Dialog>
  );
}