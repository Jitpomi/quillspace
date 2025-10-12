import {component$, Slot, useSignal, useContextProvider} from '@builder.io/qwik';
import {RequestEvent, routeLoader$, Link, routeAction$, Form, useNavigate, useLocation} from '@builder.io/qwik-city';
import type { RequestHandler } from '@builder.io/qwik-city';
import {getTenantInfo, getUserInfo, isAuthenticated, logout} from '~/utils/auth';
import { LuGlobe, LuLogOut, LuMenu, LuUsers, LuSettings, LuHome, LuBarChart3, LuRocket, LuHeart} from "@qwikest/icons/lucide";
import { AuthContextId, type AuthContext } from '~/contexts/auth';
import { useAuth } from '~/hooks/useAuth';

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
        // In development, disable aggressive caching to prevent stale content
        staleWhileRevalidate: env.get('NODE_ENV') === 'production' ? 60 * 60 * 24 * 7 : 0,
        maxAge: env.get('NODE_ENV') === 'production' ? 5 : 0,
    });
    console.log(url)

    const VITE_API_BASE_URL = env.get('VITE_API_BASE_URL');
    console.log('VITE_API_BASE_URL', VITE_API_BASE_URL);
};

// Apply route guard to all HTTP methods
export const onPost: RequestHandler = requireAuthentication;
export const onPut: RequestHandler = requireAuthentication;
export const onPatch: RequestHandler = requireAuthentication;
export const onDelete: RequestHandler = requireAuthentication;

// Route loader to fetch user and tenant data for context
export const useAuthLoader = routeLoader$(async (requestEvent) => {
    const { cookie } = requestEvent;

    const user = await getUserInfo(cookie);
    const tenant = await getTenantInfo(cookie);
    
    return {
        user,
        tenant
    };
});

export const useServerTimeLoader = routeLoader$(() => {
    return {
        date: new Date().toISOString(),
    };
});

// Logout action
export const useLogoutAction = routeAction$(async (data, { cookie, redirect }) => {
    // Clear all auth cookies
    logout(cookie);
    
    // Redirect to login page
    throw redirect(302, '/login');
});

