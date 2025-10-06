/**
 * Customization Wizard - Guides authors through personalizing their website
 */

import { component$, useSignal, $ } from '@builder.io/qwik';
import { LuArrowLeft, LuArrowRight, LuCheck, LuPalette, LuImage, LuBookOpen } from '@qwikest/icons/lucide';

interface CustomizationStep {
  id: string;
  title: string;
  description: string;
  icon: any;
}

const steps: CustomizationStep[] = [
  {
    id: 'content',
    title: 'Your Story',
    description: 'Tell us about yourself and your books',
    icon: LuBookOpen
  },
  {
    id: 'branding',
    title: 'Your Brand',
    description: 'Choose colors and fonts that reflect your style',
    icon: LuPalette
  },
  {
    id: 'images',
    title: 'Your Images',
    description: 'Add your author photo and book covers',
    icon: LuImage
  },
  {
    id: 'preview',
    title: 'Preview',
    description: 'See your beautiful website come to life',
    icon: LuCheck
  }
];

// Simplified component without props for now


export default component$(() => {
  
  const currentStep = useSignal(0);
  const authorData = useSignal({
    name: '',
    bio: '',
    genre: '',
    books: [],
    colors: { primary: '#2D3748', accent: '#9CAF88' },
    font: 'Crimson Text'
  });

  const nextStep = $(() => {
    if (currentStep.value < steps.length - 1) {
      currentStep.value++;
    }
  });

  const prevStep = $(() => {
    if (currentStep.value > 0) {
      currentStep.value--;
    }
  });

  const currentStepData = steps[currentStep.value];

  return (
    <div class="max-w-4xl mx-auto space-y-8">
      {/* Progress Header */}
      <div class="text-center py-6">
        <h2 class="text-3xl font-serif font-semibold text-[#2D3748] mb-3">
          Let's Make It Yours
        </h2>
        <p class="text-lg font-sans text-gray-600 mb-6">
          We'll guide you through each step. Take your time—this is your creative space.
        </p>
        
        {/* Progress Bar */}
        <div class="flex items-center justify-center gap-4 mb-8">
          {steps.map((step, index) => (
            <div key={step.id} class="flex items-center">
              <div class={`w-10 h-10 rounded-full flex items-center justify-center transition-gentle ${
                index <= currentStep.value 
                  ? 'bg-[#9CAF88] text-white' 
                  : 'bg-[#F7F3E9] text-gray-400'
              }`}>
                {index < currentStep.value ? (
                  <LuCheck class="w-5 h-5" />
                ) : (
                  <step.icon class="w-5 h-5" />
                )}
              </div>
              {index < steps.length - 1 && (
                <div class={`w-12 h-0.5 mx-2 transition-gentle ${
                  index < currentStep.value ? 'bg-[#9CAF88]' : 'bg-gray-200'
                }`} />
              )}
            </div>
          ))}
        </div>
      </div>

      {/* Step Content */}
      <div class="bg-[#FEFCF7] rounded-xl border border-[#E8E2D4] p-8 shadow-warm">
        <div class="text-center mb-8">
          <currentStepData.icon class="w-12 h-12 text-[#9CAF88] mx-auto mb-4" />
          <h3 class="text-2xl font-serif font-semibold text-[#2D3748] mb-2">
            {currentStepData.title}
          </h3>
          <p class="font-sans text-gray-600">
            {currentStepData.description}
          </p>
        </div>

        {/* Step 1: Content */}
        {currentStep.value === 0 && (
          <div class="space-y-6">
            <div>
              <label class="block text-sm font-serif font-medium text-[#2D3748] mb-2">
                What should readers call you?
              </label>
              <input
                type="text"
                placeholder="Your author name"
                class="w-full px-4 py-3 border border-[#E8E2D4] rounded-lg focus-soft bg-[#FEFCF7] font-sans"
                value={authorData.value.name}
                onInput$={(e) => {
                  authorData.value = { ...authorData.value, name: (e.target as HTMLInputElement).value };
                }}
              />
            </div>
            
            <div>
              <label class="block text-sm font-serif font-medium text-[#2D3748] mb-2">
                Tell readers about yourself
              </label>
              <textarea
                placeholder="Share your story, your inspiration, what drives your writing..."
                rows={4}
                class="w-full px-4 py-3 border border-[#E8E2D4] rounded-lg focus-soft bg-[#FEFCF7] font-sans resize-none"
                value={authorData.value.bio}
                onInput$={(e) => {
                  authorData.value = { ...authorData.value, bio: (e.target as HTMLTextAreaElement).value };
                }}
              />
            </div>

            <div>
              <label class="block text-sm font-serif font-medium text-[#2D3748] mb-2">
                What genre captures your heart?
              </label>
              <select
                class="w-full px-4 py-3 border border-[#E8E2D4] rounded-lg focus-soft bg-[#FEFCF7] font-sans"
                value={authorData.value.genre}
                onChange$={(e) => {
                  authorData.value = { ...authorData.value, genre: (e.target as HTMLSelectElement).value };
                }}
              >
                <option value="">Choose your primary genre</option>
                <option value="literary-fiction">Literary Fiction</option>
                <option value="romance">Romance</option>
                <option value="mystery">Mystery & Thriller</option>
                <option value="sci-fi-fantasy">Science Fiction & Fantasy</option>
                <option value="non-fiction">Non-Fiction</option>
                <option value="young-adult">Young Adult</option>
                <option value="children">Children's Books</option>
                <option value="poetry">Poetry</option>
              </select>
            </div>
          </div>
        )}

        {/* Step 2: Branding */}
        {currentStep.value === 1 && (
          <div class="space-y-8">
            <div>
              <h4 class="text-lg font-serif font-semibold text-[#2D3748] mb-4">
                Choose Your Color Palette
              </h4>
              <div class="grid grid-cols-2 md:grid-cols-3 gap-4">
                {[
                  { name: 'Literary Classic', primary: '#2D3748', accent: '#E3B23C' },
                  { name: 'Sage & Cream', primary: '#2D3748', accent: '#9CAF88' },
                  { name: 'Ocean Blue', primary: '#1E40AF', accent: '#7C9CBF' },
                  { name: 'Warm Earth', primary: '#92400E', accent: '#F59E0B' },
                  { name: 'Deep Forest', primary: '#134E4A', accent: '#10B981' },
                  { name: 'Royal Purple', primary: '#581C87', accent: '#B8A9C9' }
                ].map((palette) => (
                  <button
                    key={palette.name}
                    class="p-4 rounded-lg border-2 transition-gentle hover-lift text-left"
                    style={`border-color: ${authorData.value.colors.primary === palette.primary ? palette.accent : '#E8E2D4'}`}
                    onClick$={() => {
                      authorData.value = { 
                        ...authorData.value, 
                        colors: { primary: palette.primary, accent: palette.accent }
                      };
                    }}
                  >
                    <div class="flex gap-2 mb-2">
                      <div class="w-6 h-6 rounded-full" style={`background-color: ${palette.primary}`} />
                      <div class="w-6 h-6 rounded-full" style={`background-color: ${palette.accent}`} />
                    </div>
                    <div class="text-sm font-sans font-medium text-[#2D3748]">{palette.name}</div>
                  </button>
                ))}
              </div>
            </div>

            <div>
              <h4 class="text-lg font-serif font-semibold text-[#2D3748] mb-4">
                Select Your Typography
              </h4>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                {[
                  { name: 'Literary Elegance', font: 'Crimson Text', preview: 'The art of writing is the art of discovering what you believe.' },
                  { name: 'Modern Classic', font: 'Playfair Display', preview: 'Every story has a beginning, middle, and end.' },
                  { name: 'Clean Contemporary', font: 'Source Serif Pro', preview: 'Simple words, powerful stories.' },
                  { name: 'Warm & Inviting', font: 'Lora', preview: 'Words that touch the heart and soul.' }
                ].map((typeface) => (
                  <button
                    key={typeface.name}
                    class={`p-4 rounded-lg border-2 transition-gentle hover-lift text-left ${
                      authorData.value.font === typeface.font 
                        ? 'border-[#9CAF88] bg-[#9CAF88]/5' 
                        : 'border-[#E8E2D4]'
                    }`}
                    onClick$={() => {
                      authorData.value = { ...authorData.value, font: typeface.font };
                    }}
                  >
                    <div class="text-sm font-sans font-medium text-[#2D3748] mb-2">{typeface.name}</div>
                    <div class="text-gray-600 italic" style={`font-family: ${typeface.font}, serif`}>
                      {typeface.preview}
                    </div>
                  </button>
                ))}
              </div>
            </div>
          </div>
        )}

        {/* Navigation */}
        <div class="flex justify-between items-center mt-8 pt-6 border-t border-[#E8E2D4]">
          <button
            onClick$={prevStep}
            disabled={currentStep.value === 0}
            class="flex items-center gap-2 px-6 py-3 text-gray-600 hover:text-[#2D3748] disabled:opacity-50 disabled:cursor-not-allowed transition-gentle"
          >
            <LuArrowLeft class="w-4 h-4" />
            Previous
          </button>

          <div class="text-sm font-sans text-gray-500">
            Step {currentStep.value + 1} of {steps.length}
          </div>

          <button
            onClick$={nextStep}
            disabled={currentStep.value === steps.length - 1}
            class="flex items-center gap-2 px-6 py-3 bg-[#9CAF88] hover:bg-[#8a9e7a] text-white rounded-lg font-medium transition-gentle hover-lift disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Continue
            <LuArrowRight class="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Encouragement */}
      <div class="text-center py-6">
        <div class="text-sm font-sans text-[#9CAF88] italic">
          ✨ "You're creating something beautiful. Every choice you make brings your vision to life."
        </div>
      </div>
    </div>
  );
});
