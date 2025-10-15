import { component$ } from '@builder.io/qwik';
import { LuExternalLink, LuSettings, LuRefreshCw, LuTrash2, LuGlobe, LuPencil } from '@qwikest/icons/lucide';
import type { ConnectedWebsite } from '~/types/website-builders';

interface ConnectedWebsitesProps {
  websites: ConnectedWebsite[];
}

export const ConnectedWebsites = component$<ConnectedWebsitesProps>(({ websites }) => {
  const getStatusColor = (status: ConnectedWebsite['status']) => {
    switch (status) {
      case 'active': return 'text-green-600 bg-green-50';
      case 'inactive': return 'text-gray-600 bg-gray-50';
      case 'syncing': return 'text-blue-600 bg-blue-50';
      case 'error': return 'text-red-600 bg-red-50';
      default: return 'text-gray-600 bg-gray-50';
    }
  };

  const getStatusIcon = (status: ConnectedWebsite['status']) => {
    switch (status) {
      case 'active': return <div class="w-2 h-2 bg-green-500 rounded-full"></div>;
      case 'inactive': return <div class="w-2 h-2 bg-gray-400 rounded-full"></div>;
      case 'syncing': return <LuRefreshCw class="w-3 h-3 text-blue-500 animate-spin" />;
      case 'error': return <div class="w-2 h-2 bg-red-500 rounded-full"></div>;
      default: return <div class="w-2 h-2 bg-gray-400 rounded-full"></div>;
    }
  };

  if (websites.length === 0) {
    return (
      <div class="text-center py-12 bg-gray-50 rounded-xl border-2 border-dashed border-gray-300">
        <LuGlobe class="w-12 h-12 text-gray-400 mx-auto mb-4" />
        <h3 class="text-lg font-medium text-gray-900 mb-2">No websites connected yet</h3>
        <p class="text-gray-600 mb-6">
          Select a website builder above to connect your first website
        </p>
      </div>
    );
  }

  return (
    <div class="space-y-4">
      {websites.map((website) => (
        <div
          key={website.id}
          class="bg-white border border-gray-200 rounded-xl p-6 hover:shadow-md transition-shadow"
        >
          <div class="flex items-start justify-between">
            <div class="flex-1">
              <div class="flex items-center gap-3 mb-2">
                <h3 class="text-lg font-semibold text-gray-900">{website.name}</h3>
                <span class={`inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium ${getStatusColor(website.status)}`}>
                  {getStatusIcon(website.status)}
                  {website.status.charAt(0).toUpperCase() + website.status.slice(1)}
                </span>
              </div>
              
              <div class="flex items-center gap-4 text-sm text-gray-600 mb-3">
                <div class="flex items-center gap-1">
                  <span class="font-medium">Builder:</span>
                  <span>{website.builderName}</span>
                </div>
                {website.domain && (
                  <div class="flex items-center gap-1">
                    <span class="font-medium">Domain:</span>
                    <span>{website.domain}</span>
                  </div>
                )}
              </div>

              {website.url && (
                <a
                  href={website.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  class="inline-flex items-center gap-1 text-blue-600 hover:text-blue-800 text-sm font-medium"
                >
                  Visit Website
                  <LuExternalLink class="w-3 h-3" />
                </a>
              )}

              {website.lastSync && (
                <div class="text-xs text-gray-500 mt-2">
                  Last synced: {new Date(website.lastSync).toLocaleDateString()}
                </div>
              )}
            </div>

            <div class="flex items-center gap-2 ml-4">
              <button
                class="p-2 text-gray-400 hover:text-blue-600 hover:bg-blue-50 rounded-lg transition-colors"
                title="Edit website"
              >
                <LuPencil class="w-4 h-4" />
              </button>
              
              <button
                class="p-2 text-gray-400 hover:text-blue-600 hover:bg-blue-50 rounded-lg transition-colors"
                title="Sync website"
              >
                <LuRefreshCw class="w-4 h-4" />
              </button>
              
              <button
                class="p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-50 rounded-lg transition-colors"
                title="Settings"
              >
                <LuSettings class="w-4 h-4" />
              </button>
              
              <button
                class="p-2 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                title="Disconnect website"
              >
                <LuTrash2 class="w-4 h-4" />
              </button>
            </div>
          </div>
        </div>
      ))}
    </div>
  );
});
