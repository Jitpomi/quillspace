import { component$, useSignal, $, useVisibleTask$ } from '@builder.io/qwik';
import { routeAction$ } from '@builder.io/qwik-city';
import { LuRocket, LuBarChart3, LuUsers, LuFileText, LuSettings, LuMenu, LuLogOut, LuGlobe } from '@qwikest/icons/lucide';
import ContentManagement from '../components/content/content-management';
import Login from '../components/auth/login';
import { api, getAuthToken, clearAuth, type User, type Content, type Tenant } from "../services/api";
import WebsiteBuilder from "../components/website-builder/website-builder";

// This loader is now replaced by real-time data fetching in the component

export const useRecordEventAction = routeAction$(async (data, requestEvent) => {
  // Record analytics event to QuillSpace backend
  const eventData = {
    event_type: data.event_type,
    event_data: {
      page_path: requestEvent.url.pathname,
      timestamp: new Date().toISOString(),
    },
    session_id: requestEvent.cookie.get('session_id')?.value,
    ip_address: requestEvent.clientConn.ip,
    user_agent: requestEvent.request.headers.get('user-agent'),
  };

  try {
    // In production, make API call to QuillSpace backend
    console.log('Analytics event recorded:', eventData);
    return { success: true };
  } catch (error) {
    console.error('Failed to record analytics event:', error);
    return { success: false, error: 'Failed to record event' };
  }
});

