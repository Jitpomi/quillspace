import { component$, useSignal, useTask$, $ } from '@builder.io/qwik';
import { useLocation } from '@builder.io/qwik-city';
import { LuCheckCircle, LuCalendar, LuFileText, LuMail, LuArrowRight, LuClock, LuUser, LuBookOpen, LuShield } from '@qwikest/icons/lucide';

export default component$(() => {
  const location = useLocation();
  const eventUuid = location.url.searchParams.get('event');
  const inviteeUuid = location.url.searchParams.get('invitee');
  
  const bookingDetails = useSignal<any>(null);
  const isLoading = useSignal(true);

  // Load booking details if available
  useTask$(async () => {
    if (eventUuid) {
      try {
        // This would fetch booking details from our API
        // For now, we'll show a generic success page
        setTimeout(() => {
          isLoading.value = false;
        }, 1000);
      } catch (e) {
        isLoading.value = false;
      }
    } else {
      isLoading.value = false;
    }
  });

  const nextSteps = [
    {
      icon: LuFileText,
      title: "Complete Your Project Brief",
      description: "Help us prepare for our consultation by filling out your project requirements.",
      action: "Complete Brief",
      href: "/consultations",
      priority: "high",
      timeEstimate: "5-10 minutes"
    },
    {
      icon: LuMail,
      title: "Check Your Email",
      description: "You'll receive a confirmation email with calendar invite and preparation materials.",
      action: "Open Email",
      href: "mailto:",
      priority: "medium",
      timeEstimate: "1 minute"
    },
    {
      icon: LuBookOpen,
      title: "Review Our Portfolio",
      description: "Browse examples of author websites we've created to get inspired.",
      action: "View Portfolio",
      href: "/portfolio",
      priority: "low",
      timeEstimate: "10-15 minutes"
    },
    {
      icon: LuUser,
      title: "Prepare Your Materials",
      description: "Gather your bio, book covers, and any existing content you'd like to include.",
      action: "See Checklist",
      href: "#preparation-checklist",
      priority: "medium",
      timeEstimate: "15-30 minutes"
    }
  ];

  const preparationChecklist = [
    "Author bio and headshot",
    "Book covers and descriptions", 
    "Existing website URL (if any)",
    "Social media handles",
    "Preferred color schemes or design inspiration",
    "List of must-have website features",
    "Content you want to include (press quotes, reviews, etc.)",
    "Questions about the design process"
  ];

  if (isLoading.value) {
    return (
      <div class="min-h-screen bg-gradient-to-br from-[#9CAF88]/5 to-[#7A9B6E]/10 flex items-center justify-center">
        <div class="animate-pulse text-center">
          <div class="w-16 h-16 bg-gray-200 rounded-full mx-auto mb-4"></div>
          <div class="h-4 bg-gray-200 rounded w-48 mx-auto"></div>
        </div>
      </div>
    );
  }

  return (
    <div class="min-h-screen bg-gradient-to-br from-[#9CAF88]/5 to-[#7A9B6E]/10">
      <div class="max-w-4xl mx-auto px-6 py-12">
        {/* Success Header */}
        <div class="text-center mb-12">
          <div class="flex justify-center mb-6">
            <div class="relative">
              <div class="w-20 h-20 bg-green-100 rounded-full flex items-center justify-center">
                <LuCheckCircle class="w-10 h-10 text-green-600" />
              </div>
              <div class="absolute -top-2 -right-2 w-8 h-8 bg-[#9CAF88] rounded-full flex items-center justify-center">
                <LuCalendar class="w-4 h-4 text-white" />
              </div>
            </div>
          </div>
          
          <h1 class="text-4xl font-bold text-gray-900 mb-4">
            🎉 Consultation Booked Successfully!
          </h1>
          <p class="text-xl text-gray-600 mb-2">
            We're excited to help bring your author website to life
          </p>
          <p class="text-gray-500">
            Check your email for the calendar invite and confirmation details
          </p>
        </div>

        {/* What Happens Next */}
        <div class="bg-white rounded-2xl shadow-lg border border-gray-200 p-8 mb-8">
          <h2 class="text-2xl font-semibold text-gray-900 mb-6 flex items-center gap-3">
            <LuArrowRight class="w-6 h-6 text-[#9CAF88]" />
            What Happens Next
          </h2>
          
          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            {nextSteps.map((step, index) => (
              <div key={index} class="relative">
                <div class={`border-2 rounded-xl p-6 transition-all hover:shadow-md ${
                  step.priority === 'high' 
                    ? 'border-orange-200 bg-orange-50' 
                    : step.priority === 'medium'
                    ? 'border-blue-200 bg-blue-50'
                    : 'border-gray-200 bg-gray-50'
                }`}>
                  <div class="flex items-start gap-4">
                    <div class={`flex-shrink-0 w-12 h-12 rounded-lg flex items-center justify-center ${
                      step.priority === 'high'
                        ? 'bg-orange-100'
                        : step.priority === 'medium' 
                        ? 'bg-blue-100'
                        : 'bg-gray-100'
                    }`}>
                      <step.icon class={`w-6 h-6 ${
                        step.priority === 'high'
                          ? 'text-orange-600'
                          : step.priority === 'medium'
                          ? 'text-blue-600' 
                          : 'text-gray-600'
                      }`} />
                    </div>
                    <div class="flex-1">
                      <div class="flex items-center gap-2 mb-2">
                        <h3 class="font-semibold text-gray-900">{step.title}</h3>
                        {step.priority === 'high' && (
                          <span class="bg-orange-100 text-orange-800 px-2 py-0.5 rounded-full text-xs font-medium">
                            Priority
                          </span>
                        )}
                      </div>
                      <p class="text-sm text-gray-600 mb-3">{step.description}</p>
                      <div class="flex items-center justify-between">
                        <span class="text-xs text-gray-500 flex items-center gap-1">
                          <LuClock class="w-3 h-3" />
                          {step.timeEstimate}
                        </span>
                        <a
                          href={step.href}
                          class={`inline-flex items-center gap-1 px-3 py-1.5 rounded-lg text-sm font-medium transition-colors ${
                            step.priority === 'high'
                              ? 'bg-orange-600 text-white hover:bg-orange-700'
                              : 'bg-[#9CAF88] text-white hover:bg-[#9CAF88]/90'
                          }`}
                        >
                          {step.action}
                          <LuArrowRight class="w-3 h-3" />
                        </a>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Preparation Checklist */}
        <div class="bg-white rounded-2xl shadow-lg border border-gray-200 p-8 mb-8" id="preparation-checklist">
          <h2 class="text-2xl font-semibold text-gray-900 mb-6 flex items-center gap-3">
            <LuShield class="w-6 h-6 text-[#9CAF88]" />
            Consultation Preparation Checklist
          </h2>
          <p class="text-gray-600 mb-6">
            Having these materials ready will help us make the most of our consultation time:
          </p>
          
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            {preparationChecklist.map((item, index) => (
              <label key={index} class="flex items-center gap-3 p-3 rounded-lg hover:bg-gray-50 cursor-pointer group">
                <input 
                  type="checkbox" 
                  class="w-5 h-5 text-[#9CAF88] border-gray-300 rounded focus:ring-[#9CAF88]"
                />
                <span class="text-gray-700 group-hover:text-gray-900 transition-colors">
                  {item}
                </span>
              </label>
            ))}
          </div>
          
          <div class="mt-6 p-4 bg-[#9CAF88]/10 rounded-lg border border-[#9CAF88]/20">
            <p class="text-sm text-gray-700">
              <strong>💡 Pro Tip:</strong> Don't worry if you don't have everything ready. 
              We can discuss what's missing and help you gather materials during our consultation.
            </p>
          </div>
        </div>

        {/* What to Expect */}
        <div class="bg-gradient-to-r from-[#9CAF88]/10 to-[#7A9B6E]/10 rounded-2xl border border-[#9CAF88]/20 p-8">
          <h2 class="text-2xl font-semibold text-gray-900 mb-6">
            What to Expect in Our Consultation
          </h2>
          
          <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div class="text-center">
              <div class="w-16 h-16 bg-white rounded-xl shadow-sm flex items-center justify-center mx-auto mb-4">
                <LuUser class="w-8 h-8 text-[#9CAF88]" />
              </div>
              <h3 class="font-semibold text-gray-900 mb-2">Discovery Phase</h3>
              <p class="text-sm text-gray-600">
                We'll discuss your goals, audience, and vision for your author platform
              </p>
            </div>
            
            <div class="text-center">
              <div class="w-16 h-16 bg-white rounded-xl shadow-sm flex items-center justify-center mx-auto mb-4">
                <LuBookOpen class="w-8 h-8 text-[#9CAF88]" />
              </div>
              <h3 class="font-semibold text-gray-900 mb-2">Design Review</h3>
              <p class="text-sm text-gray-600">
                We'll show examples and discuss design options that fit your genre and style
              </p>
            </div>
            
            <div class="text-center">
              <div class="w-16 h-16 bg-white rounded-xl shadow-sm flex items-center justify-center mx-auto mb-4">
                <LuArrowRight class="w-8 h-8 text-[#9CAF88]" />
              </div>
              <h3 class="font-semibold text-gray-900 mb-2">Next Steps</h3>
              <p class="text-sm text-gray-600">
                We'll outline the project timeline, deliverables, and investment options
              </p>
            </div>
          </div>
        </div>

        {/* Contact Information */}
        <div class="text-center mt-12">
          <p class="text-gray-600 mb-4">
            Questions before our consultation?
          </p>
          <div class="flex flex-col sm:flex-row gap-4 justify-center">
            <a 
              href="mailto:hello@quillspace.io"
              class="inline-flex items-center gap-2 text-[#9CAF88] hover:text-[#9CAF88]/80 font-medium"
            >
              <LuMail class="w-4 h-4" />
              hello@quillspace.io
            </a>
            <a 
              href="/consultations"
              class="inline-flex items-center gap-2 text-[#9CAF88] hover:text-[#9CAF88]/80 font-medium"
            >
              <LuCalendar class="w-4 h-4" />
              View Consultation Dashboard
            </a>
          </div>
        </div>
      </div>
    </div>
  );
});
