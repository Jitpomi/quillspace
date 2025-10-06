import { component$ } from '@builder.io/qwik';
import { LuPenTool, LuFileText, LuEye, LuClock } from '@qwikest/icons/lucide';

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
  // Transform generic data into writer-focused metrics
  const writerStats = {
    wordsWritten: data.totalEvents * 250, // Estimate words from events
    articlesPublished: data.contentPublished,
    totalReads: data.pageViews,
    avgReadTime: Math.round(data.uniqueUsers / 60), // Convert to minutes
  };

  const recentArticles = [
    { title: "Getting Started with Technical Writing", status: "Published", date: "2 days ago", reads: 1240 },
    { title: "Content Strategy for 2024", status: "Draft", date: "1 week ago", reads: 0 },
    { title: "Building Your Writing Portfolio", status: "Published", date: "1 week ago", reads: 890 },
  ];

  const writingGoals = [
    { goal: "Publish 5 articles this month", progress: 60, current: 3, target: 5 },
    { goal: "Reach 10k total reads", progress: 85, current: 8500, target: 10000 },
    { goal: "Write 50k words", progress: 40, current: 20000, target: 50000 },
  ];

  return (
    <div class="space-y-6">
      {/* Writing Stats Overview */}
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm font-medium text-gray-600">Words Written</p>
              <p class="text-2xl font-bold text-gray-900">{writerStats.wordsWritten.toLocaleString()}</p>
              <p class="text-xs text-green-600 mt-1">This month</p>
            </div>
            <LuPenTool class="w-8 h-8 text-blue-500" />
          </div>
        </div>
        
        <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm font-medium text-gray-600">Articles Published</p>
              <p class="text-2xl font-bold text-gray-900">{writerStats.articlesPublished}</p>
              <p class="text-xs text-green-600 mt-1">+2 this week</p>
            </div>
            <LuFileText class="w-8 h-8 text-green-500" />
          </div>
        </div>
        
        <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm font-medium text-gray-600">Total Reads</p>
              <p class="text-2xl font-bold text-gray-900">{writerStats.totalReads.toLocaleString()}</p>
              <p class="text-xs text-green-600 mt-1">+12% vs last month</p>
            </div>
            <LuEye class="w-8 h-8 text-purple-500" />
          </div>
        </div>
        
        <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm font-medium text-gray-600">Avg. Read Time</p>
              <p class="text-2xl font-bold text-gray-900">{writerStats.avgReadTime}m</p>
              <p class="text-xs text-gray-500 mt-1">Per article</p>
            </div>
            <LuClock class="w-8 h-8 text-orange-500" />
          </div>
        </div>
      </div>

      {/* Recent Articles & Writing Goals */}
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
          <h3 class="text-lg font-semibold text-gray-900 mb-4">Recent Articles</h3>
          <div class="space-y-3">
            {recentArticles.map((article, index) => (
              <div key={index} class="flex items-center justify-between p-3 hover:bg-gray-50 rounded-lg transition-colors">
                <div class="flex-1">
                  <h4 class="font-medium text-gray-900 text-sm">{article.title}</h4>
                  <div class="flex items-center gap-3 mt-1">
                    <span class={`text-xs px-2 py-1 rounded-full ${
                      article.status === 'Published' 
                        ? 'bg-green-100 text-green-700' 
                        : 'bg-yellow-100 text-yellow-700'
                    }`}>
                      {article.status}
                    </span>
                    <span class="text-xs text-gray-500">{article.date}</span>
                  </div>
                </div>
                <div class="text-right">
                  <p class="text-sm font-medium text-gray-900">{article.reads > 0 ? article.reads.toLocaleString() : '—'}</p>
                  <p class="text-xs text-gray-500">reads</p>
                </div>
              </div>
            ))}
          </div>
        </div>

        <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
          <h3 class="text-lg font-semibold text-gray-900 mb-4">Writing Goals</h3>
          <div class="space-y-4">
            {writingGoals.map((goal, index) => (
              <div key={index} class="space-y-2">
                <div class="flex items-center justify-between">
                  <h4 class="text-sm font-medium text-gray-900">{goal.goal}</h4>
                  <span class="text-xs text-gray-500">{goal.current.toLocaleString()} / {goal.target.toLocaleString()}</span>
                </div>
                <div class="w-full bg-gray-200 rounded-full h-2">
                  <div 
                    class="bg-blue-500 h-2 rounded-full transition-all duration-300" 
                    style={`width: ${goal.progress}%`}
                  ></div>
                </div>
                <p class="text-xs text-gray-600">{goal.progress}% complete</p>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
});
