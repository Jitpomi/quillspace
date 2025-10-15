import { component$, useSignal, $, noSerialize } from '@builder.io/qwik';
import { LuGlobe, LuPlus, LuSettings, LuExternalLink, LuRocket, LuPalette, LuFileText, LuSquare } from '@qwikest/icons/lucide';
import { ConnectedWebsites } from '~/components/website-builder/connected-websites';
import { WebsiteBuilderService } from '~/services/website-builder.service';
import type { WebsiteBuilder, ConnectedWebsite, BuilderType } from '~/types/website-builders';

export default component$(() => {
  const selectedBuilder = useSignal<string>('');
  const showCredentials = useSignal(false);
  const connectedWebsites = useSignal<ConnectedWebsite[]>([]);
  const isLoading = useSignal(false);
  const credentials = useSignal<Record<string, string>>({});
  
  const builderService = noSerialize(WebsiteBuilderService.getInstance());

  // Helper function to render the appropriate icon
  const renderIcon = (iconName: string, className: string = "w-8 h-8") => {
    // If it's an image path, render an img element
    if (iconName.startsWith('/') || iconName.includes('.')) {
      return <img src={iconName} alt="Builder logo" class={`${className} object-contain`} />;
    }
    
    // Otherwise render Lucide icons
    switch (iconName) {
      case 'LuRocket':
        return <LuRocket class={className} />;
      case 'LuPalette':
        return <LuPalette class={className} />;
      case 'LuFileText':
        return <LuFileText class={className} />;
      case 'LuSquare':
        return <LuSquare class={className} />;
      default:
        return <LuGlobe class={className} />;
    }
  };

  const builders: WebsiteBuilder[] = [
    {
      id: 'jflux',
      name: 'JFlux',
      description: 'Our powerful visual website builder with advanced templates',
      icon: 'LuRocket',
      color: 'from-[#9CAF88] to-[#7A9B6E]',
      isConnected: false,
      authType: 'native',
      requiredFields: []
    },
    {
      id: 'wix',
      name: 'Wix',
      description: 'Connect your existing Wix website and manage through QuillSpace',
      icon: '/wix.png',
      color: 'from-orange-500 to-red-500',
      isConnected: false,
      authType: 'api_key',
      requiredFields: ['apiKey', 'siteId'],
      apiEndpoint: 'https://www.wixapis.com'
    },
    {
      id: 'wordpress',
      name: 'WordPress',
      description: 'Integrate your WordPress site for unified management',
      icon: '/wp.png',
      color: 'from-[#6B8E5A] to-[#5A7A49]',
      isConnected: false,
      authType: 'username_password',
      requiredFields: ['username', 'password', 'siteUrl']
    },
    {
      id: 'squarespace',
      name: 'Squarespace',
      description: 'Connect your Squarespace site and publish from QuillSpace',
      icon: '/squarespace.webp',
      color: 'from-gray-700 to-black',
      isConnected: false,
      authType: 'api_key',
      requiredFields: ['apiKey'],
      apiEndpoint: 'https://api.squarespace.com'
    }
  ];

  const handleBuilderSelect = $((builderId: string) => {
    selectedBuilder.value = builderId;
    showCredentials.value = true;
  });

  const handleConnect = $(async () => {
    if (!selectedBuilder.value || !builderService) return;
    
    isLoading.value = true;
    
    try {
      if (selectedBuilder.value === 'jflux') {
        // For JFlux, just redirect to the builder
        showCredentials.value = false;
        selectedBuilder.value = '';
        // TODO: Navigate to JFlux builder
      } else {
        // For external builders, test connection and save credentials
        const success = await builderService.testConnection(
          selectedBuilder.value as BuilderType, 
          credentials.value
        );
        
        if (success) {
          await builderService.connectBuilder(
            selectedBuilder.value as BuilderType, 
            credentials.value
          );
          
          // Refresh connected websites
          const websites = await builderService.getConnectedWebsites();
          connectedWebsites.value = websites;
          
          showCredentials.value = false;
          selectedBuilder.value = '';
          credentials.value = {};
        } else {
          alert('Failed to connect. Please check your credentials.');
        }
      }
    } catch (error) {
      console.error('Connection error:', error);
      alert('Failed to connect. Please try again.');
    } finally {
      isLoading.value = false;
    }
  });

  return (
    <div class="max-w-6xl mx-auto p-6">
      {/* Header */}
      <div class="mb-8">
        <div class="flex items-center gap-3 mb-4">
          <LuGlobe class="w-8 h-8 text-[#9CAF88]" />
          <h1 class="text-3xl font-bold text-gray-900">Your Websites</h1>
        </div>
        <p class="text-gray-600 text-lg">
          Choose your preferred website builder and manage all your sites through QuillSpace
        </p>
      </div>

      {!showCredentials.value ? (
        <>
          {/* Website Builder Selection */}
          <div class="mb-8">
            <h2 class="text-xl font-semibold text-gray-900 mb-6">Connect Your Website Builder</h2>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
              {builders.map((builder) => (
                <div
                  key={builder.id}
                  class="group relative overflow-hidden rounded-xl border border-gray-200 hover:border-gray-300 transition-all duration-200 cursor-pointer hover:shadow-lg"
                  onClick$={() => handleBuilderSelect(builder.id)}
                >
                  <div class={`absolute inset-0 bg-gradient-to-br ${builder.color} opacity-5 group-hover:opacity-10 transition-opacity`}></div>
                  
                  <div class="relative p-6">
                    <div class="flex items-start justify-between mb-4">
                      <div class="flex items-center gap-3">
                        <div class="flex items-center justify-center w-12 h-12 bg-white rounded-lg shadow-sm border border-gray-200">
                          {renderIcon(builder.icon, "w-8 h-8")}
                        </div>
                        <div>
                          <h3 class="text-lg font-semibold text-gray-900">{builder.name}</h3>
                          {builder.isConnected && (
                            <span class="inline-flex items-center gap-1 text-sm text-green-600 font-medium">
                              <div class="w-2 h-2 bg-green-500 rounded-full"></div>
                              Connected
                            </span>
                          )}
                        </div>
                      </div>
                      <LuPlus class="w-5 h-5 text-gray-400 group-hover:text-gray-600 transition-colors" />
                    </div>
                    
                    <p class="text-gray-600 text-sm leading-relaxed">{builder.description}</p>
                    
                    <div class="mt-4 flex items-center justify-between">
                      <div class="text-sm text-gray-500">
                        {builder.id === 'jflux' ? 'Native Builder' : 'External Integration'}
                      </div>
                      <LuExternalLink class="w-4 h-4 text-gray-400" />
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Connected Websites */}
          <div class="border-t pt-8">
            <h2 class="text-xl font-semibold text-gray-900 mb-6">Your Connected Websites</h2>
            <ConnectedWebsites websites={connectedWebsites.value} />
          </div>
        </>
      ) : (
        /* Credentials Setup Modal */
        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div class="bg-white rounded-xl shadow-xl max-w-md w-full mx-4">
            <div class="p-6">
              <div class="flex items-center gap-3 mb-6">
                <div class="flex items-center justify-center w-10 h-10 bg-gray-50 rounded-lg border border-gray-200">
                  {renderIcon(builders.find(b => b.id === selectedBuilder.value)?.icon || 'LuGlobe', "w-6 h-6")}
                </div>
                <div>
                  <h3 class="text-lg font-semibold text-gray-900">
                    Connect {builders.find(b => b.id === selectedBuilder.value)?.name}
                  </h3>
                  <p class="text-sm text-gray-600">Enter your credentials to get started</p>
                </div>
              </div>

              {selectedBuilder.value === 'jflux' ? (
                <div class="space-y-4">
                  <p class="text-sm text-gray-600">
                    JFlux is our native website builder. Click continue to start creating your website.
                  </p>
                  <div class="flex gap-3">
                    <button
                      onClick$={handleConnect}
                      class="flex-1 bg-[#9CAF88] text-white px-4 py-2 rounded-lg hover:bg-[#9CAF88]/90 transition-colors"
                    >
                      Start Building
                    </button>
                    <button
                      onClick$={() => showCredentials.value = false}
                      class="px-4 py-2 text-gray-600 hover:text-gray-800 transition-colors"
                    >
                      Cancel
                    </button>
                  </div>
                </div>
              ) : (
                <div class="space-y-4">
                  {selectedBuilder.value === 'wix' && (
                    <>
                      <div>
                        <label class="block text-sm font-medium text-gray-700 mb-2">
                          API Key
                        </label>
                        <input
                          type="text"
                          class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none"
                          placeholder="Enter your Wix API key"
                          value={credentials.value.apiKey || ''}
                          onInput$={(e) => {
                            credentials.value = { ...credentials.value, apiKey: (e.target as HTMLInputElement).value };
                          }}
                        />
                      </div>
                      <div>
                        <label class="block text-sm font-medium text-gray-700 mb-2">
                          Site ID
                        </label>
                        <input
                          type="text"
                          class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none"
                          placeholder="Enter your Wix site ID"
                          value={credentials.value.siteId || ''}
                          onInput$={(e) => {
                            credentials.value = { ...credentials.value, siteId: (e.target as HTMLInputElement).value };
                          }}
                        />
                      </div>
                    </>
                  )}
                  
                  {selectedBuilder.value === 'wordpress' && (
                    <>
                      <div>
                        <label class="block text-sm font-medium text-gray-700 mb-2">
                          Site URL
                        </label>
                        <input
                          type="url"
                          class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none"
                          placeholder="https://yoursite.com"
                          value={credentials.value.siteUrl || ''}
                          onInput$={(e) => {
                            credentials.value = { ...credentials.value, siteUrl: (e.target as HTMLInputElement).value };
                          }}
                        />
                      </div>
                      <div>
                        <label class="block text-sm font-medium text-gray-700 mb-2">
                          Username
                        </label>
                        <input
                          type="text"
                          class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none"
                          placeholder="WordPress username"
                          value={credentials.value.username || ''}
                          onInput$={(e) => {
                            credentials.value = { ...credentials.value, username: (e.target as HTMLInputElement).value };
                          }}
                        />
                      </div>
                      <div>
                        <label class="block text-sm font-medium text-gray-700 mb-2">
                          Application Password
                        </label>
                        <input
                          type="password"
                          class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none"
                          placeholder="WordPress application password"
                          value={credentials.value.password || ''}
                          onInput$={(e) => {
                            credentials.value = { ...credentials.value, password: (e.target as HTMLInputElement).value };
                          }}
                        />
                      </div>
                    </>
                  )}
                  
                  {selectedBuilder.value === 'squarespace' && (
                    <div>
                      <label class="block text-sm font-medium text-gray-700 mb-2">
                        API Key
                      </label>
                      <input
                        type="text"
                        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none"
                        placeholder="Enter your Squarespace API key"
                        value={credentials.value.apiKey || ''}
                        onInput$={(e) => {
                          credentials.value = { ...credentials.value, apiKey: (e.target as HTMLInputElement).value };
                        }}
                      />
                    </div>
                  )}

                  <div class="text-xs text-gray-500 bg-gray-50 p-3 rounded-lg">
                    <LuSettings class="w-4 h-4 inline mr-1" />
                    Your credentials are encrypted and stored securely. We only use them to manage your website on your behalf.
                  </div>

                  <div class="flex gap-3">
                    <button
                      onClick$={handleConnect}
                      disabled={isLoading.value}
                      class="flex-1 bg-[#9CAF88] text-white px-4 py-2 rounded-lg hover:bg-[#9CAF88]/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      {isLoading.value ? 'Connecting...' : 'Connect Website'}
                    </button>
                    <button
                      onClick$={() => showCredentials.value = false}
                      class="px-4 py-2 text-gray-600 hover:text-gray-800 transition-colors"
                    >
                      Cancel
                    </button>
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
});

