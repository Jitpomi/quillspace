import { component$, useSignal, $ } from '@builder.io/qwik';
import { routeLoader$, routeAction$ } from '@builder.io/qwik-city';
import { LuRocket, LuBarChart3, LuUsers, LuFileText, LuSettings, LuMenu, LuBell, LuSearch, LuLogOut } from '@qwikest/icons/lucide';
import AnalyticsDashboard from '../components/dashboard/analytics-dashboard';
import ContentManagement from '../components/content/content-management';

export const useQuillSpaceLoader = routeLoader$(async (requestEvent) => {
  // In a real app, this would fetch from your QuillSpace API
  // For demo purposes, we'll return comprehensive mock data
  const mockData = {
    analytics: {
      totalEvents: 15420,
      uniqueUsers: 1250,
      pageViews: 8930,
      contentPublished: 45,
      growthRate: 12.5,
      topContent: [
        { title: 'Getting Started with QuillSpace', views: 2340 },
        { title: 'Advanced Analytics Guide', views: 1890 },
        { title: 'Multi-Tenant Architecture', views: 1560 },
      ],
      recentActivity: [
        { type: 'content', description: 'New article published', timestamp: '2 minutes ago' },
        { type: 'user', description: 'User invited to workspace', timestamp: '15 minutes ago' },
        { type: 'analytics', description: 'Analytics report generated', timestamp: '1 hour ago' },
      ]
    },
    content: [
      {
        id: '1',
        title: 'Getting Started with QuillSpace',
        slug: 'getting-started-quillspace',
        status: 'published' as const,
        author: 'John Doe',
        created_at: '2024-01-15T10:00:00Z',
        updated_at: '2024-01-15T10:00:00Z',
        published_at: '2024-01-15T12:00:00Z'
      },
      {
        id: '2',
        title: 'Advanced Analytics Guide',
        slug: 'advanced-analytics-guide',
        status: 'draft' as const,
        author: 'Jane Smith',
        created_at: '2024-01-14T09:00:00Z',
        updated_at: '2024-01-14T09:00:00Z'
      }
    ],
    currentUser: {
      id: '1',
      name: 'Admin User',
      email: 'admin@example.com',
      role: 'admin'
    }
  };
  
  return mockData;
});

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
  const data = useQuillSpaceLoader();
  const recordEvent = useRecordEventAction();
  const activeTab = useSignal('dashboard');
  const sidebarOpen = useSignal(false);

  const handleInteraction = $((eventType: string) => {
    recordEvent.submit({ event_type: eventType });
  });

  const navigation = [
    { id: 'dashboard', label: 'Dashboard' },
    { id: 'content', label: 'Content' },
    { id: 'users', label: 'Users' },
    { id: 'settings', label: 'Settings' },
  ];

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
                {navigation.find(nav => nav.id === activeTab.value)?.label || 'Dashboard'}
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
                  <p class="text-sm font-medium text-gray-900">{data.value.currentUser.name}</p>
                  <p class="text-xs text-gray-500">{data.value.currentUser.role}</p>
                </div>
                <button class="p-2 text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded-lg">
                  <LuLogOut class="w-5 h-5" />
                </button>
              </div>
            </div>
          </div>
        </header>

        {/* Page Content */}
        <main class="p-6">
          {activeTab.value === 'dashboard' && (
            <AnalyticsDashboard 
              data={data.value.analytics} 
              onInteraction$={handleInteraction}
            />
          )}
          
          {activeTab.value === 'content' && (
            <ContentManagement
              content={data.value.content}
              onCreateContent$={() => handleInteraction('content_create')}
              onEditContent$={(id) => handleInteraction(`content_edit_${id}`)}
              onDeleteContent$={(id) => handleInteraction(`content_delete_${id}`)}
              onPublishContent$={(id) => handleInteraction(`content_publish_${id}`)}
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
