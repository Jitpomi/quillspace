import {component$, Slot, useSignal} from '@builder.io/qwik';
import {RequestEvent, routeLoader$} from '@builder.io/qwik-city';
import type { RequestHandler } from '@builder.io/qwik-city';
import { isAuthenticated } from '~/utils/auth';
import {LuFeather, LuBookOpen, LuGlobe, LuLogOut, LuMenu, LuUsers, LuSettings, LuPenTool, LuHome, LuBarChart3, LuFileText, LuRocket, LuHeart} from "@qwikest/icons/lucide";

// Route guard function to check authentication and redirect if needed
const requireAuthentication: RequestHandler = async ({ cookie, redirect }) => {
  if (!isAuthenticated(cookie)) {
    throw redirect(302, '/login');
  }
};

export const onGet: RequestHandler = async ( requestEvent: RequestEvent) => {
   const { cacheControl ,  env, url, cookie, redirect} = requestEvent;
   
   // Route guard: Require authentication for all tenant routes
   await requireAuthentication({ cookie, redirect } as any);
    // Control caching for this request for best performance and to reduce hosting costs:
    // https://qwik.builder.io/docs/caching/
    cacheControl({
        // Always serve a cached response by default, up to a week stale
        staleWhileRevalidate: 60 * 60 * 24 * 7,
        // Max once every 5 seconds, revalidate on the server to get a fresh version of this page
        maxAge: 5,
    });
    console.log(url)

    const VITE_API_BASE_URL = env.get('VITE_API_BASE_URL');
    console.log('VITE_API_BASE_URL', VITE_API_BASE_URL);

    // json(200, { hello: 'world' });
};

// Apply route guard to all HTTP methods
export const onPost: RequestHandler = requireAuthentication;
export const onPut: RequestHandler = requireAuthentication;
export const onPatch: RequestHandler = requireAuthentication;
export const onDelete: RequestHandler = requireAuthentication;

export const useServerTimeLoader = routeLoader$(() => {
    return {
        date: new Date().toISOString(),
    };
});

