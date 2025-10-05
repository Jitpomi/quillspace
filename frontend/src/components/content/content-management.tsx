import { component$, useSignal, $ } from '@builder.io/qwik';
import { LuFileText, LuPlus, LuPenTool, LuTrash2, LuEye, LuCalendar, LuUser } from '@qwikest/icons/lucide';
import { api, type Content } from '../../services/api';
import { 
  useActions, 
  triggerRefreshContent, 
  triggerCreateContent, 
  triggerEditContent, 
  triggerDeleteContent, 
  triggerPublishContent 
} from '../../contexts/actions';

interface ContentManagementProps {
  content: Content[];
}

export default component$<ContentManagementProps>(({ 
  content
}) => {
  const selectedStatus = useSignal<string>('all');
  const isCreating = useSignal(false);
  const showCreateModal = useSignal(false);
  const newContentTitle = useSignal('');
  const newContentSlug = useSignal('');
  const newContentContent = useSignal('');
  const error = useSignal<string | null>(null);
  
  // Get actions context
  const actions = useActions();

  // Handle creating new content
  const handleCreateContent = $(async () => {
    if (!newContentTitle.value || !newContentSlug.value) {
      error.value = 'Title and slug are required';
      return;
    }

    try {
      isCreating.value = true;
      error.value = null;

      await api.createContent({
        title: newContentTitle.value,
        slug: newContentSlug.value,
        content: newContentContent.value || '',
        status: 'draft',
      });

      // Reset form
      newContentTitle.value = '';
      newContentSlug.value = '';
      newContentContent.value = '';
      showCreateModal.value = false;

      // Trigger callbacks via actions context
      triggerRefreshContent(actions);
      triggerCreateContent(actions);
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to create content';
    } finally {
      isCreating.value = false;
    }
  });

  // Handle publishing content
  const handlePublishContent = $(async (id: string) => {
    try {
      await api.publishContent(id);
      triggerRefreshContent(actions);
      triggerPublishContent(actions, id);
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to publish content';
    }
  });

  // Handle deleting content
  const handleDeleteContent = $(async (id: string) => {
    if (!confirm('Are you sure you want to delete this content?')) {
      return;
    }

    try {
      await api.deleteContent(id);
      triggerRefreshContent(actions);
      triggerDeleteContent(actions, id);
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to delete content';
    }
  });

  // Generate slug from title
  const generateSlug = $(() => {
    const slug = newContentTitle.value
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/(^-|-$)/g, '');
    newContentSlug.value = slug;
  });

  const filteredContent = content.filter(item => 
    selectedStatus.value === 'all' || item.status === selectedStatus.value
  );

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'published': return 'bg-green-100 text-green-800';
      case 'draft': return 'bg-yellow-100 text-yellow-800';
      case 'archived': return 'bg-gray-100 text-gray-800';
      default: return 'bg-gray-100 text-gray-800';
    }
  };

  return (
    <div class="space-y-6">
      {/* Header */}
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          <LuFileText class="w-8 h-8 text-blue-600" />
          <h2 class="text-3xl font-bold text-gray-900">Content Management</h2>
        </div>
        <button
          onClick$={() => showCreateModal.value = true}
          class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg font-semibold transition-colors flex items-center gap-2"
        >
          <LuPlus class="w-4 h-4" />
          New Content
        </button>
      </div>

      {/* Filters */}
      <div class="bg-white rounded-xl shadow-lg p-6">
        <div class="flex gap-2 mb-4">
          {['all', 'published', 'draft', 'archived'].map((status) => (
            <button
              key={status}
              onClick$={() => selectedStatus.value = status}
              class={`px-4 py-2 rounded-lg font-medium transition-colors ${
                selectedStatus.value === status
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
              }`}
            >
              {status.charAt(0).toUpperCase() + status.slice(1)}
            </button>
          ))}
        </div>

        {/* Content List */}
        <div class="space-y-4">
          {filteredContent.map((item) => (
            <div key={item.id} class="border border-gray-200 rounded-lg p-4 hover:shadow-md transition-shadow">
              <div class="flex items-start justify-between">
                <div class="flex-1">
                  <div class="flex items-center gap-3 mb-2">
                    <h3 class="text-lg font-semibold text-gray-900">{item.title}</h3>
                    <span class={`px-2 py-1 rounded-full text-xs font-medium ${getStatusColor(item.status)}`}>
                      {item.status}
                    </span>
                  </div>
                  <p class="text-sm text-gray-600 mb-3">/{item.slug}</p>
                  <div class="flex items-center gap-4 text-xs text-gray-500">
                    <div class="flex items-center gap-1">
                      <LuUser class="w-3 h-3" />
                      {item.author_id}
                    </div>
                    <div class="flex items-center gap-1">
                      <LuCalendar class="w-3 h-3" />
                      Created {new Date(item.created_at).toLocaleDateString()}
                    </div>
                    {item.published_at && (
                      <div class="flex items-center gap-1">
                        <LuEye class="w-3 h-3" />
                        Published {new Date(item.published_at).toLocaleDateString()}
                      </div>
                    )}
                  </div>
                </div>
                <div class="flex items-center gap-2">
                  {item.status === 'draft' && (
                    <button
                      onClick$={() => handlePublishContent(item.id)}
                      class="bg-green-600 hover:bg-green-700 text-white px-3 py-1 rounded text-sm transition-colors"
                    >
                      Publish
                    </button>
                  )}
                  <button
                    onClick$={() => triggerEditContent(actions, item.id)}
                    class="bg-blue-600 hover:bg-blue-700 text-white p-2 rounded transition-colors"
                  >
                    <LuPenTool class="w-4 h-4" />
                  </button>
                  <button
                    onClick$={() => handleDeleteContent(item.id)}
                    class="bg-red-600 hover:bg-red-700 text-white p-2 rounded transition-colors"
                  >
                    <LuTrash2 class="w-4 h-4" />
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>
        {filteredContent.length === 0 && (
          <div class="text-center py-12">
            <LuFileText class="w-12 h-12 text-gray-400 mx-auto mb-4" />
            <p class="text-gray-600">No content found</p>
            <button
              onClick$={() => triggerCreateContent(actions)}
              class="mt-4 bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg font-semibold transition-colors"
            >
              Create your first content
            </button>
          </div>
        )}
      </div>

      {/* Create Content Modal */}
      {showCreateModal.value && (
        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div class="bg-white rounded-xl shadow-xl p-6 w-full max-w-md mx-4">
            <h3 class="text-xl font-bold text-gray-900 mb-4">Create New Content</h3>
            
            {error.value && (
              <div class="bg-red-50 border border-red-200 rounded-lg p-3 mb-4">
                <p class="text-red-700 text-sm">{error.value}</p>
              </div>
            )}

            <div class="space-y-4">
              <div>
                <label class="block text-sm font-medium text-gray-700 mb-1">Title</label>
                <input
                  type="text"
                  value={newContentTitle.value}
                  onInput$={(e) => {
                    newContentTitle.value = (e.target as HTMLInputElement).value;
                    generateSlug();
                  }}
                  class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  placeholder="Enter content title"
                />
              </div>

              <div>
                <label class="block text-sm font-medium text-gray-700 mb-1">Slug</label>
                <input
                  type="text"
                  value={newContentSlug.value}
                  onInput$={(e) => newContentSlug.value = (e.target as HTMLInputElement).value}
                  class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  placeholder="content-slug"
                />
              </div>

              <div>
                <label class="block text-sm font-medium text-gray-700 mb-1">Content (Optional)</label>
                <textarea
                  value={newContentContent.value}
                  onInput$={(e) => newContentContent.value = (e.target as HTMLTextAreaElement).value}
                  rows={4}
                  class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  placeholder="Enter content body..."
                />
              </div>
            </div>

            <div class="flex gap-3 mt-6">
              <button
                onClick$={() => {
                  showCreateModal.value = false;
                  error.value = null;
                  newContentTitle.value = '';
                  newContentSlug.value = '';
                  newContentContent.value = '';
                }}
                class="flex-1 px-4 py-2 border border-gray-300 rounded-lg text-gray-700 hover:bg-gray-50 transition-colors"
              >
                Cancel
              </button>
              <button
                onClick$={handleCreateContent}
                disabled={isCreating.value}
                class="flex-1 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50"
              >
                {isCreating.value ? 'Creating...' : 'Create'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
});
