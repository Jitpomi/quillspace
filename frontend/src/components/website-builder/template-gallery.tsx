/**
 * Template Gallery Component for QuillSpace
 * Helps authors choose beautiful website templates for their book selling sites
 */

import { component$, useSignal, $, useComputed$ } from '@builder.io/qwik';
import { LuEye, LuCheck, LuStar, LuHeart, LuZap, LuChevronLeft, LuChevronRight } from '@qwikest/icons/lucide';

// Template data - in a real app this would come from an API
const templates = [
  {
    id: 'literary-classic',
    name: 'Literary Classic',
    description: 'Elegant and timeless design perfect for literary fiction authors',
    category: 'Literary',
    preview: '/templates/literary-classic.jpg',
    features: ['Book showcase', 'Author bio', 'Reading list', 'Contact form'],
    bestFor: 'Literary fiction, poetry, classic literature',
    colors: ['Deep navy', 'Cream', 'Gold accents'],
    popular: true
  },
  {
    id: 'modern-minimalist',
    name: 'Modern Minimalist',
    description: 'Clean, contemporary design that lets your words take center stage',
    category: 'Modern',
    preview: '/templates/modern-minimalist.jpg',
    features: ['Minimal layout', 'Focus on content', 'Mobile-first', 'Fast loading'],
    bestFor: 'Non-fiction, business books, self-help',
    colors: ['White', 'Black', 'Sage green'],
    popular: false
  },
  {
    id: 'cozy-bookshop',
    name: 'Cozy Bookshop',
    description: 'Warm and inviting design that feels like a neighborhood bookstore',
    category: 'Cozy',
    preview: '/templates/cozy-bookshop.jpg',
    features: ['Warm colors', 'Book collections', 'Reading nook feel', 'Community focus'],
    bestFor: 'Romance, cozy mystery, children\'s books',
    colors: ['Warm brown', 'Cream', 'Soft orange'],
    popular: true
  },
  {
    id: 'sci-fi-futuristic',
    name: 'Sci-Fi Futuristic',
    description: 'Bold, futuristic design perfect for science fiction and fantasy',
    category: 'Genre',
    preview: '/templates/sci-fi-futuristic.jpg',
    features: ['Dynamic animations', 'Dark theme', 'Sci-fi elements', 'Interactive features'],
    bestFor: 'Science fiction, fantasy, thriller',
    colors: ['Dark blue', 'Electric blue', 'Silver'],
    popular: false
  },
  {
    id: 'author-portfolio',
    name: 'Author Portfolio',
    description: 'Professional showcase for established authors with multiple works',
    category: 'Professional',
    preview: '/templates/author-portfolio.jpg',
    features: ['Portfolio layout', 'Awards section', 'Media kit', 'Speaking engagements'],
    bestFor: 'Established authors, speakers, consultants',
    colors: ['Charcoal', 'White', 'Accent blue'],
    popular: true
  },
  {
    id: 'indie-creative',
    name: 'Indie Creative',
    description: 'Artistic and unique design for independent creative authors',
    category: 'Creative',
    preview: '/templates/indie-creative.jpg',
    features: ['Artistic layout', 'Creative elements', 'Personal touch', 'Storytelling focus'],
    bestFor: 'Indie authors, creative non-fiction, memoirs',
    colors: ['Teal', 'Coral', 'Cream'],
    popular: false
  }
];

