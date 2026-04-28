import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { $api } from '../lib/api'
import { Button } from '../components/ui/button'
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '../components/ui/card'
import { Label } from '../components/ui/label'
import { Input } from '../components/ui/input'
import { Badge } from '../components/ui/badge'
import { useQueryClient } from '@tanstack/react-query'

export const Route = createFileRoute('/_app/profile')({
  component: ProfilePage,
})

function ProfilePage() {
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const { data: user, isLoading } = $api.useQuery('get', '/auth/me', undefined, {
    retry: false,
  })

  const { mutate: logout } = $api.useMutation('post', '/auth/logout', {
    onSuccess: () => {
      queryClient.clear()
      navigate({ to: '/' })
    }
  })

  const { mutate: deleteAccount, isPending: isDeleting } = $api.useMutation('delete', '/auth/me', {
    onSuccess: () => {
      queryClient.clear()
      navigate({ to: '/' })
    },
    onError: (error: any) => {
      alert('Failed to delete account: ' + (error?.message ?? "unknown error"))
    }
  })

  if (isLoading) {
    return (
      <div className="flex justify-center py-12">
        <div className="animate-pulse font-medium">Loading profile...</div>
      </div>
    )
  }

  if (!user) {
    return (
      <div className="flex flex-col items-center gap-4 py-12 text-center">
        <h1 className="text-2xl font-bold">Not logged in</h1>
        <p className="text-muted-foreground">Please log in to view your profile.</p>
        <Button onClick={() => {
          const backendApi = import.meta.env.VITE_BACKEND_API ?? "localhost:8080";
          const baseUrl = backendApi.startsWith('http') ? backendApi : `http://${backendApi}`;
          window.location.href = `${baseUrl}/auth/login?redirect_to_frontend=true`;
        }}>
          Login
        </Button>
      </div>
    )
  }

  const handleLogout = () => {
    logout({})
  }

  const handleDeleteAccount = () => {
    if (window.confirm('Are you sure you want to delete your account? This action cannot be undone.')) {
      deleteAccount({
        params: {
          query: undefined,
          path: undefined,
          header: undefined,
          cookie: undefined
        }
      })
    }
  }

  return (
    <div className="container pb-10 space-y-8">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">User Profile</h1>
          <p className="text-muted-foreground text-sm mt-1">Manage your account settings and view your information.</p>
        </div>
        {user.isAdmin && <Badge variant="secondary" className="px-3 py-1">Administrator</Badge>}
      </div>
      
      <Card>
        <CardHeader>
          <CardTitle>Account Information</CardTitle>
          <CardDescription>
            Your personal details from your identity provider.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="space-y-2">
            <Label htmlFor="username">Username</Label>
            <Input id="username" value={user.username} readOnly className="bg-muted/50 cursor-default" />
          </div>
          <div className="space-y-2">
            <Label htmlFor="email">Email Address</Label>
            <Input id="email" value={user.email} readOnly className="bg-muted/50 cursor-default" />
          </div>
          <div className="space-y-2">
            <Label htmlFor="id">User ID</Label>
            <Input id="id" value={user.id} readOnly className="bg-muted/50 font-mono text-xs cursor-default" />
          </div>
        </CardContent>
        <CardFooter className="flex justify-between items-center">
          <Button variant="outline" onClick={handleLogout}>
            Log out
          </Button>
        </CardFooter>
      </Card>

      <div className="pt-4">
        <h3 className="text-lg font-semibold text-destructive mb-2">Danger Zone</h3>
        <Card className="border-destructive/20 bg-destructive/5">
          <CardContent className="flex-row flex justify-between gap-4">
            <div className="text-center sm:text-left">
              <p className="font-medium text-sm">Delete Account</p>
              <p className="text-xs text-muted-foreground mt-1">
                This will permanently delete your account and all associated data.
              </p>
            </div>
            <Button 
              variant="destructive" 
              onClick={handleDeleteAccount}
              disabled={isDeleting}
              className="shrink-0"
            >
              {isDeleting ? 'Deleting...' : 'Delete My Account'}
            </Button>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