export default component$(() => {
    const authData = useAuthLoader();
    const logoutAction = useLogoutAction();
    const location = useLocation();
    const sidebarOpen = useSignal(false);
    const nav = useNavigate();
    
    // Provide auth context to all child components
    const authContext: AuthContext = {
        user: authData.value.user,
        tenant: authData.value.tenant
    };
    useContextProvider(AuthContextId, authContext);
    // Use the enhanced auth hook for navigation
    const { userPath } = useAuth();

    return (
        <div class="min-h-screen bg-gradient-to-br from-green-50 via-sage-50 to-green-100" style="background-image: url('/feathers.png');">
            {/* Sidebar - Literary sanctuary feel */}
            <div
                class="fixed inset-y-0 left-0 z-50 w-64 bg-white shadow-xl border-r border-gray-300 hidden lg:block">
                {/* Literary Header */}
                <div class="flex cursor-pointer items-center justify-center h-16 border-b border-gray-300 bg-white">
                    <button onClick$={async () => {
                        await nav('/')
                    }} class="flex items-center gap-3">
                        <div class="relative">
                            <LuRocket class="w-7 h-7 text-[#2D3748]"/>
                            <div class="absolute -inset-1 bg-[#9CAF88]/10 rounded-full blur-sm"></div>
                        </div>
                        <span class="text-xl font-semibold text-[#2D3748] tracking-wide">QuillSpace</span>
                    </button>
                </div>

                <nav class="mt-8 px-4">
                    <div class="space-y-2">
                        <button
                            onClick$={async () => {
                                await nav(`${userPath}/`);
                                sidebarOpen.value = false;
                            }}
                            class={`w-full cursor-pointer flex items-center gap-3 px-4 py-3 text-left rounded-xl transition-all duration-300 ${
                                location.url.pathname=== `${userPath}/`
                                    ? 'bg-[#9CAF88]/20 text-[#2D3748] font-medium shadow-sm border border-[#9CAF88]/30'
                                    : 'text-gray-700 hover:bg-[#9CAF88]/10 hover:text-[#2D3748]'
                            }`}
                        >
                            <LuHome class="w-5 h-5"/>
                            <span>Writing Desk</span>
                        </button>

                        <button
                            onClick$={async () => {
                                await nav(`${userPath}/writers`);
                                sidebarOpen.value = false;
                            }}
                            class={`w-full cursor-pointer flex items-center gap-3 px-4 py-3 text-left rounded-xl transition-all duration-200 ${
                                location.url.pathname === `${userPath}/writers` || location.url.pathname === `${userPath}/writers/`
                                    ? 'bg-[#9CAF88]/20 text-[#2D3748] font-medium shadow-sm border border-[#9CAF88]/30'
                                    : 'text-gray-700 hover:bg-[#9CAF88]/10 hover:text-[#2D3748]'
                            }`}
                        >
                            <LuHeart class="w-5 h-5"/>
                            <span>Writers Circle</span>
                        </button>

                        <button
                            onClick$={async () => {
                                await nav(`${userPath}/websites`);
                                sidebarOpen.value = false;
                            }}
                            class={`w-full cursor-pointer flex items-center gap-3 px-4 py-3 text-left rounded-xl transition-all duration-200 ${
                                location.url.pathname.includes('websites')
                                    ? "bg-[#9CAF88]/20 text-[#2D3748] font-medium shadow-sm border border-[#9CAF88]/30"
                                    : "text-gray-700 hover:bg-[#9CAF88]/10 hover:text-[#2D3748]"
                            }`}
                        >
                            <LuGlobe class="w-5 h-5"/>
                            <span>Websites</span>
                        </button>

                        <button
                            onClick$={async () => {
                                await nav(`${userPath}/readers`);
                                sidebarOpen.value = false;
                            }}
                            class={`w-full cursor-pointer flex items-center gap-3 px-4 py-3 text-left rounded-xl transition-all duration-200 ${
                                location.url.pathname.includes('readers')
                                    ? 'bg-[#9CAF88]/20 text-[#2D3748] font-medium shadow-sm border border-[#9CAF88]/30'
                                    : 'text-gray-700 hover:bg-[#9CAF88]/10 hover:text-[#2D3748]'
                            }`}
                        >
                            <LuUsers class="w-5 h-5"/>
                            <span>Readers</span>
                        </button>

                        <button
                            onClick$={async () => {
                                await nav(`${userPath}/settings`);
                                sidebarOpen.value = false;
                            }}
                            class={`w-full cursor-pointer flex items-center gap-3 px-4 py-3 text-left rounded-xl transition-all duration-200 ${
                                location.url.pathname.includes('settings')
                                    ? 'bg-[#9CAF88]/20 text-[#2D3748] font-medium shadow-sm border border-[#9CAF88]/30'
                                    : 'text-gray-700 hover:bg-[#9CAF88]/10 hover:text-[#2D3748]'
                            }`}
                        >
                            <LuSettings class="w-5 h-5"/>
                            <span>Settings</span>
                        </button>
                    </div>
                    
                    {/* Literary bottom section */}
                    <div class="absolute bottom-0 left-0 right-0 p-4 border-t border-gray-300 bg-white">

                        <div class="flex items-center justify-between text-sm">
                            <span class="text-gray-600">{authContext.user ? `${authContext.user.first_name} ${authContext.user.last_name}` : 'User'}</span>
                            <Form action={logoutAction}>
                                <button
                                    type="submit"
                                    class="flex cursor-pointer items-center gap-1 text-gray-500 hover:text-gray-700 transition-colors"
                                    title="Sign out"
                                >
                                    <LuLogOut class="w-4 h-4"/>
                                    <span>Sign out</span>
                                </button>
                            </Form>
                        </div>
                    </div>
                </nav>
            </div>

            {/* Mobile Sidebar - Toggleable */}
            <div
                class={`fixed inset-y-0 left-0 z-50 w-64 bg-white shadow-xl border-r border-gray-300 transform transition-transform duration-300 ease-in-out lg:hidden ${
                    sidebarOpen.value ? 'translate-x-0' : '-translate-x-full'
                }`}>
                <button
                    onClick$={async () => {
                        await nav(`/`);
                    }}
                    class="flex cursor-pointer items-center justify-center h-16 border-b border-gray-300 bg-white">
                    <div class="flex items-center gap-3">
                        <div class="relative">
                            <LuRocket class="w-7 h-7 text-[#2D3748]"/>
                            <div class="absolute -inset-1 bg-[#9CAF88]/10 rounded-full blur-sm"></div>
                        </div>
                        <span class="text-xl font-semibold text-[#2D3748] tracking-wide">QuillSpace</span>
                    </div>
                </button>
                
                {/* Mobile navigation - same as desktop */}
                <nav class="mt-8 px-4 pb-24">
                    <div class="space-y-2">
                        <button
                            onClick$={async () => {
                                await nav(`${userPath}/`);
                                sidebarOpen.value = false;
                            }}
                            class={`w-full cursor-pointer flex items-center gap-3 px-4 py-3 text-left rounded-xl transition-all duration-200 ${
                                location.url.pathname === `${userPath}/`
                                    ? 'bg-[#9CAF88]/20 text-[#2D3748] font-medium shadow-sm border border-[#9CAF88]/30'
                                    : 'text-gray-700 hover:bg-[#9CAF88]/10 hover:text-[#2D3748]'
                            }`}
                        >
                            <LuBarChart3 class="w-5 h-5"/>
                            <span>Dashboard</span>
                        </button>

                        <button
                            onClick$={async () => {
                                await nav(`${userPath}/writers`);
                                sidebarOpen.value = false;
                            }}
                            class={`w-full cursor-pointer flex items-center gap-3 px-4 py-3 text-left rounded-xl transition-all duration-200 ${
                                location.url.pathname === `${userPath}/writers` || location.url.pathname === `${userPath}/writers/`
                                    ? 'bg-[#9CAF88]/20 text-[#2D3748] font-medium shadow-sm border border-[#9CAF88]/30'
                                    : 'text-gray-700 hover:bg-[#9CAF88]/10 hover:text-[#2D3748]'
                            }`}
                        >
                            <LuHeart class="w-5 h-5"/>
                            <span>Writers Circle</span>
                        </button>

                        <button
                            onClick$={async () => {
                                await nav(`${userPath}/websites`);
                                sidebarOpen.value = false;
                            }}
                            class={`w-full cursor-pointer flex items-center gap-3 px-4 py-3 text-left rounded-xl transition-all duration-200 ${
                                location.url.pathname.includes('websites')
                                    ? "bg-[#9CAF88]/20 text-[#2D3748] font-medium shadow-sm border border-[#9CAF88]/30"
                                    : "text-gray-700 hover:bg-[#9CAF88]/10 hover:text-[#2D3748]"
                            }`}
                        >
                            <LuGlobe class="w-5 h-5"/>
                            <span>My Websites</span>
                        </button>

                        <button
                            onClick$={async () => {
                                await nav(`${userPath}/readers`);
                                sidebarOpen.value = false;
                            }}
                            class={`w-full cursor-pointer flex items-center gap-3 px-4 py-3 text-left rounded-xl transition-all duration-200 ${
                                location.url.pathname.includes('readers')
                                    ? 'bg-[#9CAF88]/20 text-[#2D3748] font-medium shadow-sm border border-[#9CAF88]/30'
                                    : 'text-gray-700 hover:bg-[#9CAF88]/10 hover:text-[#2D3748]'
                            }`}
                        >
                            <LuUsers class="w-5 h-5"/>
                            <span>Readers</span>
                        </button>

                        <button
                            onClick$={async () => {
                                await nav(`${userPath}/settings`);
                                sidebarOpen.value = false;
                            }}
                            class={`w-full cursor-pointer flex items-center gap-3 px-4 py-3 text-left rounded-xl transition-all duration-200 ${
                                location.url.pathname.includes('settings')
                                    ? 'bg-[#9CAF88]/20 text-[#2D3748] font-medium shadow-sm border border-[#9CAF88]/30'
                                    : 'text-gray-700 hover:bg-[#9CAF88]/10 hover:text-[#2D3748]'
                            }`}
                        >
                            <LuSettings class="w-5 h-5"/>
                            <span>Settings</span>
                        </button>
                    </div>
                    
                    {/* Mobile clean bottom section */}
                    <div class="absolute bottom-0 left-0 right-0 p-4 border-t border-gray-300 bg-white">
                        <div class="flex items-center justify-between text-sm">
                            <span class="text-gray-600">{authContext.user ? `${authContext.user.first_name} ${authContext.user.last_name}` : 'User'}</span>
                            <Form action={logoutAction}>
                                <button
                                    type="submit"
                                    class="flex cursor-pointer items-center gap-1 text-gray-500 hover:text-gray-700 transition-colors"
                                    title="Sign out"
                                >
                                    <LuLogOut class="w-4 h-4"/>
                                    <span>Sign out</span>
                                </button>
                            </Form>
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
