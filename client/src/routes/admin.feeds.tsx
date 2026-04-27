import { createFileRoute } from '@tanstack/react-router'
import { $api } from '../lib/api'
import { Button } from '../components/ui/button'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '../components/ui/table'
import { Input } from '../components/ui/input'
import { useState } from 'react'

type FeedSearchParams = {
  userId?: string
}

export const Route = createFileRoute('/admin/feeds')({
  validateSearch: (search: Record<string, unknown>): FeedSearchParams => {
    return {
      userId: (search.userId as string) || undefined,
    }
  },
  component: AdminFeeds,
})

function AdminFeeds() {
  const { userId } = Route.useSearch()
  const { data: feeds, refetch } = $api.useQuery('get', '/feed', {
    params: {
      query: {
        user_id: userId
      }
    }
  })
  const { mutate: deleteFeed } = $api.useMutation('delete', '/feed/{feed_uuid}')
  const { mutate: patchFeed } = $api.useMutation('patch', '/feed/{feed_uuid}')

  const [editingId, setEditingId] = useState<string | null>(null)
  const [editName, setEditName] = useState('')
  const [editUrl, setEditUrl] = useState('')
  const [editOwner, setEditOwner] = useState('')

  const handleEdit = (feed: any) => {
    setEditingId(feed.id)
    setEditName(feed.name)
    setEditUrl(feed.url)
    setEditOwner(feed.ownerUuid || '')
  }

  const handleSave = (id: string) => {
    patchFeed({
      params: { path: { feed_uuid: id } },
      body: {
        name: editName,
        url: editUrl,
        ownerUuid: editOwner === '' ? null : editOwner,
      },
    }, {
      onSuccess: () => {
        setEditingId(null)
        refetch()
      }
    })
  }

  const handleDelete = (id: string) => {
    if (confirm('Are you sure you want to delete this feed?')) {
      deleteFeed({ params: { path: { feed_uuid: id } } }, {
        onSuccess: () => refetch()
      })
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold tracking-tight">
          Feed Management {userId && <span className="text-muted-foreground text-lg font-normal">- User: {userId}</span>}
        </h1>
        {userId && (
          <Button variant="outline" size="sm" onClick={() => window.history.back()}>
            Back to Users
          </Button>
        )}
      </div>

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Name</TableHead>
              <TableHead>URL</TableHead>
              <TableHead>Owner UUID</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {feeds?.map((feed) => (
              <TableRow key={feed.id}>
                <TableCell>
                  {editingId === feed.id ? (
                    <Input
                      value={editName}
                      onChange={(e) => setEditName(e.target.value)}
                    />
                  ) : (
                    feed.name
                  )}
                </TableCell>
                <TableCell>
                  {editingId === feed.id ? (
                    <Input
                      value={editUrl}
                      onChange={(e) => setEditUrl(e.target.value)}
                    />
                  ) : (
                    <div className="max-w-50 truncate" title={feed.url}>
                      {feed.url}
                    </div>
                  )}
                </TableCell>
                <TableCell>
                  {editingId === feed.id ? (
                    <Input
                      value={editOwner}
                      onChange={(e) => setEditOwner(e.target.value)}
                      placeholder="User UUID or empty"
                    />
                  ) : (
                    <span className="text-xs font-mono">{feed.ownerUuid || 'System'}</span>
                  )}
                </TableCell>
                <TableCell className="text-right">
                  <div className="flex justify-end gap-2">
                    {editingId === feed.id ? (
                      <>
                        <Button size="sm" onClick={() => handleSave(feed.id)}>Save</Button>
                        <Button size="sm" variant="outline" onClick={() => setEditingId(null)}>Cancel</Button>
                      </>
                    ) : (
                      <>
                        <Button size="sm" variant="outline" onClick={() => handleEdit(feed)}>Edit</Button>
                        <Button size="sm" variant="destructive" onClick={() => handleDelete(feed.id)}>Delete</Button>
                      </>
                    )}
                  </div>
                </TableCell>
              </TableRow>
            ))}
            {feeds?.length === 0 && (
              <TableRow>
                <TableCell colSpan={4} className="text-center py-10 text-muted-foreground">
                  No feeds found.
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}