export default component$(() => {
  const selectedTemplate = useSignal<string | null>(null);
  const selectedCategory = useSignal<string>('All');
  const showPreview = useSignal<string | null>(null);
  const currentPage = useSignal(1);
  const templatesPerPage = 9; // 3x3 grid

  const categories = ['All', 'Literary', 'Modern', 'Cozy', 'Genre', 'Professional', 'Creative'];

  const filteredTemplates = useComputed$(() => {
    return selectedCategory.value === 'All' 
      ? templates 
      : templates.filter(t => t.category === selectedCategory.value);
  });

  const totalPages = useComputed$(() => {
    return Math.ceil(filteredTemplates.value.length / templatesPerPage);
  });

  const paginatedTemplates = useComputed$(() => {
    const startIndex = (currentPage.value - 1) * templatesPerPage;
    const endIndex = startIndex + templatesPerPage;
    return filteredTemplates.value.slice(startIndex, endIndex);
  });

  const paginationInfo = useComputed$(() => {
    const startIndex = (currentPage.value - 1) * templatesPerPage + 1;
    const endIndex = Math.min(currentPage.value * templatesPerPage, filteredTemplates.value.length);
    return {
      start: startIndex,
      end: endIndex,
      total: filteredTemplates.value.length
    };
  });

  const handleSelectTemplate = $((id: string) => {
    selectedTemplate.value = id;
  });

  const handlePreview = $((templateId: string) => {
    showPreview.value = templateId;
  });

  const handleContinue = $((templateId: string) => {
    // templateId is passed as parameter
    if (templateId) {
      console.log("Template selected:", templateId);
    }
  });

  const handleCategoryChange = $((category: string) => {
    selectedCategory.value = category;
    currentPage.value = 1; // Reset to first page when category changes
  });

  const handlePageChange = $((page: number) => {
    currentPage.value = page;
    // Scroll to top when page changes
    window.scrollTo({ top: 0, behavior: 'smooth' });
  });

  return (
    <div class="max-w-7xl mx-auto space-y-8">
      {/* Header */}
      <div class="text-center py-8">
        <h2 class="text-4xl font-serif font-semibold text-[#2D3748] mb-3 leading-tight">
          Choose Your Perfect Website
        </h2>
        <p class="text-lg font-sans text-gray-600 leading-relaxed max-w-3xl mx-auto">
          Every great author deserves a beautiful home for their words. Select a template that reflects your unique voice and connects with your readers.
        </p>
        <div class="mt-4 text-sm font-sans text-[#9CAF88] italic">
          ✨ Don't worry—you can customize everything to make it truly yours
        </div>
      </div>

      {/* Category Filter */}
      <div class="flex flex-wrap justify-center gap-3 mb-8">
        {categories.map((category) => (
          <button
            key={category}
            onClick$={() => handleCategoryChange(category)}
            class={`px-4 py-2 rounded-full font-sans text-sm transition-gentle ${
              selectedCategory.value === category
                ? 'bg-[#9CAF88] text-white shadow-warm'
                : 'bg-[#F7F3E9] text-[#2D3748] hover:bg-[#9CAF88]/20 hover-lift'
            }`}
          >
            {category}
          </button>
        ))}
      </div>

      {/* Pagination Info */}
      <div class="text-center text-sm font-sans text-gray-600 mb-4">
        Showing {paginationInfo.value.start}-{paginationInfo.value.end} of {paginationInfo.value.total} templates
      </div>

      {/* Template Grid */}
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
        {paginatedTemplates.value.map((template: any) => (
          <div
            key={template.id}
            class={`bg-[#FEFCF7] rounded-xl border-2 transition-gentle hover-lift cursor-pointer ${
              selectedTemplate.value === template.id
                ? 'border-[#9CAF88] shadow-warm-lg'
                : 'border-[#E8E2D4] hover:border-[#9CAF88]/50'
            }`}
            onClick$={() => handleSelectTemplate(template.id)}
          >
            {/* Template Preview */}
            <div class="relative">
              <div class="aspect-video bg-gradient-to-br from-gray-100 to-gray-200 rounded-t-xl flex items-center justify-center">
                <div class="text-gray-400 font-serif text-lg">Template Preview</div>
              </div>
              
              {/* Popular Badge */}
              {template.popular && (
                <div class="absolute top-3 right-3 bg-[#9CAF88] text-white px-2 py-1 rounded-full text-xs font-medium flex items-center gap-1">
                  <LuStar class="w-3 h-3" />
                  Popular
                </div>
              )}

              {/* Preview Button */}
              <button
                onClick$={(e) => {
                  e.stopPropagation();
                  handlePreview(template.id);
                }}
                class="absolute top-3 left-3 bg-white/90 hover:bg-white text-[#2D3748] px-3 py-1 rounded-full text-xs font-medium flex items-center gap-1 transition-soft"
              >
                <LuEye class="w-3 h-3" />
                Preview
              </button>

              {/* Selection Indicator */}
              {selectedTemplate.value === template.id && (
                <div class="absolute inset-0 bg-[#9CAF88]/10 rounded-t-xl flex items-center justify-center">
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
              <p class="font-sans text-gray-600 text-sm mb-4 leading-relaxed">
                {template.description}
              </p>

              {/* Best For */}
              <div class="mb-4">
                <div class="text-xs font-sans font-medium text-[#9CAF88] mb-1">PERFECT FOR:</div>
                <div class="text-sm font-sans text-gray-600">{template.bestFor}</div>
              </div>

              {/* Color Palette */}
              <div class="mb-4">
                <div class="text-xs font-sans font-medium text-[#9CAF88] mb-2">COLORS:</div>
                <div class="flex gap-2">
                  {template.colors.map((color: string, index: number) => (
                    <div
                      key={index}
                      class="w-4 h-4 rounded-full bg-gradient-to-br from-gray-300 to-gray-400"
                      title={color}
                    />
                  ))}
                </div>
              </div>

              {/* Features */}
              <div class="space-y-1">
                {template.features.slice(0, 3).map((feature: string, index: number) => (
                  <div key={index} class="flex items-center gap-2 text-xs font-sans text-gray-600">
                    <LuZap class="w-3 h-3 text-[#9CAF88]" />
                    {feature}
                  </div>
                ))}
                {template.features.length > 3 && (
                  <div class="text-xs font-sans text-gray-500 italic">
                    +{template.features.length - 3} more features
                  </div>
                )}
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Pagination Controls */}
      {totalPages.value > 1 && (
        <div class="flex items-center justify-center gap-4 py-8">
          <button
            onClick$={() => handlePageChange(currentPage.value - 1)}
            disabled={currentPage.value === 1}
            class="flex items-center gap-2 px-4 py-2 rounded-lg font-sans text-sm transition-gentle disabled:opacity-50 disabled:cursor-not-allowed bg-[#F7F3E9] text-[#2D3748] hover:bg-[#9CAF88]/20 hover-lift"
          >
            <LuChevronLeft class="w-4 h-4" />
            Previous
          </button>

          <div class="flex items-center gap-2">
            {Array.from({ length: totalPages.value }, (_, i) => i + 1).map((page) => (
              <button
                key={page}
                onClick$={() => handlePageChange(page)}
                class={`w-10 h-10 rounded-lg font-sans text-sm transition-gentle ${
                  currentPage.value === page
                    ? 'bg-[#9CAF88] text-white shadow-warm'
                    : 'bg-[#F7F3E9] text-[#2D3748] hover:bg-[#9CAF88]/20 hover-lift'
                }`}
              >
                {page}
              </button>
            ))}
          </div>

          <button
            onClick$={() => handlePageChange(currentPage.value + 1)}
            disabled={currentPage.value === totalPages.value}
            class="flex items-center gap-2 px-4 py-2 rounded-lg font-sans text-sm transition-gentle disabled:opacity-50 disabled:cursor-not-allowed bg-[#F7F3E9] text-[#2D3748] hover:bg-[#9CAF88]/20 hover-lift"
          >
            Next
            <LuChevronRight class="w-4 h-4" />
          </button>
        </div>
      )}

      {/* Continue Button */}
      {selectedTemplate.value && (
        <div class="text-center py-8">
          <button
            onClick$={() => handleContinue(selectedTemplate.value!)}
            class="bg-[#9CAF88] hover:bg-[#8a9e7a] text-white px-8 py-4 rounded-lg font-serif text-lg font-medium transition-gentle hover-lift shadow-warm"
          >
            Continue with {templates.find(t => t.id === selectedTemplate.value)?.name}
          </button>
          <div class="mt-3 text-sm font-sans text-gray-600 italic">
            Next: We'll help you customize it with your content
          </div>
        </div>
      )}

      {/* Encouragement */}
      <div class="text-center py-6">
        <div class="bg-[#F7F3E9] rounded-xl border border-[#E8E2D4] p-6 max-w-2xl mx-auto">
          <LuHeart class="w-8 h-8 text-[#9CAF88] mx-auto mb-3" />
          <p class="font-sans text-gray-600 leading-relaxed">
            "Remember, this is just the beginning. Once you choose a template, we'll walk you through 
            personalizing every detail—your colors, your words, your story. You're not just picking a design; 
            you're creating a home for your literary legacy."
          </p>
        </div>
      </div>
    </div>
  );
});
