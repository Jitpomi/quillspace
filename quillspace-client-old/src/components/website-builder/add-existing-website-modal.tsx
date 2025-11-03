import { component$, useSignal, $, type QRL } from '@builder.io/qwik';
import { LuX, LuPlus, LuExternalLink } from '@qwikest/icons/lucide';

interface AddExistingWebsiteModalProps {
  isOpen: boolean;
  onClose: QRL<() => void>;
  onAdd: QRL<(website: any) => void>;
}

export const AddExistingWebsiteModal = component$<AddExistingWebsiteModalProps>(({ isOpen, onClose, onAdd }) => {
  const isLoading = useSignal(false);
  const formData = useSignal({
    builder_type: 'wix',
    name: '',
    url: '',
    domain: '',
    external_site_id: '',
  });

  const handleSubmit = $(async (e: Event) => {
    e.preventDefault();
    isLoading.value = true;

    try {
      const response = await fetch('/api/connected-websites/add-existing', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        credentials: 'include',
        body: JSON.stringify(formData.value),
      });

      if (response.ok) {
        const website = await response.json();
        onAdd(website);
        onClose();
        
        // Reset form
        formData.value = {
          builder_type: 'wix',
          name: '',
          url: '',
          domain: '',
          external_site_id: '',
        };
      } else {
        console.error('Failed to add website');
      }
    } catch (error) {
      console.error('Error adding website:', error);
    } finally {
      isLoading.value = false;
    }
  });

  const extractSiteIdFromUrl = $((url: string) => {
    // Extract Wix site ID from URL
    if (url.includes('wixsite.com')) {
      const match = url.match(/\/([a-f0-9-]+)$/);
      if (match) {
        formData.value = { ...formData.value, external_site_id: match[1] };
      }
    }
    
    // Extract domain
    try {
      const urlObj = new URL(url);
      formData.value = { ...formData.value, domain: urlObj.hostname };
    } catch {
      // Invalid URL, ignore
    }
  });

  if (!isOpen) return null;

  return (
    <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div class="bg-white rounded-xl shadow-xl max-w-md w-full mx-4">
        <div class="p-6">
          <div class="flex items-center justify-between mb-6">
            <h3 class="text-lg font-semibold text-gray-900">Add Existing Website</h3>
            <button
              onClick$={onClose}
              class="p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded-lg transition-colors"
            >
              <LuX class="w-5 h-5" />
            </button>
          </div>

          <form onSubmit$={handleSubmit} class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">
                Website Builder
              </label>
              <select
                value={formData.value.builder_type}
                onChange$={(e) => {
                  formData.value = { ...formData.value, builder_type: (e.target as HTMLSelectElement).value };
                }}
                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:border-[#9CAF88]"
              >
                <option value="wix">Wix</option>
                <option value="wordpress">WordPress</option>
                <option value="squarespace">Squarespace</option>
              </select>
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">
                Website Name
              </label>
              <input
                type="text"
                required
                value={formData.value.name}
                onInput$={(e) => {
                  formData.value = { ...formData.value, name: (e.target as HTMLInputElement).value };
                }}
                placeholder="e.g., My Author Website"
                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:border-[#9CAF88]"
              />
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">
                Website URL
              </label>
              <input
                type="url"
                required
                value={formData.value.url}
                onInput$={(e) => {
                  const url = (e.target as HTMLInputElement).value;
                  formData.value = { ...formData.value, url };
                  extractSiteIdFromUrl(url);
                }}
                placeholder="https://yourwebsite.com"
                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:border-[#9CAF88]"
              />
            </div>

            {formData.value.builder_type === 'wix' && (
              <div>
                <label class="block text-sm font-medium text-gray-700 mb-2">
                  Wix Site ID
                </label>
                <input
                  type="text"
                  required
                  value={formData.value.external_site_id}
                  onInput$={(e) => {
                    formData.value = { ...formData.value, external_site_id: (e.target as HTMLInputElement).value };
                  }}
                  placeholder="e.g., 1e4a0091-f1d5-4a4c-a66e-4d09a75b4e9"
                  class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:border-[#9CAF88]"
                />
                <p class="text-xs text-gray-500 mt-1">
                  Found in your Wix dashboard URL or extracted automatically from the website URL
                </p>
              </div>
            )}

            <div class="bg-blue-50 border border-blue-200 rounded-lg p-3">
              <div class="flex items-start gap-2">
                <LuExternalLink class="w-4 h-4 text-blue-600 mt-0.5 flex-shrink-0" />
                <div class="text-sm text-blue-800">
                  <p class="font-medium">Note:</p>
                  <p>This will connect your existing website to QuillSpace for tracking and management. We won't modify your website content.</p>
                </div>
              </div>
            </div>

            <div class="flex gap-3 pt-4">
              <button
                type="submit"
                disabled={isLoading.value}
                class="flex-1 bg-[#9CAF88] text-white px-4 py-2 rounded-lg hover:bg-[#9CAF88]/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
              >
                {isLoading.value ? (
                  <>
                    <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                    Adding...
                  </>
                ) : (
                  <>
                    <LuPlus class="w-4 h-4" />
                    Add Website
                  </>
                )}
              </button>
              <button
                type="button"
                onClick$={onClose}
                class="px-4 py-2 text-gray-600 hover:text-gray-800 transition-colors"
              >
                Cancel
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
  );
});
