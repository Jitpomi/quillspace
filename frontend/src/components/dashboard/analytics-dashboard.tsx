import { component$ } from '@builder.io/qwik';
import { LuBarChart3, LuUsers, LuFileText, LuZap, LuTrendingUp, LuEye } from '@qwikest/icons/lucide';

interface AnalyticsData {
  totalEvents: number;
  uniqueUsers: number;
  pageViews: number;
  contentPublished: number;
  growthRate: number;
  topContent: Array<{ title: string; views: number; }>;
  recentActivity: Array<{ type: string; description: string; timestamp: string; }>;
}

interface AnalyticsDashboardProps {
  data: AnalyticsData;
}

export default component$<AnalyticsDashboardProps>(({ data }) => {
  return (
    <div class="space-y-8">
      {/* Analytics Overview */}
      <section class="bg-white rounded-xl shadow-lg p-8">
        <div class="flex items-center gap-3 mb-6">
          <LuBarChart3 class="w-8 h-8 text-green-600" />
          <h2 class="text-3xl font-bold text-gray-900">Real-Time Analytics</h2>
        </div>
        
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          <div class="bg-gradient-to-br from-blue-50 to-blue-100 p-6 rounded-lg">
            <div class="flex items-center justify-between">
              <div>
                <p class="text-blue-600 font-semibold">Total Events</p>
                <p class="text-3xl font-bold text-blue-900">{data.totalEvents.toLocaleString()}</p>
              </div>
              <LuZap class="w-8 h-8 text-blue-600" />
            </div>
          </div>
          
          <div class="bg-gradient-to-br from-green-50 to-green-100 p-6 rounded-lg">
            <div class="flex items-center justify-between">
              <div>
                <p class="text-green-600 font-semibold">Unique Users</p>
                <p class="text-3xl font-bold text-green-900">{data.uniqueUsers.toLocaleString()}</p>
              </div>
              <LuUsers class="w-8 h-8 text-green-600" />
            </div>
          </div>
          
          <div class="bg-gradient-to-br from-purple-50 to-purple-100 p-6 rounded-lg">
            <div class="flex items-center justify-between">
              <div>
                <p class="text-purple-600 font-semibold">Page Views</p>
                <p class="text-3xl font-bold text-purple-900">{data.pageViews.toLocaleString()}</p>
              </div>
              <LuEye class="w-8 h-8 text-purple-600" />
            </div>
          </div>
          
          <div class="bg-gradient-to-br from-orange-50 to-orange-100 p-6 rounded-lg">
            <div class="flex items-center justify-between">
              <div>
                <p class="text-orange-600 font-semibold">Content Published</p>
                <p class="text-3xl font-bold text-orange-900">{data.contentPublished}</p>
              </div>
              <LuFileText class="w-8 h-8 text-orange-600" />
            </div>
          </div>
        </div>

        <div class="mt-6 flex items-center gap-2 text-sm">
          <LuTrendingUp class="w-4 h-4 text-green-500" />
          <span class="text-green-600 font-semibold">+{data.growthRate}%</span>
          <span class="text-gray-600">growth this month</span>
        </div>
      </section>

      {/* Top Content & Recent Activity */}
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
        <section class="bg-white rounded-xl shadow-lg p-6">
          <h3 class="text-xl font-bold text-gray-900 mb-4">Top Content</h3>
          <div class="space-y-3">
            {data.topContent.map((content, index) => (
              <div key={index} class="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                <span class="font-medium text-gray-900">{content.title}</span>
                <span class="text-sm text-gray-600">{content.views.toLocaleString()} views</span>
              </div>
            ))}
          </div>
        </section>

        <section class="bg-white rounded-xl shadow-lg p-6">
          <h3 class="text-xl font-bold text-gray-900 mb-4">Recent Activity</h3>
          <div class="space-y-3">
            {data.recentActivity.map((activity, index) => (
              <div key={index} class="flex items-start gap-3 p-3 bg-gray-50 rounded-lg">
                <div class="w-2 h-2 bg-blue-500 rounded-full mt-2"></div>
                <div class="flex-1">
                  <p class="text-sm font-medium text-gray-900">{activity.description}</p>
                  <p class="text-xs text-gray-500">{activity.timestamp}</p>
                </div>
              </div>
            ))}
          </div>
        </section>
      </div>
    </div>
  );
});
