import { component$, useSignal } from '@builder.io/qwik';
import { LuFileText, LuPlus, LuPenTool, LuTrash2, LuEye, LuCalendar, LuUser } from '@qwikest/icons/lucide';

interface ContentItem {
  id: string;
  title: string;
  slug: string;
  status: 'draft' | 'published' | 'archived';
  author: string;
  created_at: string;
  updated_at: string;
  published_at?: string;
}

interface ContentManagementProps {
  content: ContentItem[];
  onCreateContent$: () => void;
  onEditContent$: (id: string) => void;
  onDeleteContent$: (id: string) => void;
  onPublishContent$: (id: string) => void;
}

export default component$<ContentManagementProps>(({ 
  content, 
  onCreateContent$, 
  onEditContent$, 
  onDeleteContent$, 
  onPublishContent$ 
}) => {
  const selectedStatus = useSignal<string>('all');

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
          onClick$={onCreateContent$}
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
                      {item.author}
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
                      onClick$={() => onPublishContent$(item.id)}
                      class="bg-green-600 hover:bg-green-700 text-white px-3 py-1 rounded text-sm transition-colors"
                    >
                      Publish
                    </button>
                  )}
                  <button
                    onClick$={() => onEditContent$(item.id)}
                    class="bg-blue-600 hover:bg-blue-700 text-white p-2 rounded transition-colors"
                  >
                    <LuPenTool class="w-4 h-4" />
                  </button>
                  <button
                    onClick$={() => onDeleteContent$(item.id)}
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
              onClick$={onCreateContent$}
              class="mt-4 bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg font-semibold transition-colors"
            >
              Create your first content
            </button>
          </div>
        )}
      </div>
    </div>
  );
});
