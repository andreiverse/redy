import { useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "../ui/dialog";
import { Button } from "../ui/button";
import { RescheduleTaskForm } from "./RescheduleTaskForm";

interface RescheduleArticleDialogProps {
  articleUuid: string;
}

export function RescheduleArticleDialog({ articleUuid }: RescheduleArticleDialogProps) {
  const [open, setOpen] = useState(false);

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button variant="outline" size="sm">Reschedule Tasks</Button>
      </DialogTrigger>

      <DialogContent>
        <DialogHeader>
          <DialogTitle>Reschedule Tasks</DialogTitle>
        </DialogHeader>

        <RescheduleTaskForm 
          articleUuid={articleUuid} 
          onSuccess={() => setOpen(false)} 
        />
      </DialogContent>
    </Dialog>
  );
}
