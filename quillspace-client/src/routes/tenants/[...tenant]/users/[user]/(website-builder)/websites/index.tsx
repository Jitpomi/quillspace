import { component$, useSignal, $, noSerialize, useVisibleTask$ } from '@builder.io/qwik';
import { LuGlobe, LuPlus, LuSettings, LuExternalLink, LuRocket, LuPalette, LuFileText, LuSquare, LuUser, LuShield, LuArrowLeft, LuCalendar } from '@qwikest/icons/lucide';
import { ConnectedWebsites } from '~/components/website-builder/connected-websites';
import { WebsiteBuilderService } from '~/services/website-builder.service';
import type { WebsiteBuilder, ConnectedWebsite, BuilderType } from '~/types/website-builders';

export default component$(() => {
  const serviceType = useSignal<'diy' | 'managed' | ''>('');
  const selectedBuilder = useSignal<string>('');
  const showCredentials = useSignal(false);
  const connectedWebsites = useSignal<ConnectedWebsite[]>([]);
  const isLoading = useSignal(false);
  const credentials = useSignal<Record<string, string>>({});
  
  const builderService = noSerialize(WebsiteBuilderService.getInstance());

  // Initialize Calendly widget
  useVisibleTask$(() => {
    // Load Calendly CSS
    const link = document.createElement('link');
    link.href = 'https://assets.calendly.com/assets/external/widget.css';
    link.rel = 'stylesheet';
    document.head.appendChild(link);

    // Load Calendly JS
    const script = document.createElement('script');
    script.src = 'https://assets.calendly.com/assets/external/widget.js';
    script.async = true;
    script.onload = () => {
      // Initialize badge widget after script loads
      if (typeof window !== 'undefined' && (window as any).Calendly) {
        (window as any).Calendly.initBadgeWidget({
          url: 'https://calendly.com/dev-jitpomi/30min?hide_event_type_details=1&hide_gdpr_banner=1&primary_color=9caf88',
          text: 'Get Your Author Website ✨',
          color: '#9caf88',
          textColor: '#ffffff',
          branding: false
        });
      }
    };
    document.head.appendChild(script);
  });

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

  const handleServiceTypeSelect = $((type: 'diy' | 'managed') => {
    serviceType.value = type;
  });

  const handleBuilderSelect = $((builderId: string) => {
    selectedBuilder.value = builderId;
    showCredentials.value = true;
  });

  const handleBackToServiceSelection = $(() => {
    serviceType.value = '';
    selectedBuilder.value = '';
    showCredentials.value = false;
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
          {/* Service Type Selection */}
          {!serviceType.value ? (
            <div class="mb-8">
              <h2 class="text-xl font-semibold text-gray-900 mb-6">How would you like to manage your website?</h2>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-6 max-w-4xl">
                {/* DIY Option */}
                <div
                  class="group relative overflow-hidden rounded-xl border border-gray-200 hover:border-gray-300 transition-all duration-200 cursor-pointer hover:shadow-lg"
                  onClick$={() => handleServiceTypeSelect('diy')}
                >
                  <div class="absolute inset-0 bg-gradient-to-br from-blue-50 to-blue-100 opacity-5 group-hover:opacity-10 transition-opacity"></div>
                  
                  <div class="relative p-8">
                    <div class="flex items-start gap-4 mb-4">
                      <div class="flex items-center justify-center w-12 h-12 bg-white rounded-lg shadow-sm border border-gray-200">
                        <LuUser class="w-6 h-6 text-[#9CAF88]" />
                      </div>
                      <div class="flex-1">
                        <h3 class="text-lg font-semibold text-gray-900 mb-2">I have my own accounts</h3>
                        <p class="text-gray-600 text-sm leading-relaxed">
                          Connect your existing Wix, WordPress, or Squarespace accounts. You maintain full control and ownership of your websites.
                        </p>
                      </div>
                    </div>
                    <div class="text-xs text-gray-500 bg-gray-50 p-3 rounded-lg">
                      <strong>Best for:</strong> Users with existing websites or technical experience
                    </div>
                  </div>
                </div>

                {/* Managed Option */}
                <div
                  class="group relative overflow-hidden rounded-xl border border-gray-200 hover:border-gray-300 transition-all duration-200 cursor-pointer hover:shadow-lg"
                  onClick$={() => handleServiceTypeSelect('managed')}
                >
                  <div class="absolute inset-0 bg-gradient-to-br from-[#9CAF88] to-[#7A9B6E] opacity-5 group-hover:opacity-10 transition-opacity"></div>
                  
                  <div class="relative p-8">
                    <div class="flex items-start gap-4 mb-4">
                      <div class="flex items-center justify-center w-12 h-12 bg-white rounded-lg shadow-sm border border-gray-200">
                        <LuShield class="w-6 h-6 text-[#9CAF88]" />
                      </div>
                      <div class="flex-1">
                        <h3 class="text-lg font-semibold text-gray-900 mb-2">Build and manage for me</h3>
                        <p class="text-gray-600 text-sm leading-relaxed">
                          QuillSpace creates and manages your website using our professional accounts. Focus on writing while we handle the technical details.
                        </p>
                      </div>
                    </div>
                    <div class="text-xs text-gray-500 bg-gray-50 p-3 rounded-lg">
                      <strong>Best for:</strong> Authors who want a hassle-free, fully managed solution
                    </div>
                  </div>
                </div>
              </div>
            </div>
          ) : (
            /* Website Builder Selection */
            <div class="mb-8">
              <div class="flex items-center gap-3 mb-6">
                <button
                  onClick$={handleBackToServiceSelection}
                  class="flex items-center justify-center w-8 h-8 rounded-lg border border-gray-200 hover:border-gray-300 hover:bg-gray-50 transition-colors"
                  title="Back to service selection"
                >
                  <LuArrowLeft class="w-4 h-4 text-gray-600" />
                </button>
                <h2 class="text-xl font-semibold text-gray-900">
                  {serviceType.value === 'diy' ? 'Connect Your Website Builder' : 'Professional Website Design Service'}
                </h2>
              </div>
              {serviceType.value === 'diy' ? (
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
              ) : (
                /* Managed Service Content */
                <div class="max-w-3xl">
                  <div class="bg-gradient-to-br from-[#9CAF88]/10 to-[#7A9B6E]/10 rounded-xl p-8 border border-[#9CAF88]/20">
                    <div class="flex items-start gap-4 mb-6">
                      <div class="flex items-center justify-center w-16 h-16 bg-white rounded-xl shadow-sm border border-gray-200">
                        <LuShield class="w-8 h-8 text-[#9CAF88]" />
                      </div>
                      <div>
                        <h3 class="text-2xl font-semibold text-gray-900 mb-2">We'll Build Your Perfect Author Website</h3>
                        <p class="text-gray-600 leading-relaxed">
                          Our team of designers and developers will create a stunning, professional website tailored specifically for authors. 
                          Focus on your writing while we handle all the technical details.
                        </p>
                      </div>
                    </div>

                    <div class="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8">
                      <div class="bg-white/50 rounded-lg p-4">
                        <h4 class="font-semibold text-gray-900 mb-2">✨ What's Included</h4>
                        <ul class="text-sm text-gray-600 space-y-1">
                          <li>• Custom design tailored to your genre</li>
                          <li>• Professional author bio and book showcase</li>
                          <li>• Mobile-responsive design</li>
                          <li>• SEO optimization for discoverability</li>
                          <li>• Contact forms and newsletter signup</li>
                          <li>• Social media integration</li>
                        </ul>
                      </div>
                      <div class="bg-white/50 rounded-lg p-4">
                        <h4 class="font-semibold text-gray-900 mb-2">🚀 Coming Soon: AI Website Generator</h4>
                        <p class="text-sm text-gray-600">
                          We're developing an AI agent that will automatically generate a fully functional, 
                          personalized author website based on your preferences and writing style.
                        </p>
                      </div>
                    </div>

                    <div class="bg-white rounded-lg p-6 border border-gray-200">
                      <div class="text-center">
                        <h4 class="text-lg font-semibold text-gray-900 mb-2">Ready to Get Started?</h4>
                        <p class="text-gray-600 mb-4">
                          Contact our team to discuss your website needs and get a custom quote.
                        </p>
                        <div class="flex justify-center">
                          <button 
                            onClick$={() => {
                              // Trigger Calendly widget
                              if (typeof window !== 'undefined' && (window as any).Calendly) {
                                (window as any).Calendly.initPopupWidget({
                                  url: 'https://calendly.com/dev-jitpomi/30min?hide_event_type_details=1&hide_gdpr_banner=1&primary_color=9caf88'
                                });
                              }
                            }}
                            class="bg-[#9CAF88] text-white px-8 py-3 rounded-lg hover:bg-[#9CAF88]/90 transition-colors font-medium flex items-center gap-2"
                          >
                            <LuCalendar class="w-5 h-5" />
                            Schedule Consultation
                          </button>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              )}
            </div>
          )}

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

