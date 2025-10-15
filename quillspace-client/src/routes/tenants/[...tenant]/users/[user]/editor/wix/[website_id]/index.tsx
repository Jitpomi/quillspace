import { component$, useSignal, useTask$, $ } from '@builder.io/qwik';
import { useLocation } from '@builder.io/qwik-city';
import { LuSave, LuEye, LuArrowLeft, LuGlobe, LuLoader } from '@qwikest/icons/lucide';

export default component$(() => {
  const location = useLocation();
  const websiteId = location.params.website_id;
  
  const isLoading = useSignal(true);
  const isSaving = useSignal(false);
  const isPublishing = useSignal(false);
  const error = useSignal<string | null>(null);
  const websiteData = useSignal<any>(null);
  const puckData = useSignal<any>(null);
  const selectedPageId = useSignal<string>('');

  // Load website data and pages
  useTask$(async () => {
    try {
      // Get website details
      const websiteResponse = await fetch(`/api/connected-websites/${websiteId}`, {
        credentials: 'include',
      });
      
      if (!websiteResponse.ok) {
        throw new Error('Failed to load website');
      }
      
      const website = await websiteResponse.json();
      websiteData.value = website;
      
      // Load first page for editing (in real implementation, would show page selector)
      if (website.metadata?.wix_site_id) {
        const pageResponse = await fetch(
          `/api/connected-websites/wix/${website.metadata.wix_site_id}/pages/home/edit`,
          { credentials: 'include' }
        );
        
        if (pageResponse.ok) {
          const pageData = await pageResponse.json();
          puckData.value = pageData;
          selectedPageId.value = 'home';
        }
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load website';
    } finally {
      isLoading.value = false;
    }
  });

  const handleSave = $(async () => {
    if (!websiteData.value?.metadata?.wix_site_id || !selectedPageId.value) return;
    
    isSaving.value = true;
    error.value = null;

    try {
      const response = await fetch(
        `/api/connected-websites/wix/${websiteData.value.metadata.wix_site_id}/pages/${selectedPageId.value}/save`,
        {
          method: 'PUT',
          headers: { 'Content-Type': 'application/json' },
          credentials: 'include',
          body: JSON.stringify(puckData.value),
        }
      );

      if (!response.ok) {
        throw new Error('Failed to save changes');
      }

      // Show success message
      console.log('Changes saved successfully');
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to save changes';
    } finally {
      isSaving.value = false;
    }
  });

  const handlePublish = $(async () => {
    if (!websiteData.value?.metadata?.wix_site_id) return;
    
    isPublishing.value = true;
    error.value = null;

    try {
      const response = await fetch(
        `/api/connected-websites/wix/${websiteData.value.metadata.wix_site_id}/publish`,
        {
          method: 'POST',
          credentials: 'include',
        }
      );

      if (!response.ok) {
        throw new Error('Failed to publish website');
      }

      // Show success message
      console.log('Website published successfully');
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to publish website';
    } finally {
      isPublishing.value = false;
    }
  });

  if (isLoading.value) {
    return (
      <div class="min-h-screen bg-gray-50 flex items-center justify-center">
        <div class="text-center">
          <LuLoader class="w-8 h-8 animate-spin text-[#9CAF88] mx-auto mb-4" />
          <p class="text-gray-600">Loading your Wix website...</p>
        </div>
      </div>
    );
  }

  if (error.value && !websiteData.value) {
    return (
      <div class="min-h-screen bg-gray-50 flex items-center justify-center">
        <div class="text-center">
          <div class="bg-red-50 border border-red-200 rounded-lg p-6 max-w-md">
            <h3 class="text-red-800 font-medium mb-2">Failed to Load Website</h3>
            <p class="text-red-600 text-sm">{error.value}</p>
            <a
              href="/websites"
              class="inline-block mt-4 text-[#9CAF88] hover:text-[#9CAF88]/80 font-medium"
            >
              ← Back to Websites
            </a>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div class="min-h-screen bg-white">
      {/* Header */}
      <div class="border-b border-gray-200 bg-white">
        <div class="flex items-center justify-between px-6 py-4">
          <div class="flex items-center gap-4">
            <a
              href="/websites"
              class="flex items-center justify-center w-8 h-8 rounded-lg border border-gray-200 hover:border-gray-300 hover:bg-gray-50 transition-colors"
            >
              <LuArrowLeft class="w-4 h-4 text-gray-600" />
            </a>
            
            <div class="flex items-center gap-3">
              <div class="flex items-center justify-center w-8 h-8 bg-blue-100 rounded-lg">
                <LuGlobe class="w-4 h-4 text-blue-600" />
              </div>
              <div>
                <h1 class="text-lg font-semibold text-gray-900">
                  {websiteData.value?.name || 'Wix Website'}
                </h1>
                <p class="text-sm text-gray-500">
                  Editing via QuillSpace • Wix Integration
                </p>
              </div>
            </div>
          </div>

          <div class="flex items-center gap-3">
            {error.value && (
              <div class="text-sm text-red-600 bg-red-50 px-3 py-1 rounded-lg">
                {error.value}
              </div>
            )}
            
            <button
              onClick$={handleSave}
              disabled={isSaving.value}
              class="flex items-center gap-2 bg-gray-100 text-gray-700 px-4 py-2 rounded-lg hover:bg-gray-200 transition-colors disabled:opacity-50"
            >
              {isSaving.value ? (
                <LuLoader class="w-4 h-4 animate-spin" />
              ) : (
                <LuSave class="w-4 h-4" />
              )}
              {isSaving.value ? 'Saving...' : 'Save Draft'}
            </button>
            
            <button
              onClick$={handlePublish}
              disabled={isPublishing.value}
              class="flex items-center gap-2 bg-[#9CAF88] text-white px-4 py-2 rounded-lg hover:bg-[#9CAF88]/90 transition-colors disabled:opacity-50"
            >
              {isPublishing.value ? (
                <LuLoader class="w-4 h-4 animate-spin" />
              ) : (
                <LuGlobe class="w-4 h-4" />
              )}
              {isPublishing.value ? 'Publishing...' : 'Publish to Wix'}
            </button>
            
            {websiteData.value?.url && (
              <a
                href={websiteData.value.url}
                target="_blank"
                rel="noopener noreferrer"
                class="flex items-center gap-2 border border-gray-300 text-gray-700 px-4 py-2 rounded-lg hover:bg-gray-50 transition-colors"
              >
                <LuEye class="w-4 h-4" />
                Preview Live
              </a>
            )}
          </div>
        </div>
      </div>

      {/* Editor Content */}
      <div class="flex-1">
        {puckData.value ? (
          <div class="h-[calc(100vh-73px)]">
            {/* Puck Editor Integration */}
            <div class="w-full h-full bg-gray-50 flex items-center justify-center">
              <div class="text-center">
                <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-8 max-w-md">
                  <div class="w-16 h-16 bg-[#9CAF88]/10 rounded-xl flex items-center justify-center mx-auto mb-4">
                    <LuGlobe class="w-8 h-8 text-[#9CAF88]" />
                  </div>
                  <h3 class="text-lg font-semibold text-gray-900 mb-2">
                    Wix Editor Integration
                  </h3>
                  <p class="text-gray-600 text-sm mb-4">
                    The Puck visual editor will be integrated here to allow editing of Wix website content.
                  </p>
                  <div class="text-xs text-gray-500 bg-gray-50 p-3 rounded-lg">
                    <strong>Technical Note:</strong> This would load the Puck editor component with 
                    the converted Wix page data, allowing visual editing that gets saved back to Wix via API.
                  </div>
                </div>
              </div>
            </div>
          </div>
        ) : (
          <div class="h-[calc(100vh-73px)] flex items-center justify-center">
            <div class="text-center">
              <LuLoader class="w-8 h-8 animate-spin text-[#9CAF88] mx-auto mb-4" />
              <p class="text-gray-600">Loading page content...</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
});
