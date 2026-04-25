import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { $api } from '../lib/api'
import { Button } from '../components/ui/button'
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '../components/ui/card'
import { useQueryClient } from '@tanstack/react-query'
import { object } from 'zod'

export const Route = createFileRoute('/profile')({
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
    onError: (error) => {
      let any_error = error as any;
      // TODO: move this to a function
      if (!(any_error instanceof object)) alert("Unknown error");
      alert('Failed to delete account: ' + (any_error.message ?? "unknown message"))
    }
  })

  if (isLoading) {
    return (
      <div className="flex justify-center py-12">
        <div className="text-[var(--sea-ink)] animate-pulse font-medium">Loading profile...</div>
      </div>
    )
  }

  if (!user) {
    return (
      <div className="flex flex-col items-center gap-4 py-12">
        <h1 className="text-2xl font-bold text-[var(--sea-ink)]">Not logged in</h1>
        <p className="text-[var(--sea-ink-muted)]">Please log in to view your profile.</p>
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
    <div className="container max-w-2xl px-4 py-8">
      <h1 className="mb-8 text-3xl font-bold tracking-tight text-[var(--sea-ink)]">User Profile</h1>
      
      <Card className="border-[var(--line)] bg-[var(--card-bg)] shadow-sm">
        <CardHeader>
          <CardTitle className="text-xl text-[var(--sea-ink)]">Account Information</CardTitle>
          <CardDescription className="text-[var(--sea-ink-muted)]">
            Your personal details from your OIDC provider.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-1 gap-1 sm:grid-cols-3 sm:gap-4">
            <span className="text-sm font-semibold text-[var(--sea-ink-muted)]">Username</span>
            <span className="text-sm text-[var(--sea-ink)] sm:col-span-2">{user.username}</span>
          </div>
          <div className="grid grid-cols-1 gap-1 sm:grid-cols-3 sm:gap-4 border-t border-[var(--line)] pt-4">
            <span className="text-sm font-semibold text-[var(--sea-ink-muted)]">Email</span>
            <span className="text-sm text-[var(--sea-ink)] sm:col-span-2">{user.email}</span>
          </div>
          <div className="grid grid-cols-1 gap-1 sm:grid-cols-3 sm:gap-4 border-t border-[var(--line)] pt-4">
            <span className="text-sm font-semibold text-[var(--sea-ink-muted)]">User ID</span>
            <span className="text-xs font-mono text-[var(--sea-ink-muted)] sm:col-span-2">{user.id}</span>
          </div>
        </CardContent>
        <CardFooter className="flex flex-col items-start gap-8 border-t border-[var(--line)] pt-6">
          <div className="w-full">
            <h3 className="text-sm font-bold text-[var(--sea-ink)] mb-2 uppercase tracking-wider">Account Actions</h3>
            <Button 
              variant="outline" 
              onClick={handleLogout}
              className="w-full sm:w-auto border-[var(--chip-line)] text-[var(--sea-ink)] hover:bg-[var(--chip-bg)]"
            >
              Log out
            </Button>
          </div>

          <div className="w-full">
            <h3 className="text-sm font-bold text-red-500 mb-2 uppercase tracking-wider">Danger Zone</h3>
            <p className="text-xs text-[var(--sea-ink-muted)] mb-4">
              Once you delete your account, there is no going back. Please be certain.
            </p>
            <Button 
              variant="destructive" 
              onClick={handleDeleteAccount}
              disabled={isDeleting}
              className="w-full sm:w-auto"
            >
              {isDeleting ? 'Deleting...' : 'Delete Account'}
            </Button>
          </div>
        </CardFooter>
      </Card>
    </div>
  )
}
