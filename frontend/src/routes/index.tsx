import { component$, useSignal, $, useVisibleTask$ } from '@builder.io/qwik';
import { routeAction$ } from '@builder.io/qwik-city';
import { LuRocket, LuBarChart3, LuUsers, LuFileText, LuSettings, LuMenu, LuBell, LuSearch, LuLogOut } from '@qwikest/icons/lucide';
import AnalyticsDashboard from '../components/dashboard/analytics-dashboard';
import ContentManagement from '../components/content/content-management';
import Login from '../components/auth/login';
import { api, getAuthToken, clearAuth, type User, type Content, type Tenant } from '../services/api';

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

        // Fetch all data in parallel
        const [analyticsData, contentData, usersData, tenantData, topContent, recentActivity] = await Promise.all([
          api.getAnalyticsMetrics(),
          api.getContent(),
          api.getUsers(),
          api.getCurrentTenant(),
          api.getTopContent(),
          api.getRecentActivity(),
        ]);

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
        event_data: { timestamp: new Date().toISOString() },
      }).catch(console.error);
    }
  });

  const handleLogout = $(() => {
    clearAuth();
    isAuthenticated.value = false;
    currentUser.value = null;
    // Reload the page to reset state
    window.location.reload();
  });

  // Show login if not authenticated
  if (!isAuthenticated.value && !authLoading.value) {
    return <Login />;
  }

  // Show loading state
  if (authLoading.value || isLoading.value) {
    return (
      <div class="min-h-screen flex items-center justify-center">
        <div class="text-center">
          <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto"></div>
          <p class="mt-4 text-gray-600">Loading QuillSpace...</p>
        </div>
      </div>
    );
  }

  // Show error state
  if (error.value) {
    return (
      <div class="min-h-screen  flex items-center justify-center">
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
    <div class="min-h-screen bg-gray-50">
      {/* Sidebar */}
      <div class={`fixed inset-y-0 left-0 z-50 w-64 bg-white shadow-lg transform transition-transform duration-300 ease-in-out ${
        sidebarOpen.value ? 'translate-x-0' : '-translate-x-full'
      } lg:translate-x-0 lg:static lg:inset-0`}>
        <div class="flex items-center justify-center h-16 bg-blue-600">
          <div class="flex items-center gap-2">
            <LuRocket class="w-8 h-8 text-white" />
            <span class="text-xl font-bold text-white">QuillSpace</span>
          </div>
        </div>
        
        <nav class="mt-8">
          <button
            onClick$={() => {
              activeTab.value = 'dashboard';
              handleInteraction('nav_dashboard');
            }}
            class={`w-full flex items-center gap-3 px-6 py-3 text-left transition-colors ${
              activeTab.value === 'dashboard'
                ? 'bg-blue-50 text-blue-600 border-r-2 border-blue-600'
                : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900'
            }`}
          >
            <LuBarChart3 class="w-5 h-5" />
            Dashboard
          </button>
          <button
            onClick$={() => {
              activeTab.value = 'content';
              handleInteraction('nav_content');
            }}
            class={`w-full flex items-center gap-3 px-6 py-3 text-left transition-colors ${
              activeTab.value === 'content'
                ? 'bg-blue-50 text-blue-600 border-r-2 border-blue-600'
                : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900'
            }`}
          >
            <LuFileText class="w-5 h-5" />
            Content
          </button>
          <button
            onClick$={() => {
              activeTab.value = 'users';
              handleInteraction('nav_users');
            }}
            class={`w-full flex items-center gap-3 px-6 py-3 text-left transition-colors ${
              activeTab.value === 'users'
                ? 'bg-blue-50 text-blue-600 border-r-2 border-blue-600'
                : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900'
            }`}
          >
            <LuUsers class="w-5 h-5" />
            Users
          </button>
          <button
            onClick$={() => {
              activeTab.value = 'settings';
              handleInteraction('nav_settings');
            }}
            class={`w-full flex items-center gap-3 px-6 py-3 text-left transition-colors ${
              activeTab.value === 'settings'
                ? 'bg-blue-50 text-blue-600 border-r-2 border-blue-600'
                : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900'
            }`}
          >
            <LuSettings class="w-5 h-5" />
            Settings
          </button>
        </nav>
      </div>

      {/* Main Content */}
      <div class="lg:pl-64">
        {/* Header */}
        <header class="bg-white shadow-sm border-b border-gray-200">
          <div class="flex items-center justify-between px-6 py-4">
            <div class="flex items-center gap-4">
              <button
                onClick$={() => sidebarOpen.value = !sidebarOpen.value}
                class="lg:hidden p-2 rounded-md text-gray-600 hover:text-gray-900 hover:bg-gray-100"
              >
                <LuMenu class="w-6 h-6" />
              </button>
              <h1 class="text-2xl font-semibold text-gray-900">
                {activeTab.value === 'dashboard' && 'Dashboard'}
                {activeTab.value === 'content' && 'Content Management'}
                {activeTab.value === 'users' && 'User Management'}
                {activeTab.value === 'settings' && 'Settings'}
              </h1>
            </div>
            
            <div class="flex items-center gap-4">
              <div class="relative">
                <LuSearch class="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
                <input
                  type="text"
                  placeholder="Search..."
                  class="pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                />
              </div>
              <button class="p-2 text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded-lg">
                <LuBell class="w-5 h-5" />
              </button>
              <div class="flex items-center gap-3">
                <div class="text-right">
                  <p class="text-sm font-medium text-gray-900">{currentUser.value?.name}</p>
                  <p class="text-xs text-gray-500">{currentUser.value?.role}</p>
                </div>
                <button 
                  onClick$={handleLogout}
                  class="p-2 text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded-lg"
                >
                  <LuLogOut class="w-5 h-5" />
                </button>
              </div>
            </div>
          </div>
        </header>

        {/* Page Content */}
        <main class="p-6">
          {activeTab.value === 'dashboard' && analytics.value && (
            <AnalyticsDashboard 
              data={analytics.value} 
            />
          )}
          
          {activeTab.value === 'content' && (
            <ContentManagement
              content={content.value}
            />
          )}
          
          {activeTab.value === 'users' && (
            <div class="bg-white rounded-xl shadow-lg p-8 text-center">
              <LuUsers class="w-16 h-16 text-gray-400 mx-auto mb-4" />
              <h3 class="text-xl font-semibold text-gray-900 mb-2">User Management</h3>
              <p class="text-gray-600">User management component coming soon...</p>
            </div>
          )}
          
          {activeTab.value === 'settings' && (
            <div class="bg-white rounded-xl shadow-lg p-8 text-center">
              <LuSettings class="w-16 h-16 text-gray-400 mx-auto mb-4" />
              <h3 class="text-xl font-semibold text-gray-900 mb-2">Settings</h3>
              <p class="text-gray-600">Settings component coming soon...</p>
            </div>
          )}
        </main>
      </div>

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