export default component$(() => {
    const currentUser = useSignal({name: 'test'});
    const pageTitle = useSignal('dashboard');
    const sidebarOpen = useSignal(false);
    return (
        <div class="min-h-screen bg-gradient-to-br from-green-50 via-sage-50 to-green-100" style="background-image: url('https://www.transparenttextures.com/patterns/subtle-white-feathers.png');">
            {/* Sidebar - Literary sanctuary feel */}
            <div
                class="fixed inset-y-0 left-0 z-50 w-64 bg-gradient-to-b from-slate-800 via-slate-700 to-slate-800 shadow-2xl border-r border-[#9CAF88]/30 hidden lg:block">
                {/* Literary Header */}
                <div class="flex items-center justify-center h-16 border-b border-[#9CAF88]/30 bg-slate-800">
                    <div class="flex items-center gap-3">
                        <div class="relative">
                            <LuFeather class="w-8 h-8 text-[#9CAF88] drop-shadow-lg"/>
                            <div class="absolute -inset-2 bg-[#9CAF88]/20 rounded-full blur-md"></div>
                        </div>
                        <span class="text-xl font-bold text-white tracking-wide">Your Sanctuary</span>
                    </div>
                </div>

                <nav class="mt-8 px-4">
                    <div class="space-y-2">
                        <button
                            onClick$={() => {
                                sidebarOpen.value = false;
                            }}
                            class={`w-full flex items-center gap-3 px-4 py-3 text-left rounded-xl transition-all duration-300 ${
                                pageTitle.value === 'dashboard'
                                    ? 'bg-[#9CAF88]/20 text-white font-medium shadow-lg border border-[#9CAF88]/40'
                                    : 'text-gray-300 hover:bg-[#9CAF88]/10 hover:text-white'
                            }`}
                        >
                            <LuHome class="w-5 h-5"/>
                            <span>Writing Desk</span>
                        </button>

                        <button
                            onClick$={() => {
                                sidebarOpen.value = false;
                            }}
                            class={`w-full flex items-center gap-3 px-4 py-3 text-left rounded-xl transition-all duration-200 ${
                                pageTitle.value === 'writers-circle'
                                    ? 'bg-[#9CAF88]/20 text-white font-medium shadow-sm border border-[#9CAF88]/30'
                                    : 'text-gray-300 hover:bg-[#9CAF88]/10 hover:text-white'
                            }`}
                        >
                            <LuHeart class="w-5 h-5"/>
                            <span>Writers Circle</span>
                        </button>

                        <button
                            onClick$={() => {
                                sidebarOpen.value = false;
                            }}
                            class={`w-full flex items-center gap-3 px-4 py-3 text-left rounded-xl transition-all duration-200 ${
                                pageTitle.value === "website-builder"
                                    ? "bg-[#9CAF88]/20 text-white font-medium shadow-sm border border-[#9CAF88]/30"
                                    : "text-gray-300 hover:bg-[#9CAF88]/10 hover:text-white"
                            }`}
                        >
                            <LuGlobe class="w-5 h-5"/>
                            <span>My Website</span>
                        </button>

                        <button
                            onClick$={() => {
                                sidebarOpen.value = false;
                            }}
                            class={`w-full flex items-center gap-3 px-4 py-3 text-left rounded-xl transition-all duration-200 ${
                                pageTitle.value === 'users'
                                    ? 'bg-[#9CAF88]/20 text-white font-medium shadow-sm border border-[#9CAF88]/30'
                                    : 'text-gray-300 hover:bg-[#9CAF88]/10 hover:text-white'
                            }`}
                        >
                            <LuUsers class="w-5 h-5"/>
                            <span>Readers</span>
                        </button>

                        <button
                            onClick$={() => {
                                sidebarOpen.value = false;
                            }}
                            class={`w-full flex items-center gap-3 px-4 py-3 text-left rounded-xl transition-all duration-200 ${
                                pageTitle.value === 'settings'
                                    ? 'bg-[#9CAF88]/20 text-white font-medium shadow-sm border border-[#9CAF88]/30'
                                    : 'text-gray-300 hover:bg-[#9CAF88]/10 hover:text-white'
                            }`}
                        >
                            <LuSettings class="w-5 h-5"/>
                            <span>Settings</span>
                        </button>
                    </div>
                    
                    {/* Literary bottom section */}
                    <div class="absolute bottom-0 left-0 right-0 p-4 border-t border-[#9CAF88]/30 bg-slate-800">
                        <button
                            class="w-full bg-[#9CAF88] hover:bg-[#8a9e7a] text-white px-4 py-2.5 rounded-lg font-medium transition-all duration-300 shadow-lg hover:shadow-xl mb-3">
                            Begin Writing
                        </button>
                        <div class="flex items-center justify-between text-sm">
                            <span class="text-gray-300">{currentUser.value?.name}</span>
                            <button
                                // onClick$={handleLogout}
                                class="flex items-center gap-1 text-[#9CAF88] hover:text-white transition-colors"
                                title="Leave sanctuary"
                            >
                                <LuLogOut class="w-4 h-4"/>
                                <span>Leave</span>
                            </button>
                        </div>
                    </div>
                </nav>
            </div>

            {/* Mobile Sidebar - Toggleable */}
            <div
                class={`fixed inset-y-0 left-0 z-50 w-64 bg-white/95 backdrop-blur-md shadow-xl border-r border-[#E8E2D4] transform transition-transform duration-300 ease-in-out lg:hidden ${
                    sidebarOpen.value ? 'translate-x-0' : '-translate-x-full'
                }`} style="background-image: url('https://www.transparenttextures.com/patterns/subtle-white-feathers.png'); background-blend-mode: overlay;">
                <div class="flex items-center justify-center h-16 border-b border-[#E8E2D4] bg-white/80">
                    <div class="flex items-center gap-3">
                        <div class="relative">
                            <LuRocket class="w-7 h-7 text-[#2D3748]"/>
                            <div class="absolute -inset-1 bg-[#9CAF88]/10 rounded-full blur-sm"></div>
                        </div>
                        <span class="text-xl font-semibold text-[#2D3748] tracking-wide">QuillSpace</span>
                    </div>
                </div>
                
                {/* Mobile navigation - same as desktop */}
                <nav class="mt-8 px-4 pb-24">
                    <div class="space-y-2">
                        <button
                            onClick$={() => {
                                sidebarOpen.value = false;
                            }}
                            class={`w-full flex items-center gap-3 px-4 py-3 text-left rounded-xl transition-all duration-200 ${
                                pageTitle.value === 'dashboard'
                                    ? 'bg-[#9CAF88]/20 text-white font-medium shadow-sm border border-[#9CAF88]/30'
                                    : 'text-gray-300 hover:bg-[#9CAF88]/10 hover:text-white'
                            }`}
                        >
                            <LuBarChart3 class="w-5 h-5"/>
                            <span>Dashboard</span>
                        </button>

                        <button
                            onClick$={() => {
                                sidebarOpen.value = false;
                            }}
                            class={`w-full flex items-center gap-3 px-4 py-3 text-left rounded-xl transition-all duration-200 ${
                                pageTitle.value === 'writers-circle'
                                    ? 'bg-[#9CAF88]/20 text-white font-medium shadow-sm border border-[#9CAF88]/30'
                                    : 'text-gray-300 hover:bg-[#9CAF88]/10 hover:text-white'
                            }`}
                        >
                            <LuHeart class="w-5 h-5"/>
                            <span>Writers Circle</span>
                        </button>

                        <button
                            onClick$={() => {
                                sidebarOpen.value = false;
                            }}
                            class={`w-full flex items-center gap-3 px-4 py-3 text-left rounded-xl transition-all duration-200 ${
                                pageTitle.value === "website-builder"
                                    ? "bg-[#9CAF88]/20 text-white font-medium shadow-sm border border-[#9CAF88]/30"
                                    : "text-gray-300 hover:bg-[#9CAF88]/10 hover:text-white"
                            }`}
                        >
                            <LuGlobe class="w-5 h-5"/>
                            <span>My Website</span>
                        </button>

                        <button
                            onClick$={() => {
                                sidebarOpen.value = false;
                            }}
                            class={`w-full flex items-center gap-3 px-4 py-3 text-left rounded-xl transition-all duration-200 ${
                                pageTitle.value === 'users'
                                    ? 'bg-[#9CAF88]/20 text-white font-medium shadow-sm border border-[#9CAF88]/30'
                                    : 'text-gray-300 hover:bg-[#9CAF88]/10 hover:text-white'
                            }`}
                        >
                            <LuUsers class="w-5 h-5"/>
                            <span>Readers</span>
                        </button>

                        <button
                            onClick$={() => {
                                sidebarOpen.value = false;
                            }}
                            class={`w-full flex items-center gap-3 px-4 py-3 text-left rounded-xl transition-all duration-200 ${
                                pageTitle.value === 'settings'
                                    ? 'bg-[#9CAF88]/20 text-white font-medium shadow-sm border border-[#9CAF88]/30'
                                    : 'text-gray-300 hover:bg-[#9CAF88]/10 hover:text-white'
                            }`}
                        >
                            <LuSettings class="w-5 h-5"/>
                            <span>Settings</span>
                        </button>
                    </div>
                    
                    {/* Mobile clean bottom section */}
                    <div class="absolute bottom-0 left-0 right-0 p-4 border-t border-[#E8E2D4] bg-white/90">
                        <button
                            class="w-full bg-[#9CAF88] hover:bg-[#8a9e7a] text-white px-4 py-2.5 rounded-lg font-medium transition-colors mb-3">
                            Create
                        </button>
                        <div class="flex items-center justify-between text-sm">
                            <span class="text-gray-600">{currentUser.value?.name}</span>
                            <button
                                // onClick$={handleLogout}
                                class="flex items-center gap-1 text-gray-500 hover:text-gray-700 transition-colors"
                                title="Sign out"
                            >
                                <LuLogOut class="w-4 h-4"/>
                                <span>Sign out</span>
                            </button>
                        </div>
                    </div>
                </nav>

            </div>

            {/* Main Content */}
            <div class="lg:pl-64">
                {/* Mobile menu button - only visible on mobile */}
                <div class="lg:hidden fixed top-4 left-4 z-40">
                    <button
                        onClick$={() => sidebarOpen.value = !sidebarOpen.value}
                        class="p-2 rounded-lg text-gray-600 hover:text-[#2D3748] hover:bg-white/80 backdrop-blur-sm shadow-sm transition-all duration-200"
                    >
                        <LuMenu class="w-5 h-5"/>
                    </button>
                </div>

                {/* Page Content - Clean and spacious */}
                <main class="p-8 min-h-screen">
                    <div class="max-w-6xl mx-auto">
                        <Slot/>
                    </div>
                </main>
            </div>
        </div>
    );
});
