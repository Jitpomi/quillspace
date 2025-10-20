import { component$, useSignal} from '@builder.io/qwik';
import { LuSettings, LuRefreshCw, LuTrash2, LuGlobe, LuPencil, LuEye} from '@qwikest/icons/lucide';
import type { ConnectedWebsite } from '~/types/website-builders';
import {useAuth} from "~/hooks/useAuth";

interface ConnectedWebsitesProps {
  // Make websites optional since we'll fetch them via action
  websites?: ConnectedWebsite[];
}


export const ConnectedWebsites = component$<ConnectedWebsitesProps>(({ websites = [] }) => {
  const websiteList = useSignal(websites);
  const { userPath } = useAuth();
  const getStatusColor = (status: ConnectedWebsite['status']) => {
    switch (status) {
      case 'active': return 'text-green-600 bg-green-50';
      case 'inactive': return 'text-gray-600 bg-gray-50';
      case 'syncing': return 'text-blue-600 bg-blue-50';
      case 'error': return 'text-red-600 bg-red-50';
      case 'pending': return 'text-yellow-600 bg-yellow-50';
      default: return 'text-gray-600 bg-gray-50';
    }
  };

  const getStatusIcon = (status: ConnectedWebsite['status']) => {
    switch (status) {
      case 'active': return <div class="w-2 h-2 bg-green-500 rounded-full"></div>;
      case 'inactive': return <div class="w-2 h-2 bg-gray-400 rounded-full"></div>;
      case 'syncing': return <LuRefreshCw class="w-3 h-3 text-blue-500 animate-spin" />;
      case 'error': return <div class="w-2 h-2 bg-red-500 rounded-full"></div>;
      case 'pending': return <div class="w-2 h-2 bg-yellow-500 rounded-full"></div>;
      default: return <div class="w-2 h-2 bg-gray-400 rounded-full"></div>;
    }
  };

  if (websiteList.value.length === 0) {
    return (
      <>
        <div class="text-center py-16 bg-gradient-to-br from-gray-50 to-gray-100/50 rounded-2xl border border-gray-200/60">
          <div class="w-16 h-16 bg-gradient-to-br from-[#9CAF88] to-[#8BA079] rounded-2xl flex items-center justify-center mx-auto mb-6 shadow-lg">
            <LuGlobe class="w-8 h-8 text-white" />
          </div>
          <h3 class="text-xl font-semibold text-gray-900 mb-3">No websites connected yet</h3>
          <p class="text-gray-600 max-w-md mx-auto leading-relaxed">
            Select a website builder above to connect your first website and start managing your content
          </p>
        </div>
      </>
    );
  }

  return (
    <>
      <div class="mb-6">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-gray-600 text-sm">
              {websiteList.value.length} website{websiteList.value.length !== 1 ? 's' : ''} connected
            </p>
          </div>
          <div class="flex items-center gap-2">
            <button class="p-2 text-gray-400 hover:text-gray-600 rounded-lg transition-colors" title="Refresh">
              <LuRefreshCw class="w-4 h-4" />
            </button>
            <button class="p-2 text-gray-400 hover:text-gray-600 rounded-lg transition-colors" title="Settings">
              <LuSettings class="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>
      
      <div class="space-y-4">
        {websiteList.value.map((website) => (
        <div
          key={website.id}
          class="bg-white rounded-2xl p-6 shadow-sm border border-gray-100 hover:shadow-md transition-all duration-200 group"
        >
          {/* Mobile Layout */}
          <div class="block md:hidden">
            <div class="text-center">
              <div class="w-full h-32 bg-gray-100 rounded-lg overflow-hidden mb-4">
                {website.metadata?.thumbnail ? (
                  <img 
                    src={`https://www.wix.com${website.metadata.thumbnail}`} 
                    alt={website.name}
                    class="w-full h-full object-cover"
                    onError$={(e) => {
                      (e.target as HTMLImageElement).style.display = 'none';
                    }}
                  />
                ) : (
                  <div class="flex items-center justify-center h-full">
                    <LuGlobe class="w-8 h-8 text-gray-400" />
                  </div>
                )}
              </div>
              
              <div class="flex items-center justify-center gap-2 mb-2">
                <h3 class="text-gray-500">
                  {website.name}
                </h3>
                <span class={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs ${getStatusColor(website.status)}`}>
                  {getStatusIcon(website.status)}
                  {website.status === 'active' ? 'Live' : website.status.charAt(0).toUpperCase() + website.status.slice(1)}
                </span>
              </div>
              
              <div class="text-sm text-gray-500 mb-2 break-words">
                {website.url ? website.url.replace('https://', '') : 'No domain'}
              </div>
              
              <div class="text-xs text-gray-400 mb-4">
                Last sync: {website.lastSync ? new Date(website.lastSync).toLocaleDateString() : 'Never'}
              </div>
              
              <div class="flex gap-2">
                <a
                  href={`${userPath}/websites/${website.metadata?.wix_site_id}`}
                  class="flex-1 inline-flex items-center justify-center gap-2 px-4 py-2.5 bg-[#9CAF88] text-white rounded-lg transition-colors hover:bg-[#8BA079] font-medium"
                >
                  <LuPencil class="w-4 h-4" />
                  Edit Site
                </a>
                {website.url && (
                  <a
                    href={website.url}
                    target="_blank"
                    rel="noopener noreferrer"
                    class="px-3 py-2.5 border border-gray-300 text-gray-700 rounded-lg transition-colors hover:bg-gray-50"
                    title="View Live Site"
                  >
                    <LuEye class="w-4 h-4" />
                  </a>
                )}
              </div>
            </div>
          </div>

          {/* Desktop Layout */}
          <div class="hidden md:flex items-center gap-6">
            {/* Website Thumbnail */}
            <div class="w-32 h-20 bg-gray-50 rounded-lg overflow-hidden flex-shrink-0 border border-gray-200">
              {website.metadata?.thumbnail ? (
                <img 
                  src={`https://www.wix.com${website.metadata.thumbnail}`} 
                  alt={website.name}
                  class="w-full h-full object-cover"
                  onError$={(e) => {
                    (e.target as HTMLImageElement).style.display = 'none';
                  }}
                />
              ) : (
                <div class="flex items-center justify-center h-full bg-gray-50">
                  <LuGlobe class="w-6 h-6 text-gray-400" />
                </div>
              )}
            </div>

            {/* Website Info */}
            <div class="flex-1 min-w-0">
              <div class="flex items-center justify-between">
                <div class="flex-1">
                  <div class="flex items-center justify-between mb-2">
                    <div class="flex items-center gap-3">
                      <h3 class="text-gray-500">
                        {website.name}
                      </h3>
                      <span class={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs ${getStatusColor(website.status)}`}>
                        {getStatusIcon(website.status)}
                        {website.status === 'active' ? 'Live' : website.status.charAt(0).toUpperCase() + website.status.slice(1)}
                      </span>
                    </div>
                    
                    {/* All Actions as Icons */}
                    <div class="flex items-center gap-1">
                      <a
                        href={`${userPath}/websites/${website.metadata?.wix_site_id}`}
                        class="p-2 text-[#9CAF88] hover:text-[#8BA079] hover:bg-green-50 rounded-lg transition-all duration-150"
                        title="Edit Site"
                      >
                        <LuPencil class="w-4 h-4" />
                      </a>
                      {website.url && (
                        <a
                          href={website.url}
                          target="_blank"
                          rel="noopener noreferrer"
                          class="p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-50 rounded-lg transition-all duration-150"
                          title="View Live Site"
                        >
                          <LuEye class="w-4 h-4" />
                        </a>
                      )}
                      <button class="p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-50 rounded-lg transition-all duration-150" title="Sync Website">
                        <LuRefreshCw class="w-4 h-4" />
                      </button>
                      <button class="p-2 text-gray-400 hover:text-red-500 hover:bg-red-50 rounded-lg transition-all duration-150" title="Disconnect Website">
                        <LuTrash2 class="w-4 h-4" />
                      </button>
                    </div>
                  </div>
                  
                  <div class="grid grid-cols-1 lg:grid-cols-4 gap-6 text-sm mt-3">
                    <div>
                      <div class="text-gray-400 text-xs mb-1">Builder</div>
                      <div class="text-gray-500 text-sm capitalize">
                        {website.builderName || 'Unknown'}
                      </div>
                    </div>
                    
                    <div class="lg:col-span-2">
                      <div class="text-gray-400 text-xs mb-1">Domain</div>
                      <div class="text-gray-500 text-sm break-all">
                        {website.url ? website.url.replace('https://', '') : 'No domain'}
                      </div>
                    </div>
                    
                    <div>
                      <div class="text-gray-400 text-xs mb-1">Last Sync</div>
                      <div class="text-gray-500 text-sm">
                        {website.lastSync ? new Date(website.lastSync).toLocaleDateString() : 'Never'}
                      </div>
                    </div>
                    
                  </div>
                  
                </div>
              </div>
            </div>
          </div>
        </div>
      ))}
      </div>
    </>
  );
});
