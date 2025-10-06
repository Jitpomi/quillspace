/**
 * Website Builder - Main orchestrator for the author website creation flow
 */

import { component$, useSignal, $ } from '@builder.io/qwik';
import TemplateGallery from './template-gallery';
import CustomizationWizard from './customization-wizard';

type BuilderStep = 'templates' | 'customize' | 'preview' | 'publish';

export default component$(() => {
  const currentStep = useSignal<BuilderStep>('templates');
  const selectedTemplate = useSignal<string | null>(null);


  const handleBackToTemplates = $(() => {
    currentStep.value = 'templates';
    selectedTemplate.value = null;
  });

  return (
    <div class="min-h-screen bg-[#FEFCF7]">
      {currentStep.value === 'templates' && (
        <TemplateGallery />
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
          
          <CustomizationWizard />
        </div>
      )}
    </div>
  );
});
