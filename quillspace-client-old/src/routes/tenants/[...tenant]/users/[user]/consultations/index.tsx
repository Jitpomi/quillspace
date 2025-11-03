import { component$, useSignal, useTask$, $ } from '@builder.io/qwik';
import { LuCalendar, LuClock, LuFileText, LuArrowRight, LuUser, LuMail } from '@qwikest/icons/lucide';

interface ConsultationBooking {
  id: string;
  event_name: string;
  scheduled_at: string;
  status: 'scheduled' | 'completed' | 'cancelled' | 'no_show' | 'rescheduled';
  guest_email: string;
  project_brief?: any;
  consultation_notes?: string;
  proposal_sent: boolean;
}

interface PendingBrief {
  booking_id: string;
  event_name: string;
  scheduled_at: string;
  brief_url: string;
}

interface PreparationMaterial {
  title: string;
  material_type: string;
  content?: string;
  file_url?: string;
}

interface ConsultationDashboard {
  upcoming_bookings: ConsultationBooking[];
  past_bookings: ConsultationBooking[];
  pending_briefs: PendingBrief[];
  preparation_materials: PreparationMaterial[];
}

export default component$(() => {
  const dashboard = useSignal<ConsultationDashboard | null>(null);
  const isLoading = useSignal(true);
  const error = useSignal<string | null>(null);

  // Load dashboard data
  useTask$(async () => {
    try {
      const response = await fetch('/api/consultations/dashboard', {
        credentials: 'include',
      });
      
      if (response.ok) {
        dashboard.value = await response.json();
      } else {
        error.value = 'Failed to load consultation dashboard';
      }
    } catch {
      error.value = 'Network error loading dashboard';
    } finally {
      isLoading.value = false;
    }
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

  const getStatusColor = $((status: string) => {
    switch (status) {
      case 'scheduled': return 'text-blue-600 bg-blue-50';
      case 'completed': return 'text-green-600 bg-green-50';
      case 'cancelled': return 'text-red-600 bg-red-50';
      case 'no_show': return 'text-orange-600 bg-orange-50';
      case 'rescheduled': return 'text-yellow-600 bg-yellow-50';
      default: return 'text-gray-600 bg-gray-50';
    }
  });

  if (isLoading.value) {
    return (
      <div class="max-w-6xl mx-auto p-6">
        <div class="animate-pulse">
          <div class="h-8 bg-gray-200 rounded w-1/3 mb-6"></div>
          <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
            <div class="h-64 bg-gray-200 rounded-xl"></div>
            <div class="h-64 bg-gray-200 rounded-xl"></div>
            <div class="h-64 bg-gray-200 rounded-xl"></div>
          </div>
        </div>
      </div>
    );
  }

  if (error.value) {
    return (
      <div class="max-w-6xl mx-auto p-6">
        <div class="bg-red-50 border border-red-200 rounded-lg p-4">
          <p class="text-red-800">{error.value}</p>
        </div>
      </div>
    );
  }

  return (
    <div class="max-w-6xl mx-auto p-6">
      {/* Header */}
      <div class="mb-8">
        <div class="flex items-center gap-3 mb-4">
          <LuCalendar class="w-8 h-8 text-[#9CAF88]" />
          <h1 class="text-3xl font-bold text-gray-900">Consultation Dashboard</h1>
        </div>
        <p class="text-gray-600 text-lg">
          Manage your website design consultations and project progress
        </p>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Upcoming Consultations */}
        <div class="lg:col-span-2">
          <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
            <div class="flex items-center justify-between mb-6">
              <h2 class="text-xl font-semibold text-gray-900">Upcoming Consultations</h2>
              <span class="bg-[#9CAF88]/10 text-[#9CAF88] px-3 py-1 rounded-full text-sm font-medium">
                {dashboard.value?.upcoming_bookings.length || 0} scheduled
              </span>
            </div>

            {dashboard.value?.upcoming_bookings.length === 0 ? (
              <div class="text-center py-8">
                <LuCalendar class="w-12 h-12 text-gray-300 mx-auto mb-4" />
                <p class="text-gray-500">No upcoming consultations</p>
                <p class="text-sm text-gray-400 mt-1">
                  Book a consultation to get started with your website project
                </p>
              </div>
            ) : (
              <div class="space-y-4">
                {dashboard.value?.upcoming_bookings.map((booking) => (
                  <div key={booking.id} class="border border-gray-200 rounded-lg p-4 hover:border-[#9CAF88]/30 transition-colors">
                    <div class="flex items-start justify-between mb-3">
                      <div>
                        <h3 class="font-semibold text-gray-900">{booking.event_name}</h3>
                        <p class="text-sm text-gray-600 flex items-center gap-1 mt-1">
                          <LuClock class="w-4 h-4" />
                          {formatDate(booking.scheduled_at)}
                        </p>
                      </div>
                      <span class={`px-2 py-1 rounded-full text-xs font-medium ${getStatusColor(booking.status)}`}>
                        {booking.status.replace('_', ' ')}
                      </span>
                    </div>
                    
                    <div class="flex items-center justify-between">
                      <div class="flex items-center gap-4 text-sm text-gray-500">
                        <span class="flex items-center gap-1">
                          <LuMail class="w-4 h-4" />
                          {booking.guest_email}
                        </span>
                      </div>
                      <button class="text-[#9CAF88] hover:text-[#9CAF88]/80 text-sm font-medium flex items-center gap-1">
                        View Details
                        <LuArrowRight class="w-4 h-4" />
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* Past Consultations */}
          {dashboard.value?.past_bookings && dashboard.value.past_bookings.length > 0 && (
            <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6 mt-6">
              <h2 class="text-xl font-semibold text-gray-900 mb-6">Recent Consultations</h2>
              <div class="space-y-3">
                {dashboard.value.past_bookings.slice(0, 3).map((booking) => (
                  <div key={booking.id} class="flex items-center justify-between py-3 border-b border-gray-100 last:border-b-0">
                    <div>
                      <p class="font-medium text-gray-900">{booking.event_name}</p>
                      <p class="text-sm text-gray-500">{formatDate(booking.scheduled_at)}</p>
                    </div>
                    <div class="flex items-center gap-2">
                      {booking.proposal_sent && (
                        <span class="bg-green-100 text-green-800 px-2 py-1 rounded text-xs">
                          Proposal Sent
                        </span>
                      )}
                      <span class={`px-2 py-1 rounded-full text-xs font-medium ${getStatusColor(booking.status)}`}>
                        {booking.status}
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>

        {/* Sidebar */}
        <div class="space-y-6">
          {/* Pending Action Items */}
          {dashboard.value?.pending_briefs && dashboard.value.pending_briefs.length > 0 && (
            <div class="bg-gradient-to-br from-orange-50 to-yellow-50 rounded-xl border border-orange-200 p-6">
              <div class="flex items-center gap-2 mb-4">
                <LuFileText class="w-5 h-5 text-orange-600" />
                <h3 class="font-semibold text-orange-900">Action Required</h3>
              </div>
              <div class="space-y-3">
                {dashboard.value.pending_briefs.map((brief) => (
                  <div key={brief.booking_id} class="bg-white rounded-lg p-3 border border-orange-200">
                    <p class="font-medium text-gray-900 text-sm">{brief.event_name}</p>
                    <p class="text-xs text-gray-600 mb-2">
                      Scheduled: {formatDate(brief.scheduled_at)}
                    </p>
                    <a 
                      href={brief.brief_url}
                      class="inline-flex items-center gap-1 bg-orange-600 text-white px-3 py-1 rounded text-xs font-medium hover:bg-orange-700 transition-colors"
                    >
                      Complete Project Brief
                      <LuArrowRight class="w-3 h-3" />
                    </a>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Preparation Materials */}
          <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
            <h3 class="font-semibold text-gray-900 mb-4">Preparation Materials</h3>
            <div class="space-y-3">
              {dashboard.value?.preparation_materials.map((material, index) => (
                <div key={index} class="flex items-start gap-3 p-3 bg-gray-50 rounded-lg">
                  <div class="flex-shrink-0 w-8 h-8 bg-[#9CAF88]/10 rounded-lg flex items-center justify-center">
                    <LuFileText class="w-4 h-4 text-[#9CAF88]" />
                  </div>
                  <div class="flex-1 min-w-0">
                    <p class="font-medium text-gray-900 text-sm">{material.title}</p>
                    <p class="text-xs text-gray-600 capitalize">{material.material_type}</p>
                    {material.content && (
                      <p class="text-xs text-gray-500 mt-1 line-clamp-2">{material.content}</p>
                    )}
                    {material.file_url && (
                      <a 
                        href={material.file_url}
                        class="text-[#9CAF88] hover:text-[#9CAF88]/80 text-xs font-medium mt-1 inline-block"
                      >
                        View Resource →
                      </a>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Quick Actions */}
          <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
            <h3 class="font-semibold text-gray-900 mb-4">Quick Actions</h3>
            <div class="space-y-3">
              <button class="w-full bg-[#9CAF88] text-white px-4 py-2 rounded-lg hover:bg-[#9CAF88]/90 transition-colors text-sm font-medium flex items-center justify-center gap-2">
                <LuCalendar class="w-4 h-4" />
                Schedule New Consultation
              </button>
              <button class="w-full border border-gray-300 text-gray-700 px-4 py-2 rounded-lg hover:bg-gray-50 transition-colors text-sm font-medium flex items-center justify-center gap-2">
                <LuUser class="w-4 h-4" />
                Update Profile
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
});
