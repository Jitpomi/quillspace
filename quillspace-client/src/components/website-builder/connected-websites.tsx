import { component$, useSignal, $ } from '@builder.io/qwik';
import { LuExternalLink, LuSettings, LuRefreshCw, LuTrash2, LuGlobe, LuPencil, LuEye, LuCopy } from '@qwikest/icons/lucide';
import type { ConnectedWebsite } from '~/types/website-builders';
import {globalAction$} from "@builder.io/qwik-city";
import {getTenantInfo, getUserInfo} from "~/utils/auth";

interface ConnectedWebsitesProps {
  // Make websites optional since we'll fetch them via action
  websites?: ConnectedWebsite[];
}

export const useUsersWebsite = globalAction$(async (_, requestEventAction) => {
  const { cookie } = requestEventAction;
  const user = await getUserInfo(cookie);
  const tenant = await getTenantInfo(cookie);

  if (!user || !tenant) {
    throw new Error('User not authenticated');
  }

  try {
    // Call the backend API to get connected websites
    const response = await fetch(`${process.env.BACKEND_URL || 'http://localhost:3001'}/api/connected-websites/websites`, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${user.token}`,
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      throw new Error(`Failed to fetch websites: ${response.statusText}`);
    }

    const data = await response.json();
    return { success: true, websites: data.websites };
  } catch (error) {
    console.error('Error fetching connected websites:', error);
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Unknown error',
      websites: []
    };
  }
});

export const ConnectedWebsites = component$<ConnectedWebsitesProps>(({ websites = [] }) => {
  const websiteList = useSignal(websites);
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
          class="bg-white border border-gray-200 rounded-lg p-4 md:p-6 hover:shadow-md transition-shadow"
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
                <h3 class="text-lg font-medium text-gray-900">
                  {website.name}
                </h3>
                <span class={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium ${getStatusColor(website.status)}`}>
                  {getStatusIcon(website.status)}
                  {website.status === 'active' ? 'Live' : website.status.charAt(0).toUpperCase() + website.status.slice(1)}
                </span>
              </div>
              
              <div class="text-sm text-gray-600 mb-4 break-words">
                {website.url ? website.url.replace('https://', '') : 'No domain'}
              </div>
              
              <a
                href={`/editor/wix/${(website as any).external_site_id}`}
                class="w-full inline-flex items-center justify-center gap-2 px-4 py-2.5 bg-[#9CAF88] text-white rounded-lg transition-colors hover:bg-[#8BA079] font-medium"
              >
                <LuPencil class="w-4 h-4" />
                Edit Site
              </a>
            </div>
          </div>

          {/* Desktop Layout */}
          <div class="hidden md:flex items-center gap-6">
            {/* Website Thumbnail */}
            <div class="w-24 h-16 bg-gray-100 rounded overflow-hidden flex-shrink-0">
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
                  <LuGlobe class="w-5 h-5 text-gray-400" />
                </div>
              )}
            </div>

            {/* Website Info */}
            <div class="flex-1 min-w-0">
              <div class="flex items-center justify-between">
                <div class="flex-1">
                  <div class="flex items-center gap-3 mb-3">
                    <h3 class="text-lg font-medium text-gray-900">
                      {website.name}
                    </h3>
                    <span class={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium ${getStatusColor(website.status)}`}>
                      {getStatusIcon(website.status)}
                      {website.status === 'active' ? 'Live' : website.status.charAt(0).toUpperCase() + website.status.slice(1)}
                    </span>
                  </div>
                  
                  <div class="grid grid-cols-3 lg:grid-cols-5 gap-6 text-sm">
                    <div>
                      <div class="text-gray-500 text-xs font-medium mb-1">BUILDER</div>
                      <div class="text-gray-900 capitalize">{(website as any).builder_type || 'Unknown'}</div>
                    </div>
                    
                    <div>
                      <div class="text-gray-500 text-xs font-medium mb-1">DOMAIN</div>
                      <div class="text-gray-900 truncate">
                        {website.url ? website.url.replace('https://', '') : 'No domain'}
                      </div>
                    </div>
                    
                    <div>
                      <div class="text-gray-500 text-xs font-medium mb-1">ACTIONS</div>
                      <a
                        href={`/editor/wix/${(website as any).external_site_id}`}
                        class="text-[#9CAF88] hover:text-[#8BA079] text-sm transition-colors"
                      >
                        Edit Site
                      </a>
                    </div>
                    
                    <div class="lg:block hidden">
                      <div class="text-gray-500 text-xs font-medium mb-1">BUSINESS EMAIL</div>
                      <button class="text-[#9CAF88] hover:text-[#8BA079] text-sm transition-colors text-left">Connect</button>
                    </div>
                    
                    <div class="lg:block hidden">
                      <div class="text-gray-500 text-xs font-medium mb-1">BUSINESS PHONE</div>
                      <button class="text-[#9CAF88] hover:text-[#8BA079] text-sm transition-colors text-left">Connect</button>
                    </div>
                  </div>
                </div>
                
                {/* Action Menu */}
                <div class="flex items-center gap-1 ml-4">
                  <button class="p-2 text-gray-400 hover:text-gray-600 rounded transition-colors" title="Settings">
                    <LuSettings class="w-4 h-4" />
                  </button>
                  <button class="p-2 text-gray-400 hover:text-gray-600 rounded transition-colors" title="Refresh">
                    <LuRefreshCw class="w-4 h-4" />
                  </button>
                  <button class="p-2 text-gray-400 hover:text-gray-600 rounded transition-colors" title="View Website">
                    <LuEye class="w-4 h-4" />
                  </button>
                  <button class="p-2 text-gray-400 hover:text-red-600 rounded transition-colors" title="Delete">
                    <LuTrash2 class="w-4 h-4" />
                  </button>
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
