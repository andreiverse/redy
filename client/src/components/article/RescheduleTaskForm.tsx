import { useState } from "react";
import { $api } from "#/lib/api";
import { Button } from "../ui/button";
import { Label } from "../ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../ui/select";
import { useQueryClient } from "@tanstack/react-query";
import { type components } from "@/lib/api/v1";

type TaskType = components["schemas"]["TaskType"];

interface RescheduleTaskFormProps {
  articleUuid?: string;
  onSuccess?: () => void;
}

export function RescheduleTaskForm({ articleUuid, onSuccess }: RescheduleTaskFormProps) {
  const queryClient = useQueryClient();
  const [tasks, setTasks] = useState<TaskType[]>(["SentimentalAnalysis", "Categorize"]);
  const [missingOnly, setMissingOnly] = useState(true);
  const [timeRange, setTimeRange] = useState<string>("all");

  const rescheduleMutation = $api.useMutation("post", "/workers/reschedule", {
    onSuccess: () => {
      alert("Successfully scheduled!");
      if (articleUuid) {
        queryClient.invalidateQueries({ queryKey: ["get", "/articles/{article_uuid}"] });
      }
      onSuccess?.();
    },
    onError: (error) => {
      alert("Error: " + JSON.stringify(error));
    }
  });

  const toggleTask = (task: TaskType) => {
    setTasks(prev => 
      prev.includes(task) ? prev.filter(t => t !== task) : [...prev, task]
    );
  };

  const getTimeDate = (range: string): string | null => {
    const now = new Date();
    switch (range) {
      case "24h":
        return new Date(now.getTime() - 24 * 60 * 60 * 1000).toISOString();
      case "7d":
        return new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000).toISOString();
      case "1m":
        return new Date(now.getTime() - 30 * 24 * 60 * 60 * 1000).toISOString();
      case "1y":
        return new Date(now.getTime() - 365 * 24 * 60 * 60 * 1000).toISOString();
      default:
        return null;
    }
  };

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    await rescheduleMutation.mutateAsync({
      body: {
        article_uuid: articleUuid,
        tasks: tasks,
        missing_only: missingOnly,
        from_date: getTimeDate(timeRange),
      },
    });
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div className="space-y-2">
        <Label>Tasks to Run</Label>
        <div className="flex flex-col gap-2">
          {(["Scrape", "SentimentalAnalysis", "Categorize"] as TaskType[]).map((task) => (
            <label key={task} className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={tasks.includes(task)}
                onChange={() => toggleTask(task)}
                className="w-4 h-4"
              />
              <span>{task}</span>
            </label>
          ))}
        </div>
      </div>

      {!articleUuid && (
        <div className="space-y-2">
          <Label>Time Period</Label>
          <Select value={timeRange} onValueChange={setTimeRange}>
            <SelectTrigger>
              <SelectValue placeholder="Select time period" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Time</SelectItem>
              <SelectItem value="24h">Last 24 Hours</SelectItem>
              <SelectItem value="7d">Last 7 Days</SelectItem>
              <SelectItem value="1m">Last Month</SelectItem>
              <SelectItem value="1y">Last Year</SelectItem>
            </SelectContent>
          </Select>
        </div>
      )}

      <div className="flex items-center gap-2">
        <input
          id="missing-only"
          type="checkbox"
          checked={missingOnly}
          onChange={(e) => setMissingOnly(e.target.checked)}
          className="w-4 h-4"
        />
        <Label htmlFor="missing-only" className="cursor-pointer">Only if missing</Label>
      </div>

      <Button
        type="submit"
        disabled={rescheduleMutation.isPending || tasks.length === 0}
        className="w-full"
      >
        {rescheduleMutation.isPending ? "Scheduling..." : articleUuid ? "Run Selected Tasks" : "Run Tasks for All Articles"}
      </Button>
    </form>
  );
}
