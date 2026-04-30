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
import { Plus, Trash2, Edit2, X, Check } from 'lucide-react'
import { type components } from '#/lib/api/v1'

export const Route = createFileRoute('/admin/categories')({
  component: AdminCategories,
})

function AdminCategories() {
  const { data: categories, refetch } = $api.useQuery('get', '/category')
  const { mutate: deleteCategory } = $api.useMutation('delete', '/category/{id}')
  const { mutate: createCategory } = $api.useMutation('post', '/category')
  const { mutate: updateCategory } = $api.useMutation('put', '/category/{id}')

  const [editingId, setEditingId] = useState<string | null>(null)
  const [editForm, setEditForm] = useState<components["schemas"]["CategoryDto"]>({
    humanName: '',
    humanDescription: '',
    modelDescription: '',
  })
  const [isAdding, setIsAdding] = useState(false)

  const handleEdit = (category: components["schemas"]["CategoryDto"]) => {
    setEditingId(category.id!)
    setEditForm({ ...category })
  }

  const handleSave = (id: string) => {
    updateCategory({
      params: { path: { id } },
      body: editForm,
    }, {
      onSuccess: () => {
        setEditingId(null)
        refetch()
      }
    })
  }

  const handleCreate = () => {
    createCategory({
      body: editForm,
    }, {
      onSuccess: () => {
        setIsAdding(false)
        setEditForm({ humanName: '', humanDescription: '', modelDescription: '' })
        refetch()
      }
    })
  }

  const handleDelete = (id: string) => {
    if (confirm('Are you sure you want to delete this category?')) {
      deleteCategory({ params: { path: { id } } }, {
        onSuccess: () => refetch()
      })
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold tracking-tight">Category Management</h1>
        <Button onClick={() => {
            setIsAdding(true)
            setEditForm({ humanName: '', humanDescription: '', modelDescription: '' })
        }} disabled={isAdding}>
          <Plus className="size-4 mr-2" />
          Add Category
        </Button>
      </div>

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Name</TableHead>
              <TableHead>Human Description</TableHead>
              <TableHead>Model Description</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isAdding && (
              <TableRow className="bg-muted/50">
                <TableCell>
                  <Input
                    placeholder="Category Name"
                    value={editForm.humanName}
                    onChange={(e) => setEditForm({ ...editForm, humanName: e.target.value })}
                  />
                </TableCell>
                <TableCell>
                  <Input
                    placeholder="Human Description"
                    value={editForm.humanDescription}
                    onChange={(e) => setEditForm({ ...editForm, humanDescription: e.target.value })}
                  />
                </TableCell>
                <TableCell>
                  <Input
                    placeholder="Model Description"
                    value={editForm.modelDescription}
                    onChange={(e) => setEditForm({ ...editForm, modelDescription: e.target.value })}
                  />
                </TableCell>
                <TableCell className="text-right">
                  <div className="flex justify-end gap-2">
                    <Button size="sm" onClick={handleCreate}>
                      <Check className="size-4 mr-1" /> Create
                    </Button>
                    <Button size="sm" variant="outline" onClick={() => setIsAdding(false)}>
                      <X className="size-4" />
                    </Button>
                  </div>
                </TableCell>
              </TableRow>
            )}
            {categories?.map((category) => (
              <TableRow key={category.id}>
                <TableCell>
                  {editingId === category.id ? (
                    <Input
                      value={editForm.humanName}
                      onChange={(e) => setEditForm({ ...editForm, humanName: e.target.value })}
                    />
                  ) : (
                    category.humanName
                  )}
                </TableCell>
                <TableCell>
                  {editingId === category.id ? (
                    <Input
                      value={editForm.humanDescription}
                      onChange={(e) => setEditForm({ ...editForm, humanDescription: e.target.value })}
                    />
                  ) : (
                    <div className="max-w-[200px] truncate" title={category.humanDescription}>
                      {category.humanDescription}
                    </div>
                  )}
                </TableCell>
                <TableCell>
                  {editingId === category.id ? (
                    <Input
                      value={editForm.modelDescription}
                      onChange={(e) => setEditForm({ ...editForm, modelDescription: e.target.value })}
                    />
                  ) : (
                    <div className="max-w-[200px] truncate" title={category.modelDescription}>
                      {category.modelDescription}
                    </div>
                  )}
                </TableCell>
                <TableCell className="text-right">
                  <div className="flex justify-end gap-2">
                    {editingId === category.id ? (
                      <>
                        <Button size="sm" onClick={() => handleSave(category.id!)}>
                          <Check className="size-4" />
                        </Button>
                        <Button size="sm" variant="outline" onClick={() => setEditingId(null)}>
                          <X className="size-4" />
                        </Button>
                      </>
                    ) : (
                      <>
                        <Button size="sm" variant="outline" onClick={() => handleEdit(category)}>
                          <Edit2 className="size-4" />
                        </Button>
                        <Button size="sm" variant="destructive" onClick={() => handleDelete(category.id!)}>
                          <Trash2 className="size-4" />
                        </Button>
                      </>
                    )}
                  </div>
                </TableCell>
              </TableRow>
            ))}
            {!isAdding && categories?.length === 0 && (
              <TableRow>
                <TableCell colSpan={4} className="text-center py-10 text-muted-foreground">
                  No categories found.
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}