export default component$(() => {
  const recordEvent = useRecordEventAction();
  
  // Component state
  const activeTab = useSignal('dashboard');
  const sidebarOpen = useSignal(false);
  
  // Auth state
  const isAuthenticated = useSignal(false);
  const currentUser = useSignal<User | null>(null);
  const authLoading = useSignal(true);
  
  // Data state
  const analytics = useSignal<any>(null);
  const content = useSignal<Content[]>([]);
  const users = useSignal<User[]>([]);
  const tenant = useSignal<Tenant | null>(null);
  const isLoading = useSignal(true);
  const error = useSignal<string | null>(null);

  // Load data function
  const loadData = $(async () => {
    if (isAuthenticated.value && currentUser.value) {
      try {
        isLoading.value = true;
        error.value = null;

        // Fetch data with fallbacks for missing endpoints
        let analyticsData: any, contentData: Content[], usersData: User[], tenantData: Tenant, topContent: any[], recentActivity: any[];
        
        try {
          analyticsData = await api.getAnalyticsMetrics();
        } catch {
          analyticsData = { total_events: 1250, unique_users: 340, page_views: 5680, content_published: 45, growth_rate: 12.5 };
        }
        
        try {
          contentData = await api.getContent();
        } catch {
          contentData = [];
        }
        
        try {
          usersData = await api.getUsers();
        } catch {
          usersData = [currentUser.value];
        }
        
        try {
          tenantData = await api.getCurrentTenant();
        } catch {
          tenantData = { 
            id: 'demo', 
            name: 'Demo Company', 
            slug: 'demo',
            settings: {
              branding: { primary_color: '#3b82f6' },
              features: { analytics_enabled: true, comments_enabled: true, seo_enabled: true },
              security: { two_factor_required: false, password_policy: 'basic' }
            },
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
            is_active: true
          };
        }
        
        try {
          topContent = await api.getTopContent();
        } catch {
          topContent = [];
        }
        
        try {
          recentActivity = await api.getRecentActivity();
        } catch {
          recentActivity = [];
        }

        analytics.value = {
          totalEvents: analyticsData.total_events,
          uniqueUsers: analyticsData.unique_users,
          pageViews: analyticsData.page_views,
          contentPublished: analyticsData.content_published,
          growthRate: analyticsData.growth_rate,
          topContent,
          recentActivity,
        };
        
        content.value = contentData;
        users.value = usersData;
        tenant.value = tenantData;
      } catch (err) {
        error.value = err instanceof Error ? err.message : 'Failed to load data';
        console.error('Failed to load dashboard data:', err);
      } finally {
        isLoading.value = false;
      }
    }
  });

  // Check authentication and load data (browser-only)
  // eslint-disable-next-line qwik/no-use-visible-task
  useVisibleTask$(async () => {
    const token = getAuthToken();
    
    if (token) {
      try {
        // Verify token is still valid
        const user = await api.getCurrentUser();
        currentUser.value = user;
        isAuthenticated.value = true;
        
        // Load dashboard data
        await loadData();
      } catch (err) {
        // Token is invalid, clear auth
        clearAuth();
        isAuthenticated.value = false;
        currentUser.value = null;
      }
    } else {
      isAuthenticated.value = false;
    }
    
    authLoading.value = false;
  });

  const handleInteraction = $((eventType: string) => {
    // Record event both locally and to backend
    recordEvent.submit({ event_type: eventType });
    
    // Also record to backend analytics
    if (isAuthenticated.value) {
      api.recordEvent({
        event_type: eventType,
        properties: { timestamp: new Date().toISOString() },
      }).catch(console.error);
    }
  });

  const handleLogout = $(async () => {
    try {
      // Call backend logout endpoint
      await api.logout();
    } catch (err) {
      console.error('Backend logout failed:', err);
    } finally {
      // Always clear local auth state
      clearAuth();
      isAuthenticated.value = false;
      currentUser.value = null;
      // Reload the page to reset state
      window.location.reload();
    }
  });

  // Show login if not authenticated
  if (!isAuthenticated.value && !authLoading.value) {
    return (
      <div class="min-h-screen bg-[#FEFCF7] flex items-center justify-center p-4">
        <div class="w-full max-w-md">
          <Login />
        </div>
      </div>
    );
  }

  // Show loading state
  if (authLoading.value || isLoading.value) {
    return (
      <div class="min-h-screen bg-[#FEFCF7] flex items-center justify-center">
        <div class="text-center">
          <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-[#9CAF88] mx-auto"></div>
          <p class="mt-4 text-gray-600">Loading your sanctuary...</p>
        </div>
      </div>
    );
  }

  // Show error state
  if (error.value) {
    return (
      <div class="min-h-screen bg-[#FEFCF7] flex items-center justify-center">
        <div class="text-center">
          <div class="bg-red-50 border border-red-200 rounded-lg p-6 max-w-md">
            <h3 class="text-lg font-medium text-red-800 mb-2">Error Loading Dashboard</h3>
            <p class="text-red-700">{error.value}</p>
            <button
              onClick$={() => window.location.reload()}
              class="mt-4 bg-red-600 text-white px-4 py-2 rounded-lg hover:bg-red-700"
            >
              Retry
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div class="min-h-screen bg-[#FEFCF7]">
      {/* Sidebar - Always visible on desktop, toggleable on mobile */}
      <div class="fixed inset-y-0 left-0 z-50 w-64 bg-[#F7F3E9] shadow-sm border-r border-[#E8E2D4] hidden lg:block">
        {/* Desktop Sidebar - Always Visible */}
        <div class="flex items-center justify-center h-16 border-b border-[#E8E2D4] bg-[#F7F3E9]">
          <div class="flex items-center gap-2">
            <LuRocket class="w-8 h-8 text-[#2D3748]" />
            <span class="text-xl font-bold text-[#2D3748]">QuillSpace</span>
          </div>
        </div>
        
        <nav class="mt-8 px-4">
          <div class="space-y-2">
            <button
              onClick$={() => {
                activeTab.value = 'dashboard';
                handleInteraction('nav_dashboard');
                sidebarOpen.value = false; // Ensure mobile menu closes
              }}
              class={`w-full flex items-center gap-3 px-3 py-3 text-left rounded-lg transition-all ${
                activeTab.value === 'dashboard'
                  ? 'bg-[#9CAF88]/20 text-[#2D3748] font-medium'
                  : 'text-gray-600 hover:bg-[#9CAF88]/10 hover:text-[#2D3748]'
              }`}
            >
              <LuBarChart3 class="w-5 h-5" />
              <span>Dashboard</span>
            </button>
            
            <button
              onClick$={() => {
                activeTab.value = 'content';
                handleInteraction('nav_content');
                sidebarOpen.value = false;
              }}
              class={`w-full flex items-center gap-3 px-3 py-3 text-left rounded-lg transition-all ${
                activeTab.value === 'content'
                  ? 'bg-[#9CAF88]/20 text-[#2D3748] font-medium'
                  : 'text-gray-600 hover:bg-[#9CAF88]/10 hover:text-[#2D3748]'
              }`}
            >
              <LuFileText class="w-5 h-5" />
              <span>My Writing</span>
            </button>
            
            <button
              onClick$={() => {
                activeTab.value = "website-builder";
                handleInteraction("nav_website");
                sidebarOpen.value = false;
              }}
              class={`w-full flex items-center gap-3 px-3 py-3 text-left rounded-lg transition-all ${
                activeTab.value === "website-builder"
                  ? "bg-[#9CAF88]/20 text-[#2D3748] font-medium"
                  : "text-gray-600 hover:bg-[#9CAF88]/10 hover:text-[#2D3748]"
              }`}
            >
              <LuGlobe class="w-5 h-5" />
              <span>My Website</span>            </button>
            
            <button
              onClick$={() => {
                activeTab.value = 'users';
                handleInteraction('nav_users');
                sidebarOpen.value = false;
              }}
              class={`w-full flex items-center gap-3 px-3 py-3 text-left rounded-lg transition-all ${
                activeTab.value === 'users'
                  ? 'bg-[#9CAF88]/20 text-[#2D3748] font-medium'
                  : 'text-gray-600 hover:bg-[#9CAF88]/10 hover:text-[#2D3748]'
              }`}
            >
              <LuUsers class="w-5 h-5" />
              <span>Readers</span>
            </button>
            
            <button
              onClick$={() => {
                activeTab.value = 'settings';
                handleInteraction('nav_settings');
                sidebarOpen.value = false;
              }}
              class={`w-full flex items-center gap-3 px-3 py-3 text-left rounded-lg transition-all ${
                activeTab.value === 'settings'
                  ? 'bg-[#9CAF88]/20 text-[#2D3748] font-medium'
                  : 'text-gray-600 hover:bg-[#9CAF88]/10 hover:text-[#2D3748]'
              }`}
            >
              <LuSettings class="w-5 h-5" />
              <span>Settings</span>
            </button>
          </div>
        </nav>
      </div>

      {/* Mobile Sidebar - Toggleable */}
      <div class={`fixed inset-y-0 left-0 z-50 w-64 bg-[#F7F3E9] shadow-sm border-r border-[#E8E2D4] transform transition-transform duration-300 ease-in-out lg:hidden ${
        sidebarOpen.value ? 'translate-x-0' : '-translate-x-full'
      }`}>
        <div class="flex items-center justify-center h-16 border-b border-[#E8E2D4] bg-[#F7F3E9]">
          <div class="flex items-center gap-2">
            <LuRocket class="w-8 h-8 text-[#2D3748]" />
            <span class="text-xl font-bold text-[#2D3748]">QuillSpace</span>
          </div>
        </div>
        
      </div>

      {/* Main Content */}
      <div class="lg:pl-64">
        {/* Header - Minimal and clean */}
        <header class="bg-[#FEFCF7] border-b border-[#E8E2D4]">
          <div class="flex items-center justify-between px-4 py-3">
            {/* Left side - Just hamburger menu on mobile */}
            <div class="flex items-center">
              <button
                onClick$={() => sidebarOpen.value = !sidebarOpen.value}
                class="lg:hidden p-2 rounded-md text-gray-600 hover:text-gray-900 hover:bg-gray-100"
              >
                <LuMenu class="w-5 h-5" />
              </button>
              {/* Title only on large screens */}
              <h1 class="hidden lg:block text-2xl font-semibold text-[#2D3748] ml-4">
                {activeTab.value === 'dashboard' && 'Dashboard'}
                {activeTab.value === 'content' && 'My Writing'}
                {activeTab.value === 'users' && 'Readers'}
                {activeTab.value === 'settings' && 'Settings'}
              </h1>
            </div>
            
            {/* Right side - User info only */}
            <div class="flex items-center gap-2">
              {/* Desktop only - Create button + user info */}
              <div class="hidden lg:flex items-center gap-3">
                <button class="bg-[#9CAF88] hover:bg-[#8a9e7a] text-white px-4 py-2 rounded-lg font-medium transition-colors">
                  Create
                </button>
                <span class="text-sm text-gray-600">Welcome back, <span class="font-medium text-gray-900">{currentUser.value?.name}</span></span>
                <button 
                  onClick$={handleLogout}
                  class="flex items-center gap-2 px-3 py-1.5 text-sm text-gray-600 hover:text-gray-800 hover:bg-gray-50 rounded-md transition-colors"
                  title="Sign out"
                >
                  <LuLogOut class="w-4 h-4" />
                  <span>Sign out</span>
                </button>
              </div>
              
              {/* Mobile only - just logout icon */}
              <button 
                onClick$={handleLogout}
                class="lg:hidden p-2 rounded-md text-gray-600 hover:text-gray-800 hover:bg-gray-50 transition-colors"
                title="Sign out"
              >
                <LuLogOut class="w-5 h-5" />
              </button>
            </div>
          </div>
        </header>

        {/* Page Content */}
        <main class="p-6">
          {activeTab.value === 'dashboard' && (
            <div class="max-w-6xl mx-auto space-y-8">
              {/* Welcome Message */}
              <div class="text-center py-8">
                <h2 class="text-4xl font-serif font-semibold text-[#2D3748] mb-3 leading-tight">Welcome home, writer</h2>
                <p class="text-lg font-sans text-gray-600 leading-relaxed max-w-2xl mx-auto">This is your sanctuary—where thoughts become stories, and stories find the hearts they're meant to touch.</p>
                <div class="mt-4 text-sm font-sans text-[#9CAF88] italic">✨ Take a deep breath. Your creative space awaits.</div>
              </div>

              {/* Quick Actions - Ultra Clean */}
              <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                <div 
                  class="bg-[#FEFCF7] rounded-xl border border-[#E8E2D4] p-8 text-center shadow-warm hover-lift transition-gentle cursor-pointer group"
                  onClick$={() => activeTab.value = 'website-builder'}
                >
                  <div class="w-16 h-16 bg-[#9CAF88]/20 rounded-full flex items-center justify-center mx-auto mb-4 group-hover:bg-[#9CAF88]/30 transition-soft animate-breathe">
                    <LuFileText class="w-8 h-8 text-[#2D3748] group-hover:scale-110 transition-soft" />
                  </div>
                  <h3 class="text-xl font-serif font-semibold text-[#2D3748] mb-2">Create Your Website</h3>
                  <p class="font-sans text-gray-600 mb-4 leading-relaxed">Build a beautiful website to showcase and sell your books.</p>
                  <div class="text-[#9CAF88] font-medium font-sans">Choose Template →</div>
                  <div class="mt-2 text-xs text-gray-500 italic">"Your words deserve a beautiful home"</div>
                </div>

                <div class="bg-[#FEFCF7] rounded-xl border border-[#E8E2D4] p-8 text-center shadow-warm hover-lift transition-gentle cursor-pointer group">
                  <div class="w-16 h-16 bg-[#7C9CBF]/20 rounded-full flex items-center justify-center mx-auto mb-4 group-hover:bg-[#7C9CBF]/30 transition-soft">
                    <LuUsers class="w-8 h-8 text-[#2D3748] group-hover:scale-110 transition-soft" />
                  </div>
                  <h3 class="text-xl font-serif font-semibold text-[#2D3748] mb-2">Your Readers</h3>
                  <p class="font-sans text-gray-600 mb-4 leading-relaxed">Connect with the people who love your writing and want to hear more.</p>
                  <div class="text-[#7C9CBF] font-medium font-sans">View Readers →</div>
                  <div class="mt-2 text-xs text-gray-500 italic">"Writing is a conversation across time"</div>
                </div>

                <div class="bg-[#FEFCF7] rounded-xl border border-[#E8E2D4] p-8 text-center shadow-warm hover-lift transition-gentle cursor-pointer group">
                  <div class="w-16 h-16 bg-[#B8A9C9]/20 rounded-full flex items-center justify-center mx-auto mb-4 group-hover:bg-[#B8A9C9]/30 transition-soft">
                    <LuBarChart3 class="w-8 h-8 text-[#2D3748] group-hover:scale-110 transition-soft" />
                  </div>
                  <h3 class="text-xl font-serif font-semibold text-[#2D3748] mb-2">Your Impact</h3>
                  <p class="font-sans text-gray-600 mb-4 leading-relaxed">See how your words are touching hearts and inspiring minds.</p>
                  <div class="text-[#B8A9C9] font-medium font-sans">View Stats →</div>
                  <div class="mt-2 text-xs text-gray-500 italic">"Words have power beyond measure"</div>
                </div>
              </div>

              {/* Writing Stats - Celebration of Progress */}
              <div class="bg-[#F7F3E9] rounded-xl border border-[#E8E2D4] p-8 shadow-warm">
                <h3 class="text-2xl font-serif font-semibold text-[#2D3748] mb-2 text-center">Your Writing Journey</h3>
                <p class="text-center text-gray-600 font-sans mb-6 italic">Every word you write matters. Look how far you've come.</p>
                <div class="grid grid-cols-2 md:grid-cols-4 gap-8 text-center">
                  <div class="group cursor-pointer transition-gentle hover-lift">
                    <div class="text-4xl font-serif font-bold text-[#9CAF88] mb-1 group-hover:scale-110 transition-soft">12</div>
                    <div class="text-sm font-sans text-gray-600 font-medium">Stories Shared</div>
                    <div class="text-xs text-gray-500 mt-1">🌱 Growing</div>
                  </div>
                  <div class="group cursor-pointer transition-gentle hover-lift">
                    <div class="text-4xl font-serif font-bold text-[#7C9CBF] mb-1 group-hover:scale-110 transition-soft">2.3k</div>
                    <div class="text-sm font-sans text-gray-600 font-medium">Hearts Touched</div>
                    <div class="text-xs text-gray-500 mt-1">💝 Inspiring</div>
                  </div>
                  <div class="group cursor-pointer transition-gentle hover-lift">
                    <div class="text-4xl font-serif font-bold text-[#B8A9C9] mb-1 group-hover:scale-110 transition-soft">156</div>
                    <div class="text-sm font-sans text-gray-600 font-medium">Loyal Readers</div>
                    <div class="text-xs text-gray-500 mt-1">🤝 Connected</div>
                  </div>
                  <div class="group cursor-pointer transition-gentle hover-lift">
                    <div class="text-4xl font-serif font-bold text-[#2D3748] mb-1 group-hover:scale-110 transition-soft">4.2m</div>
                    <div class="text-sm font-sans text-gray-600 font-medium">Time Well Spent</div>
                    <div class="text-xs text-gray-500 mt-1">⏰ Meaningful</div>
                  </div>
                </div>
                <div class="mt-6 text-center">
                  <div class="text-sm font-sans text-[#9CAF88] italic">✨ "You're not just writing—you're creating connections that matter."</div>
                </div>
              </div>
            </div>
          )}
          
          {activeTab.value === 'content' && (
            <div class="max-w-6xl mx-auto">
              <div class="text-center py-8 mb-8">
                <h2 class="text-3xl font-bold text-gray-900 mb-2">Your Writing</h2>
                <p class="text-lg text-gray-600">All your articles, drafts, and published pieces in one place.</p>
              </div>
              <ContentManagement content={content.value} />
            </div>
          )}
          
          {activeTab.value === 'website-builder' && (
            <div class="max-w-7xl mx-auto">
              <WebsiteBuilder />
            </div>
          )}
                    {activeTab.value === 'users' && (
            <div class="max-w-6xl mx-auto">
              <div class="text-center py-8 mb-8">
                <h2 class="text-3xl font-bold text-gray-900 mb-2">Your Readers</h2>
                <p class="text-lg text-gray-600">The people who love your writing and want to hear from you.</p>
              </div>
              <div class="bg-white rounded-xl border border-gray-200 p-12 text-center">
                <div class="w-20 h-20 bg-blue-100 rounded-full flex items-center justify-center mx-auto mb-6">
                  <LuUsers class="w-10 h-10 text-blue-600" />
                </div>
                <h3 class="text-2xl font-semibold text-gray-900 mb-3">Connect with Your Audience</h3>
                <p class="text-gray-600 mb-6 max-w-md mx-auto">Build meaningful relationships with readers who appreciate your work. Share your thoughts, get feedback, and grow your community.</p>
                <button class="bg-blue-600 hover:bg-blue-700 text-white px-6 py-3 rounded-lg font-medium transition-colors">
                  Add Readers
                </button>
              </div>
            </div>
          )}
          
          {activeTab.value === 'settings' && (
            <div class="max-w-6xl mx-auto">
              <div class="text-center py-8 mb-8">
                <h2 class="text-3xl font-bold text-gray-900 mb-2">Settings</h2>
                <p class="text-lg text-gray-600">Customize your writing space to work exactly how you want.</p>
              </div>
              <div class="bg-white rounded-xl border border-gray-200 p-12 text-center">
                <div class="w-20 h-20 bg-green-100 rounded-full flex items-center justify-center mx-auto mb-6">
                  <LuSettings class="w-10 h-10 text-green-600" />
                </div>
                <h3 class="text-2xl font-semibold text-gray-900 mb-3">Your Perfect Writing Environment</h3>
                <p class="text-gray-600 mb-6 max-w-md mx-auto">Set up your workspace, customize your editor, manage your profile, and configure how you want to connect with readers.</p>
                <button class="bg-[#9CAF88] hover:bg-[#8a9e7a] text-white px-6 py-3 rounded-lg font-medium transition-colors">
                  Customize Workspace
                </button>
              </div>
            </div>
          )}
        </main>
      </div>

      {/* Floating Action Button - Mobile only */}
      <button 
        class="lg:hidden fixed bottom-6 right-6 bg-[#9CAF88] hover:bg-[#8a9e7a] text-white p-4 rounded-full shadow-lg hover:shadow-xl transition-all z-50"
        onClick$={() => {
          activeTab.value = 'content';
          handleInteraction('fab_create');
        }}
        title="Create new article"
      >
        <LuFileText class="w-6 h-6" />
      </button>

      {/* Mobile Sidebar Overlay */}
      {sidebarOpen.value && (
        <div 
          class="fixed inset-0 bg-gray-600 bg-opacity-75 z-40 lg:hidden"
          onClick$={() => sidebarOpen.value = false}
        />
      )}
    </div>
  );
});
