import { component$, useSignal, useTask$, $ } from '@builder.io/qwik';
import { LuPlus, LuPencil, LuTrash2, LuExternalLink, LuUser, LuGlobe } from '@qwikest/icons/lucide';

interface UserWixSite {
  id: string;
  user_id: string;
  wix_site_id: string;
  site_name: string;
  wix_url: string;
  custom_domain?: string;
  project_status: string;
  service_type: string;
  client_can_edit: boolean;
  client_can_publish: boolean;
  created_at: string;
  updated_at: string;
}

export default component$(() => {
  const sites = useSignal<UserWixSite[]>([]);
  const isLoading = useSignal(true);
  const showAddModal = useSignal(false);
  const newSite = useSignal({
    user_email: '',
    wix_site_id: '',
    site_name: '',
    wix_url: '',
    custom_domain: '',
    project_status: 'active',
    service_type: 'build_and_manage',
    client_can_edit: true,
    client_can_publish: true,
  });

  // Load existing mappings
  useTask$(async () => {
    try {
      const response = await fetch('/api/admin/user-wix-sites', {
        credentials: 'include',
      });
      
      if (response.ok) {
        const data = await response.json();
        sites.value = data;
      }
    } catch (error) {
      console.error('Failed to load user wix sites:', error);
    } finally {
      isLoading.value = false;
    }
  });

  const handleAddSite = $(async () => {
    try {
      const response = await fetch('/api/admin/user-wix-sites', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify(newSite.value),
      });

      if (response.ok) {
        const site = await response.json();
        sites.value = [site, ...sites.value];
        showAddModal.value = false;
        
        // Reset form
        newSite.value = {
          user_email: '',
          wix_site_id: '',
          site_name: '',
          wix_url: '',
          custom_domain: '',
          project_status: 'active',
          service_type: 'build_and_manage',
          client_can_edit: true,
          client_can_publish: true,
        };
      }
    } catch (error) {
      console.error('Failed to add site:', error);
    }
  });

  const handleDeleteSite = $(async (siteId: string) => {
    if (!confirm('Are you sure you want to delete this mapping?')) return;

    try {
      const response = await fetch(`/api/admin/user-wix-sites/${siteId}`, {
        method: 'DELETE',
        credentials: 'include',
      });

      if (response.ok) {
        sites.value = sites.value.filter(site => site.id !== siteId);
      }
    } catch (error) {
      console.error('Failed to delete site:', error);
    }
  });

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active': return 'bg-green-100 text-green-800';
      case 'development': return 'bg-blue-100 text-blue-800';
      case 'review': return 'bg-yellow-100 text-yellow-800';
      case 'maintenance': return 'bg-gray-100 text-gray-800';
      default: return 'bg-gray-100 text-gray-800';
    }
  };

  if (isLoading.value) {
    return (
      <div class="min-h-screen bg-gray-50 flex items-center justify-center">
        <div class="text-center">
          <div class="animate-spin w-8 h-8 border-2 border-[#9CAF88] border-t-transparent rounded-full mx-auto mb-4"></div>
          <p class="text-gray-600">Loading user Wix sites...</p>
        </div>
      </div>
    );
  }

  return (
    <div class="min-h-screen bg-gray-50">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Header */}
        <div class="flex items-center justify-between mb-8">
          <div>
            <h1 class="text-2xl font-bold text-gray-900">User Wix Sites</h1>
            <p class="text-gray-600 mt-1">
              Manage mappings between users and their Wix websites
            </p>
          </div>
          
          <button
            onClick$={() => showAddModal.value = true}
            class="inline-flex items-center gap-2 bg-[#9CAF88] text-white px-4 py-2 rounded-lg hover:bg-[#9CAF88]/90 transition-colors"
          >
            <LuPlus class="w-4 h-4" />
            Add Site Mapping
          </button>
        </div>

        {/* Sites List */}
        <div class="bg-white rounded-xl shadow-sm border border-gray-200">
          {sites.value.length === 0 ? (
            <div class="text-center py-12">
              <LuGlobe class="w-12 h-12 text-gray-400 mx-auto mb-4" />
              <h3 class="text-lg font-medium text-gray-900 mb-2">No site mappings yet</h3>
              <p class="text-gray-600 mb-6">
                Add your first user-to-Wix-site mapping to get started
              </p>
              <button
                onClick$={() => showAddModal.value = true}
                class="inline-flex items-center gap-2 bg-[#9CAF88] text-white px-4 py-2 rounded-lg hover:bg-[#9CAF88]/90 transition-colors"
              >
                <LuPlus class="w-4 h-4" />
                Add Site Mapping
              </button>
            </div>
          ) : (
            <div class="overflow-x-auto">
              <table class="w-full">
                <thead class="bg-gray-50 border-b border-gray-200">
                  <tr>
                    <th class="text-left px-6 py-3 text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Site
                    </th>
                    <th class="text-left px-6 py-3 text-xs font-medium text-gray-500 uppercase tracking-wider">
                      User
                    </th>
                    <th class="text-left px-6 py-3 text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Status
                    </th>
                    <th class="text-left px-6 py-3 text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Permissions
                    </th>
                    <th class="text-left px-6 py-3 text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Actions
                    </th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-gray-200">
                  {sites.value.map((site) => (
                    <tr key={site.id} class="hover:bg-gray-50">
                      <td class="px-6 py-4">
                        <div>
                          <div class="font-medium text-gray-900">{site.site_name}</div>
                          <div class="text-sm text-gray-500">
                            <a
                              href={site.wix_url}
                              target="_blank"
                              rel="noopener noreferrer"
                              class="inline-flex items-center gap-1 text-blue-600 hover:text-blue-800"
                            >
                              {site.custom_domain || site.wix_url.replace('https://', '')}
                              <LuExternalLink class="w-3 h-3" />
                            </a>
                          </div>
                          <div class="text-xs text-gray-400 mt-1">
                            Wix ID: {site.wix_site_id}
                          </div>
                        </div>
                      </td>
                      
                      <td class="px-6 py-4">
                        <div class="flex items-center gap-2">
                          <LuUser class="w-4 h-4 text-gray-400" />
                          <span class="text-sm text-gray-900">{site.user_id}</span>
                        </div>
                      </td>
                      
                      <td class="px-6 py-4">
                        <span class={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getStatusColor(site.project_status)}`}>
                          {site.project_status}
                        </span>
                        <div class="text-xs text-gray-500 mt-1">
                          {site.service_type.replace('_', ' ')}
                        </div>
                      </td>
                      
                      <td class="px-6 py-4">
                        <div class="text-sm">
                          {site.client_can_edit && (
                            <span class="inline-flex items-center px-2 py-1 rounded text-xs bg-green-100 text-green-800 mr-1">
                              Can Edit
                            </span>
                          )}
                          {site.client_can_publish && (
                            <span class="inline-flex items-center px-2 py-1 rounded text-xs bg-blue-100 text-blue-800">
                              Can Publish
                            </span>
                          )}
                        </div>
                      </td>
                      
                      <td class="px-6 py-4">
                        <div class="flex items-center gap-2">
                          <button
                            class="p-1 text-gray-400 hover:text-blue-600 hover:bg-blue-50 rounded transition-colors"
                            title="Edit mapping"
                          >
                            <LuPencil class="w-4 h-4" />
                          </button>
                          
                          <button
                            onClick$={() => handleDeleteSite(site.id)}
                            class="p-1 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded transition-colors"
                            title="Delete mapping"
                          >
                            <LuTrash2 class="w-4 h-4" />
                          </button>
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>

        {/* Add Site Modal */}
        {showAddModal.value && (
          <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
            <div class="bg-white rounded-xl shadow-xl max-w-md w-full mx-4">
              <div class="p-6">
                <h3 class="text-lg font-semibold text-gray-900 mb-4">
                  Add User Wix Site Mapping
                </h3>

                <div class="space-y-4">
                  <div>
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                      User Email
                    </label>
                    <input
                      type="email"
                      value={newSite.value.user_email || ''}
                      onInput$={(e) => {
                        newSite.value = { ...newSite.value, user_email: (e.target as HTMLInputElement).value };
                      }}
                      placeholder="yasin@example.com"
                      class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:border-[#9CAF88]"
                    />
                    <p class="text-xs text-gray-500 mt-1">
                      We'll find the user by email automatically
                    </p>
                  </div>

                  <div>
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                      Wix Site ID
                    </label>
                    <input
                      type="text"
                      value={newSite.value.wix_site_id}
                      onInput$={(e) => {
                        newSite.value = { ...newSite.value, wix_site_id: (e.target as HTMLInputElement).value };
                      }}
                      placeholder="1e4a0091-f4d5-4a4c-a66e-4d09a75b4e9"
                      class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:border-[#9CAF88]"
                    />
                    <p class="text-xs text-gray-500 mt-1">
                      Copy from Wix editor URL: /edit/1e4a0091-f4d5-4a4c-a66e-4d09a75b4e9
                    </p>
                  </div>

                  <div>
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                      Site Name
                    </label>
                    <input
                      type="text"
                      value={newSite.value.site_name}
                      onInput$={(e) => {
                        newSite.value = { ...newSite.value, site_name: (e.target as HTMLInputElement).value };
                      }}
                      placeholder="Yasin Kakande"
                      class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:border-[#9CAF88]"
                    />
                  </div>

                  <div>
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                      Wix URL
                    </label>
                    <input
                      type="url"
                      value={newSite.value.wix_url}
                      onInput$={(e) => {
                        newSite.value = { ...newSite.value, wix_url: (e.target as HTMLInputElement).value };
                      }}
                      placeholder="https://yasinkakande.wixsite.com/yasin-kakande"
                      class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:border-[#9CAF88]"
                    />
                  </div>

                  <div class="grid grid-cols-2 gap-4">
                    <div>
                      <label class="block text-sm font-medium text-gray-700 mb-2">
                        Status
                      </label>
                      <select
                        value={newSite.value.project_status}
                        onChange$={(e) => {
                          newSite.value = { ...newSite.value, project_status: (e.target as HTMLSelectElement).value };
                        }}
                        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:border-[#9CAF88]"
                      >
                        <option value="active">Active</option>
                        <option value="development">Development</option>
                        <option value="review">Review</option>
                        <option value="maintenance">Maintenance</option>
                      </select>
                    </div>

                    <div>
                      <label class="block text-sm font-medium text-gray-700 mb-2">
                        Service Type
                      </label>
                      <select
                        value={newSite.value.service_type}
                        onChange$={(e) => {
                          newSite.value = { ...newSite.value, service_type: (e.target as HTMLSelectElement).value };
                        }}
                        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:border-[#9CAF88]"
                      >
                        <option value="build_and_manage">Build & Manage</option>
                        <option value="consultation_only">Consultation Only</option>
                        <option value="maintenance_only">Maintenance Only</option>
                      </select>
                    </div>
                  </div>

                  <div class="flex items-center gap-4">
                    <label class="flex items-center">
                      <input
                        type="checkbox"
                        checked={newSite.value.client_can_edit}
                        onChange$={(e) => {
                          newSite.value = { ...newSite.value, client_can_edit: (e.target as HTMLInputElement).checked };
                        }}
                        class="rounded border-gray-300 text-[#9CAF88] focus:ring-[#9CAF88]"
                      />
                      <span class="ml-2 text-sm text-gray-700">Client can edit</span>
                    </label>

                    <label class="flex items-center">
                      <input
                        type="checkbox"
                        checked={newSite.value.client_can_publish}
                        onChange$={(e) => {
                          newSite.value = { ...newSite.value, client_can_publish: (e.target as HTMLInputElement).checked };
                        }}
                        class="rounded border-gray-300 text-[#9CAF88] focus:ring-[#9CAF88]"
                      />
                      <span class="ml-2 text-sm text-gray-700">Client can publish</span>
                    </label>
                  </div>
                </div>

                <div class="flex gap-3 mt-6">
                  <button
                    onClick$={handleAddSite}
                    class="flex-1 bg-[#9CAF88] text-white px-4 py-2 rounded-lg hover:bg-[#9CAF88]/90 transition-colors"
                  >
                    Add Mapping
                  </button>
                  <button
                    onClick$={() => showAddModal.value = false}
                    class="px-4 py-2 text-gray-600 hover:text-gray-800 transition-colors"
                  >
                    Cancel
                  </button>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
});
