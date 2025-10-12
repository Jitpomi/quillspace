/**
 * Website Builder - Main orchestrator for the author (website-builder) creation flow
 */

import { component$, useSignal, $, useStore } from '@builder.io/qwik';
import type { Data } from '@measured/puck';
import TemplateGallery from './template-gallery';
import CustomizationWizard from './customization-wizard';
import PuckEditor from './puck-editor';
import PuckRenderer from './puck-renderer';

type BuilderStep = 'templates' | 'customize' | 'editor' | 'preview' | 'publish';

export default component$(() => {
  const currentStep = useSignal<BuilderStep>('templates');
  const selectedTemplate = useSignal<string | null>(null);
  const puckData = useStore<Data>({
    content: [],
    root: { props: { title: "My Author Website" } },
  });

  const handleTemplateSelect = $((templateId: string) => {
    selectedTemplate.value = templateId;
    currentStep.value = 'customize';
  });

  const handleStartEditing = $(() => {
    currentStep.value = 'editor';
  });

  const handleBackToTemplates = $(() => {
    currentStep.value = 'templates';
    selectedTemplate.value = null;
  });

  const handleBackToCustomize = $(() => {
    currentStep.value = 'customize';
  });

  const handlePuckChange = $((data: Data) => {
    Object.assign(puckData, data);
  });

  const handlePreview = $(() => {
    currentStep.value = 'preview';
  });

  const handlePublish = $(async (data: Data) => {
    Object.assign(puckData, data);
    
    // TODO: Send data to backend API for publishing
    try {
      const response = await fetch('/api/sites/publish', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          template_id: selectedTemplate.value,
          puck_data: data,
        }),
      });
      
      if (response.ok) {
        currentStep.value = 'publish';
      } else {
        console.error('Failed to publish (website-builder)');
      }
    } catch (error) {
      console.error('Error publishing (website-builder):', error);
    }
  });

  const handlePublishClick = $(() => {
    handlePublish(puckData);
  });

  return (
    <div class="min-h-screen">
      {currentStep.value === 'templates' && (
        <TemplateGallery onTemplateSelect={handleTemplateSelect} />
      )}
      
      {currentStep.value === 'customize' && (
        <div>
          {/* Back to Templates Button */}
          <div class="max-w-7xl mx-auto px-6 py-4">
            <button
              onClick$={handleBackToTemplates}
              class="flex items-center gap-2 text-gray-600 hover:text-[#2D3748] transition-gentle"
            >
              ← Back to Templates
            </button>
          </div>
          
          <CustomizationWizard onStartEditing={handleStartEditing} />
        </div>
      )}

      {currentStep.value === 'editor' && (
        <div class="h-screen">
          {/* Editor Header */}
          <div class="bg-white border-b px-6 py-3 flex justify-between items-center">
            <button
              onClick$={handleBackToCustomize}
              class="flex items-center gap-2 text-gray-600 hover:text-[#2D3748] transition-gentle"
            >
              ← Back to Setup
            </button>
            <div class="flex gap-3">
              <button
                onClick$={handlePreview}
                class="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors"
              >
                Preview
              </button>
            </div>
          </div>
          
          {/* Puck Editor */}
          <PuckEditor
            data={puckData}
            onChange$={handlePuckChange}
            onPublish$={handlePublish}
          />
        </div>
      )}

      {currentStep.value === 'preview' && (
        <div class="min-h-screen">
          {/* Preview Header */}
          <div class="bg-white border-b px-6 py-3 flex justify-between items-center">
            <button
              onClick$={() => currentStep.value = 'editor'}
              class="flex items-center gap-2 text-gray-600 hover:text-[#2D3748] transition-gentle"
            >
              ← Back to Editor
            </button>
            <div class="flex gap-3">
              <button
                onClick$={handlePublishClick}
                class="px-6 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
              >
                Publish Website
              </button>
            </div>
          </div>
          
          {/* Preview Content */}
          <div class="bg-gray-100 p-6">
            <div class="max-w-6xl mx-auto bg-white rounded-lg shadow-lg overflow-hidden">
              <PuckRenderer data={puckData} />
            </div>
          </div>
        </div>
      )}

      {currentStep.value === 'publish' && (
        <div class="min-h-screen flex items-center justify-center">
          <div class="text-center max-w-md">
            <div class="w-16 h-16 bg-green-100 rounded-full flex items-center justify-center mx-auto mb-4">
              <svg class="w-8 h-8 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path>
              </svg>
            </div>
            <h2 class="text-2xl font-bold text-gray-900 mb-2">Website Published!</h2>
            <p class="text-gray-600 mb-6">Your author website is now live and ready for readers to discover.</p>
            <div class="space-y-3">
              <button
                onClick$={() => currentStep.value = 'editor'}
                class="w-full px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors"
              >
                Continue Editing
              </button>
              <button
                onClick$={handleBackToTemplates}
                class="w-full px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
              >
                Create Another Site
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
});
