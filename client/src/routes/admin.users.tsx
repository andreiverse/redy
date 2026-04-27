import { createFileRoute, Link } from '@tanstack/react-router'
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
import { Badge } from '../components/ui/badge'

export const Route = createFileRoute('/admin/users')({
  component: AdminUsers,
})

function AdminUsers() {
  const { data: users, refetch } = $api.useQuery('get', '/user')
  const { mutate: deleteUser } = $api.useMutation('delete', '/user/{user_id}')
  const { mutate: patchUser } = $api.useMutation('patch', '/user/{user_id}')

  const handleToggleAdmin = (user: any) => {
    patchUser({
      params: { path: { user_id: user.id } },
      body: { isAdmin: !user.isAdmin }
    }, {
      onSuccess: () => refetch()
    })
  }

  const handleToggleCanCreateFeeds = (user: any) => {
    patchUser({
      params: { path: { user_id: user.id } },
      body: { canCreateFeeds: !user.canCreateFeeds }
    }, {
      onSuccess: () => refetch()
    })
  }

  const handleDelete = (id: string) => {
    if (confirm('Are you sure you want to delete this user?')) {
      deleteUser({ params: { path: { user_id: id } } }, {
        onSuccess: () => refetch()
      })
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold tracking-tight">User Management</h1>
      </div>

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Username</TableHead>
              <TableHead>Email</TableHead>
              <TableHead>Status</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {users?.map((user) => (
              <TableRow key={user.id}>
                <TableCell className="font-medium">{user.username}</TableCell>
                <TableCell>{user.email}</TableCell>
                <TableCell>
                  <div className="flex gap-2">
                    {user.isAdmin && <Badge variant="default">Admin</Badge>}
                    {user.canCreateFeeds && <Badge variant="secondary">Can Create Feeds</Badge>}
                  </div>
                </TableCell>
                <TableCell className="text-right">
                  <div className="flex justify-end gap-2">
                    <Button 
                      size="sm" 
                      variant="outline" 
                      asChild
                    >
                      <Link to="/admin/feeds" search={{ userId: user.id }}>
                        View Feeds
                      </Link>
                    </Button>
                    <Button 
                      size="sm" 
                      variant="outline" 
                      onClick={() => handleToggleAdmin(user)}
                    >
                      {user.isAdmin ? 'Revoke Admin' : 'Make Admin'}
                    </Button>
                    <Button 
                      size="sm" 
                      variant="outline" 
                      onClick={() => handleToggleCanCreateFeeds(user)}
                    >
                      {user.canCreateFeeds ? 'Disable Feeds' : 'Enable Feeds'}
                    </Button>
                    <Button 
                      size="sm" 
                      variant="destructive" 
                      onClick={() => handleDelete(user.id)}
                    >
                      Delete
                    </Button>
                  </div>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}
