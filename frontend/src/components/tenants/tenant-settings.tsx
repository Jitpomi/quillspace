import { component$, useSignal, $ } from '@builder.io/qwik';
import { LuSettings, LuSave, LuBuilding, LuGlobe, LuPalette, LuShield } from '@qwikest/icons/lucide';

interface TenantSettings {
  id: string;
  name: string;
  slug: string;
  settings: {
    branding: {
      primary_color: string;
      logo_url?: string;
      favicon_url?: string;
    };
    features: {
      analytics_enabled: boolean;
      comments_enabled: boolean;
      seo_enabled: boolean;
    };
    security: {
      two_factor_required: boolean;
      password_policy: 'basic' | 'strong' | 'enterprise';
    };
  };
  created_at: string;
  updated_at: string;
  is_active: boolean;
}

interface TenantSettingsProps {
  tenant: TenantSettings;
  onUpdateSettings: (_settings: any) => void;
}

export const TenantSettingsComponent = component$<TenantSettingsProps>(({ tenant, onUpdateSettings }) => {
  const activeTab = useSignal('general');
  const settings = useSignal(tenant.settings);

  const handleSave = $(() => {
    setTimeout(() => {
      onUpdateSettings(settings.value);
    }, 0);
  });

  const tabs = [
    { id: 'general', label: 'General' },
    { id: 'branding', label: 'Branding' },
    { id: 'features', label: 'Features' },
    { id: 'security', label: 'Security' },
  ];

  const getTabIcon = (tabId: string) => {
    switch (tabId) {
      case 'general': return <LuBuilding class="w-4 h-4" />;
      case 'branding': return <LuPalette class="w-4 h-4" />;
      case 'features': return <LuGlobe class="w-4 h-4" />;
      case 'security': return <LuShield class="w-4 h-4" />;
      default: return null;
    }
  };

  return (
    <div class="space-y-6">
      {/* Header */}
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          <LuSettings class="w-8 h-8 text-indigo-600" />
          <h2 class="text-3xl font-bold text-gray-900">Tenant Settings</h2>
        </div>
        <button
          onClick$={handleSave}
          class="bg-indigo-600 hover:bg-indigo-700 text-white px-4 py-2 rounded-lg font-semibold transition-colors flex items-center gap-2"
        >
          <LuSave class="w-4 h-4" />
          Save Changes
        </button>
      </div>

      <div class="bg-white rounded-xl shadow-lg overflow-hidden">
        {/* Tabs */}
        <div class="border-b border-gray-200">
          <nav class="flex space-x-8 px-6">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick$={() => activeTab.value = tab.id}
                class={`py-4 px-1 border-b-2 font-medium text-sm flex items-center gap-2 ${
                  activeTab.value === tab.id
                    ? 'border-indigo-500 text-indigo-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                }`}
              >
                {getTabIcon(tab.id)}
                {tab.label}
              </button>
            ))}
          </nav>
        </div>

        {/* Tab Content */}
        <div class="p-6">
          {activeTab.value === 'general' && (
            <div class="space-y-6">
              <div>
                <label class="block text-sm font-medium text-gray-700 mb-2">Tenant Name</label>
                <input
                  type="text"
                  value={tenant.name}
                  class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
                  placeholder="Enter tenant name"
                />
              </div>
              <div>
                <label class="block text-sm font-medium text-gray-700 mb-2">Slug</label>
                <input
                  type="text"
                  value={tenant.slug}
                  class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
                  placeholder="tenant-slug"
                />
              </div>
              <div class="flex items-center gap-3">
                <input
                  type="checkbox"
                  id="is_active"
                  checked={tenant.is_active}
                  class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
                />
                <label for="is_active" class="text-sm font-medium text-gray-700">Active Tenant</label>
              </div>
            </div>
          )}

          {activeTab.value === 'branding' && (
            <div class="space-y-6">
              <div>
                <label class="block text-sm font-medium text-gray-700 mb-2">Primary Color</label>
                <div class="flex items-center gap-3">
                  <input
                    type="color"
                    value={settings.value.branding.primary_color}
                    onChange$={(e) => {
                      settings.value = {
                        ...settings.value,
                        branding: {
                          ...settings.value.branding,
                          primary_color: (e.target as HTMLInputElement).value
                        }
                      };
                    }}
                    class="w-12 h-10 border border-gray-300 rounded cursor-pointer"
                  />
                  <input
                    type="text"
                    value={settings.value.branding.primary_color}
                    class="flex-1 border border-gray-300 rounded-lg px-3 py-2 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
                    placeholder="#3B82F6"
                  />
                </div>
              </div>
              <div>
                <label class="block text-sm font-medium text-gray-700 mb-2">Logo URL</label>
                <input
                  type="url"
                  value={settings.value.branding.logo_url || ''}
                  class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
                  placeholder="https://example.com/logo.png"
                />
              </div>
              <div>
                <label class="block text-sm font-medium text-gray-700 mb-2">Favicon URL</label>
                <input
                  type="url"
                  value={settings.value.branding.favicon_url || ''}
                  class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
                  placeholder="https://example.com/favicon.ico"
                />
              </div>
            </div>
          )}

          {activeTab.value === 'features' && (
            <div class="space-y-6">
              <div class="flex items-center justify-between">
                <div>
                  <h4 class="text-sm font-medium text-gray-900">Analytics</h4>
                  <p class="text-sm text-gray-500">Enable real-time analytics tracking</p>
                </div>
                <input
                  type="checkbox"
                  checked={settings.value.features.analytics_enabled}
                  onChange$={(e) => {
                    settings.value = {
                      ...settings.value,
                      features: {
                        ...settings.value.features,
                        analytics_enabled: (e.target as HTMLInputElement).checked
                      }
                    };
                  }}
                  class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
                />
              </div>
              <div class="flex items-center justify-between">
                <div>
                  <h4 class="text-sm font-medium text-gray-900">Comments</h4>
                  <p class="text-sm text-gray-500">Allow comments on published content</p>
                </div>
                <input
                  type="checkbox"
                  checked={settings.value.features.comments_enabled}
                  class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
                />
              </div>
              <div class="flex items-center justify-between">
                <div>
                  <h4 class="text-sm font-medium text-gray-900">SEO</h4>
                  <p class="text-sm text-gray-500">Enable SEO optimization features</p>
                </div>
                <input
                  type="checkbox"
                  checked={settings.value.features.seo_enabled}
                  class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
                />
              </div>
            </div>
          )}

          {activeTab.value === 'security' && (
            <div class="space-y-6">
              <div class="flex items-center justify-between">
                <div>
                  <h4 class="text-sm font-medium text-gray-900">Two-Factor Authentication</h4>
                  <p class="text-sm text-gray-500">Require 2FA for all users</p>
                </div>
                <input
                  type="checkbox"
                  checked={settings.value.security.two_factor_required}
                  class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
                />
              </div>
              <div>
                <label class="block text-sm font-medium text-gray-700 mb-2">Password Policy</label>
                <select
                  value={settings.value.security.password_policy}
                  class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
                >
                  <option value="basic">Basic (8+ characters)</option>
                  <option value="strong">Strong (12+ characters, mixed case, numbers)</option>
                  <option value="enterprise">Enterprise (16+ characters, symbols required)</option>
                </select>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
});
