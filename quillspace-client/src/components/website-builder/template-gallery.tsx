/**
 * Template Gallery Component for QuillSpace
 * Helps authors choose beautiful (website-builder) templates for their book selling sites
 */

import { component$, useSignal, $, useResource$, Resource } from '@builder.io/qwik';
import { server$ } from '@builder.io/qwik-city';
import { LuEye, LuCheck, LuStar, LuHeart } from '@qwikest/icons/lucide';
import { getAuthToken } from '~/utils/auth';

interface Template {
  id: string;
  name: string;
  description: string;
  category: string;
  preview: string;
  features: string[];
  bestFor: string;
  colors: string[];
  popular: boolean;
}

interface TemplateGalleryProps {
  onTemplateSelect?: (templateId: string) => void;
}

// Server function to fetch templates with authentication
const fetchTemplates = server$(async function() {
  const authToken = getAuthToken(this.cookie);
  
  if (!authToken) {
    console.warn('No auth token available for templates API');
    return [];
  }

  try {
    const response = await fetch('http://localhost:3001/api/templates', {
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${authToken}`
      }
    });

    if (response.ok) {
      const apiResponse = await response.json();
      return apiResponse.data || apiResponse || [];
    } else {
      console.warn('Failed to fetch templates from API, status:', response.status);
      return [];
    }
  } catch (error) {
    console.error('Error fetching templates:', error);
    return [];
  }
});

export default component$<TemplateGalleryProps>(({ onTemplateSelect }) => {
  const selectedTemplate = useSignal<string | null>(null);

  // Fetch templates from backend API using server function
  const templatesResource = useResource$<Template[]>(async () => {
    return await fetchTemplates();
  });

  const handleTemplateSelect = $((templateId: string) => {
    selectedTemplate.value = templateId;
    if (onTemplateSelect) {
      onTemplateSelect(templateId);
    }
  });

  return (
    <div class="min-h-screen py-12">
      <div class="max-w-7xl mx-auto px-6">
        {/* Header */}
        <div class="text-center mb-12">
          <h1 class="text-4xl font-serif font-bold text-[#2D3748] mb-4">
            Choose Your Perfect Website
          </h1>
          <p class="text-lg text-gray-600 max-w-2xl mx-auto">
            Every great author deserves a beautiful home for their words. Select a template that reflects
            your unique voice and connects with your readers.
          </p>
          <p class="text-sm text-[#9CAF88] mt-2 italic">
            ✨ Don't worry—you can customize everything to make it truly yours
          </p>
        </div>

        {/* Templates Grid */}
        <Resource
          value={templatesResource}
          onPending={() => (
            <div class="text-center py-12">
              <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-[#9CAF88]"></div>
              <p class="mt-4 text-gray-600">Loading beautiful templates...</p>
            </div>
          )}
          onRejected={(error) => (
            <div class="text-center py-12">
              <p class="text-red-600 mb-4">Failed to load templates</p>
              <p class="text-gray-600">{String(error)}</p>
            </div>
          )}
          onResolved={(templates) => {
            if (!templates || templates.length === 0) {
              return (
                <div class="text-center py-12">
                  <LuHeart class="w-16 h-16 text-gray-400 mx-auto mb-4" />
                  <h3 class="text-xl font-semibold text-gray-700 mb-2">No templates found</h3>
                  <p class="text-gray-600">
                    We're working on adding beautiful templates for you. Check back soon!
                  </p>
                </div>
              );
            }

            return (
              <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
                {templates.map((template) => (
                  <div
                    key={template.id}
                    class={`bg-white rounded-xl shadow-lg overflow-hidden transition-all duration-300 hover:shadow-xl hover:scale-105 cursor-pointer border-2 ${
                      selectedTemplate.value === template.id
                        ? 'border-[#9CAF88] ring-2 ring-[#9CAF88]/20'
                        : 'border-transparent hover:border-[#9CAF88]/30'
                    }`}
                    onClick$={() => handleTemplateSelect(template.id)}
                  >
                    {/* Template Preview */}
                    <div class="relative h-48 bg-gray-100">
                      <img
                        src={template.preview}
                        alt={template.name}
                        class="w-full h-full object-cover"
                        loading="lazy"
                      />
                      {template.popular && (
                        <div class="absolute top-3 right-3 bg-[#9CAF88] text-white px-2 py-1 rounded-full text-xs font-medium flex items-center gap-1">
                          <LuStar class="w-3 h-3" />
                          Popular
                        </div>
                      )}
                      {selectedTemplate.value === template.id && (
                        <div class="absolute inset-0 bg-[#9CAF88]/20 flex items-center justify-center">
                          <div class="bg-[#9CAF88] text-white p-2 rounded-full">
                            <LuCheck class="w-6 h-6" />
                          </div>
                        </div>
                      )}
                    </div>

                    {/* Template Info */}
                    <div class="p-6">
                      <h3 class="text-xl font-serif font-semibold text-[#2D3748] mb-2">
                        {template.name}
                      </h3>
                      <p class="text-gray-600 text-sm mb-4 leading-relaxed">
                        {template.description}
                      </p>
                      
                      {/* Features */}
                      <div class="mb-4">
                        <p class="text-xs font-medium text-gray-500 uppercase tracking-wide mb-2">
                          Features
                        </p>
                        <div class="flex flex-wrap gap-1">
                          {template.features.slice(0, 3).map((feature) => (
                            <span
                              key={feature}
                              class="bg-gray-100 text-gray-700 px-2 py-1 rounded text-xs"
                            >
                              {feature}
                            </span>
                          ))}
                          {template.features.length > 3 && (
                            <span class="text-gray-500 text-xs px-2 py-1">
                              +{template.features.length - 3} more
                            </span>
                          )}
                        </div>
                      </div>

                      {/* Best For */}
                      <div class="border-t pt-4">
                        <p class="text-xs font-medium text-gray-500 uppercase tracking-wide mb-1">
                          Best for
                        </p>
                        <p class="text-sm text-gray-700">{template.bestFor}</p>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            );
          }}
        />

        {/* Continue Button */}
        {selectedTemplate.value && (
          <div class="text-center mt-12">
            <button
              onClick$={() => {
                if (onTemplateSelect && selectedTemplate.value) {
                  onTemplateSelect(selectedTemplate.value);
                }
              }}
              class="bg-[#9CAF88] hover:bg-[#8ba077] text-white px-8 py-4 rounded-xl font-semibold text-lg transition-all duration-300 shadow-lg hover:shadow-xl transform hover:scale-105"
            >
              Continue with Selected Template →
            </button>
          </div>
        )}
      </div>
    </div>
  );
});
