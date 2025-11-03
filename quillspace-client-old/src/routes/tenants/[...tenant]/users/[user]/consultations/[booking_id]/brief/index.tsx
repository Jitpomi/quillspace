import { component$, useSignal, useTask$, $ } from '@builder.io/qwik';
import { useLocation } from '@builder.io/qwik-city';
import { LuFileText, LuSave, LuArrowLeft, LuCheckCircle, LuClock } from '@qwikest/icons/lucide';

interface ProjectBriefForm {
  project_name?: string;
  project_type?: string;
  genre?: string;
  target_audience?: string;
  pages_needed?: string[];
  features_required?: string[];
  design_preferences?: any;
  content_status?: string;
  timeline?: string;
  budget_range?: string;
  existing_website?: string;
  special_requirements?: string;
  questions_for_team?: string;
}

interface BookingDetails {
  id: string;
  event_name: string;
  scheduled_at: string;
  guest_email: string;
  status: string;
}

export default component$(() => {
  const location = useLocation();
  const bookingId = location.params.booking_id;
  
  const booking = useSignal<BookingDetails | null>(null);
  const formData = useSignal<ProjectBriefForm>({});
  const isLoading = useSignal(true);
  const isSaving = useSignal(false);
  const error = useSignal<string | null>(null);
  const saved = useSignal(false);

  // Load booking details and existing brief
  useTask$(async () => {
    try {
      const response = await fetch(`/api/consultations/${bookingId}`, {
        credentials: 'include',
      });
      
      if (response.ok) {
        const data = await response.json();
        booking.value = data.booking;
        
        // Load existing project brief if available
        if (data.booking.project_brief) {
          formData.value = data.booking.project_brief;
        }
      } else {
        error.value = 'Failed to load consultation details';
      }
    } catch {
      error.value = 'Network error loading consultation';
    } finally {
      isLoading.value = false;
    }
  });

  const handleSubmit = $(async (e: Event) => {
    e.preventDefault();
    isSaving.value = true;
    error.value = null;

    try {
      const response = await fetch(`/api/consultations/${bookingId}/brief`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
        },
        credentials: 'include',
        body: JSON.stringify(formData.value),
      });

      if (response.ok) {
        saved.value = true;
        setTimeout(() => {
          saved.value = false;
        }, 3000);
      } else {
        error.value = 'Failed to save project brief';
      }
    } catch {
      error.value = 'Network error saving brief';
    } finally {
      isSaving.value = false;
    }
  });

  const updateField = $((field: keyof ProjectBriefForm, value: any) => {
    formData.value = { ...formData.value, [field]: value };
  });

  const formatDate = $((dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleDateString('en-US', {
      weekday: 'long',
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  });

  if (isLoading.value) {
    return (
      <div class="max-w-4xl mx-auto p-6">
        <div class="animate-pulse">
          <div class="h-8 bg-gray-200 rounded w-1/2 mb-6"></div>
          <div class="space-y-4">
            <div class="h-4 bg-gray-200 rounded w-3/4"></div>
            <div class="h-4 bg-gray-200 rounded w-1/2"></div>
            <div class="h-32 bg-gray-200 rounded"></div>
          </div>
        </div>
      </div>
    );
  }

  if (error.value && !booking.value) {
    return (
      <div class="max-w-4xl mx-auto p-6">
        <div class="bg-red-50 border border-red-200 rounded-lg p-4">
          <p class="text-red-800">{error.value}</p>
        </div>
      </div>
    );
  }

  return (
    <div class="max-w-4xl mx-auto p-6">
      {/* Header */}
      <div class="mb-8">
        <div class="flex items-center gap-3 mb-4">
          <a 
            href="/consultations" 
            class="flex items-center justify-center w-8 h-8 rounded-lg border border-gray-200 hover:border-gray-300 hover:bg-gray-50 transition-colors"
          >
            <LuArrowLeft class="w-4 h-4 text-gray-600" />
          </a>
          <LuFileText class="w-8 h-8 text-[#9CAF88]" />
          <div>
            <h1 class="text-3xl font-bold text-gray-900">Project Brief</h1>
            <p class="text-gray-600">{booking.value?.event_name}</p>
          </div>
        </div>
        
        {booking.value && (
          <div class="bg-[#9CAF88]/10 border border-[#9CAF88]/20 rounded-lg p-4">
            <div class="flex items-start gap-3">
              <LuClock class="w-5 h-5 text-[#9CAF88] mt-0.5" />
              <div>
                <p class="font-medium text-gray-900">Consultation Scheduled</p>
                <p class="text-sm text-gray-600">{formatDate(booking.value.scheduled_at)}</p>
                <p class="text-xs text-gray-500 mt-1">
                  Please complete this brief before our consultation to help us prepare
                </p>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Success Message */}
      {saved.value && (
        <div class="mb-6 bg-green-50 border border-green-200 rounded-lg p-4">
          <div class="flex items-center gap-2">
            <LuCheckCircle class="w-5 h-5 text-green-600" />
            <p class="text-green-800 font-medium">Project brief saved successfully!</p>
          </div>
        </div>
      )}

      {/* Error Message */}
      {error.value && (
        <div class="mb-6 bg-red-50 border border-red-200 rounded-lg p-4">
          <p class="text-red-800">{error.value}</p>
        </div>
      )}

      {/* Form */}
      <form onSubmit$={handleSubmit} class="space-y-8">
        {/* Basic Project Information */}
        <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
          <h2 class="text-xl font-semibold text-gray-900 mb-6">Basic Project Information</h2>
          
          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">
                Project Name
              </label>
              <input
                type="text"
                value={formData.value.project_name || ''}
                onInput$={(e) => updateField('project_name', (e.target as HTMLInputElement).value)}
                placeholder="e.g., My Author Website"
                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:border-[#9CAF88]"
              />
            </div>
            
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">
                Project Type
              </label>
              <select
                value={formData.value.project_type || ''}
                onChange$={(e) => updateField('project_type', (e.target as HTMLSelectElement).value)}
                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:border-[#9CAF88]"
              >
                <option value="">Select project type</option>
                <option value="new_website">New Website</option>
                <option value="redesign">Website Redesign</option>
                <option value="maintenance">Website Maintenance</option>
              </select>
            </div>
            
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">
                Your Genre/Niche
              </label>
              <input
                type="text"
                value={formData.value.genre || ''}
                onInput$={(e) => updateField('genre', (e.target as HTMLInputElement).value)}
                placeholder="e.g., Fantasy, Romance, Non-fiction"
                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:border-[#9CAF88]"
              />
            </div>
            
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">
                Target Audience
              </label>
              <input
                type="text"
                value={formData.value.target_audience || ''}
                onInput$={(e) => updateField('target_audience', (e.target as HTMLInputElement).value)}
                placeholder="e.g., Young adults, Business professionals"
                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:border-[#9CAF88]"
              />
            </div>
          </div>
        </div>

        {/* Website Requirements */}
        <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
          <h2 class="text-xl font-semibold text-gray-900 mb-6">Website Requirements</h2>
          
          <div class="space-y-6">
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-3">
                Pages Needed (select all that apply)
              </label>
              <div class="grid grid-cols-2 md:grid-cols-3 gap-3">
                {['Home', 'About', 'Books', 'Blog', 'Contact', 'Events', 'Press Kit', 'Newsletter Signup'].map((page) => (
                  <label key={page} class="flex items-center gap-2 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={formData.value.pages_needed?.includes(page) || false}
                      onChange$={(e) => {
                        const checked = (e.target as HTMLInputElement).checked;
                        const current = formData.value.pages_needed || [];
                        if (checked) {
                          updateField('pages_needed', [...current, page]);
                        } else {
                          updateField('pages_needed', current.filter(p => p !== page));
                        }
                      }}
                      class="rounded border-gray-300 text-[#9CAF88] focus:ring-[#9CAF88]"
                    />
                    <span class="text-sm text-gray-700">{page}</span>
                  </label>
                ))}
              </div>
            </div>
            
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-3">
                Features Required (select all that apply)
              </label>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                {[
                  'Email Newsletter Integration',
                  'Social Media Integration',
                  'Book Sales Integration',
                  'Event Calendar',
                  'Blog/News Section',
                  'Photo Gallery',
                  'Contact Forms',
                  'SEO Optimization',
                  'Mobile Responsive Design',
                  'Analytics Integration'
                ].map((feature) => (
                  <label key={feature} class="flex items-center gap-2 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={formData.value.features_required?.includes(feature) || false}
                      onChange$={(e) => {
                        const checked = (e.target as HTMLInputElement).checked;
                        const current = formData.value.features_required || [];
                        if (checked) {
                          updateField('features_required', [...current, feature]);
                        } else {
                          updateField('features_required', current.filter(f => f !== feature));
                        }
                      }}
                      class="rounded border-gray-300 text-[#9CAF88] focus:ring-[#9CAF88]"
                    />
                    <span class="text-sm text-gray-700">{feature}</span>
                  </label>
                ))}
              </div>
            </div>
          </div>
        </div>

        {/* Project Details */}
        <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
          <h2 class="text-xl font-semibold text-gray-900 mb-6">Project Details</h2>
          
          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">
                Content Status
              </label>
              <select
                value={formData.value.content_status || ''}
                onChange$={(e) => updateField('content_status', (e.target as HTMLSelectElement).value)}
                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:border-[#9CAF88]"
              >
                <option value="">Select content status</option>
                <option value="ready">Content is ready</option>
                <option value="partial">Some content ready</option>
                <option value="needs_creation">Need help creating content</option>
              </select>
            </div>
            
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">
                Timeline
              </label>
              <select
                value={formData.value.timeline || ''}
                onChange$={(e) => updateField('timeline', (e.target as HTMLSelectElement).value)}
                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:border-[#9CAF88]"
              >
                <option value="">Select timeline</option>
                <option value="asap">ASAP</option>
                <option value="1_month">Within 1 month</option>
                <option value="3_months">Within 3 months</option>
                <option value="flexible">Flexible</option>
              </select>
            </div>
            
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">
                Budget Range
              </label>
              <select
                value={formData.value.budget_range || ''}
                onChange$={(e) => updateField('budget_range', (e.target as HTMLSelectElement).value)}
                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:border-[#9CAF88]"
              >
                <option value="">Select budget range</option>
                <option value="under_5k">Under $5,000</option>
                <option value="5k_10k">$5,000 - $10,000</option>
                <option value="10k_plus">$10,000+</option>
                <option value="discuss">Let's discuss</option>
              </select>
            </div>
            
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">
                Existing Website (if any)
              </label>
              <input
                type="url"
                value={formData.value.existing_website || ''}
                onInput$={(e) => updateField('existing_website', (e.target as HTMLInputElement).value)}
                placeholder="https://yourwebsite.com"
                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:border-[#9CAF88]"
              />
            </div>
          </div>
          
          <div class="mt-6 space-y-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">
                Special Requirements
              </label>
              <textarea
                value={formData.value.special_requirements || ''}
                onInput$={(e) => updateField('special_requirements', (e.target as HTMLTextAreaElement).value)}
                placeholder="Any specific requirements, accessibility needs, or technical constraints..."
                rows={3}
                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:border-[#9CAF88]"
              />
            </div>
            
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">
                Questions for Our Team
              </label>
              <textarea
                value={formData.value.questions_for_team || ''}
                onInput$={(e) => updateField('questions_for_team', (e.target as HTMLTextAreaElement).value)}
                placeholder="Any questions you'd like to discuss during our consultation..."
                rows={3}
                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:border-[#9CAF88]"
              />
            </div>
          </div>
        </div>

        {/* Submit Button */}
        <div class="flex justify-end">
          <button
            type="submit"
            disabled={isSaving.value}
            class="bg-[#9CAF88] text-white px-8 py-3 rounded-lg hover:bg-[#9CAF88]/90 transition-colors font-medium flex items-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isSaving.value ? (
              <>
                <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                Saving...
              </>
            ) : (
              <>
                <LuSave class="w-4 h-4" />
                Save Project Brief
              </>
            )}
          </button>
        </div>
      </form>
    </div>
  );
});
