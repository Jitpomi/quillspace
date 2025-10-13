import {component$, Slot, useSignal, useContextProvider} from '@builder.io/qwik';
import {RequestEvent, routeLoader$,  routeAction$, Form, useNavigate, useLocation} from '@builder.io/qwik-city';
import type { RequestHandler } from '@builder.io/qwik-city';
import {getTenantInfo, getUserInfo, isAuthenticated, logout} from '~/utils/auth';
import {
    LuGlobe,
    LuLogOut,
    LuMenu,
    LuUsers,
    LuSettings,
    LuHome,
    LuBarChart3,
    LuRocket,
    LuHeart,
    LuFeather,
    LuChevronLeft,
    LuChevronRight,
    LuPanelLeft,
    LuPanelLeftClose
} from "@qwikest/icons/lucide";
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
    const sidebarCollapsed = useSignal(false);
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
            <style>{`
                @keyframes fadeIn {
                    from { opacity: 0; }
                    to { opacity: 1; }
                }
            `}</style>
            {/* Sidebar - Literary sanctuary feel */}
            <div
                class={`fixed inset-y-0 left-0 z-50 bg-white shadow-xl border-r border-gray-300 hidden lg:block transition-all duration-300 ${
                    sidebarCollapsed.value ? 'w-16' : 'w-64'
                }`}>
                {/* Literary Header */}
                <div class="flex items-center h-16 border-b border-gray-300 bg-white px-4 relative">
                    <button onClick$={async () => {
                        await nav('/')
                    }} class="flex items-center gap-3 flex-1">
                        <div class="relative">
                            <LuFeather class="w-8 h-8 text-[#9CAF88] drop-shadow-lg" />
                            <div class="absolute -inset-1 bg-[#9CAF88]/10 rounded-full blur-sm"></div>
                        </div>
                        {!sidebarCollapsed.value && (
                            <span class="text-xl font-semibold text-[#2D3748] tracking-wide opacity-0 animate-[fadeIn_0.3s_ease-in-out_0.1s_forwards]">QuillSpace</span>
                        )}
                    </button>
                </div>

                {/* Sidebar Toggle Button - Professional panel controls */}
                <button
                    onClick$={() => sidebarCollapsed.value = !sidebarCollapsed.value}
                    class="absolute top-4 -right-6 z-60 w-10 h-10 bg-white border border-gray-200 rounded-full shadow-md hover:shadow-lg transition-all duration-200 flex items-center justify-center group hover:bg-gray-50 hover:border-gray-300 cursor-pointer"
                    title={sidebarCollapsed.value ? 'Show sidebar' : 'Hide sidebar'}
                >
                    {sidebarCollapsed.value ? (
                        <LuPanelLeft class="w-5 h-5 text-gray-600 group-hover:text-gray-800" />
                    ) : (
                        <LuPanelLeftClose class="w-5 h-5 text-gray-600 group-hover:text-gray-800" />
                    )}
                </button>

                <nav class={`mt-8 ${sidebarCollapsed.value ? 'px-2' : 'px-4'}`}>
                    <div class="space-y-2">
                        <button
                            onClick$={async () => {
                                await nav(`${userPath}/`);
                                sidebarOpen.value = false;
                            }}
                            class={`w-full cursor-pointer flex items-center rounded-xl transition-all duration-300 ${
                                location.url.pathname=== `${userPath}/`
                                    ? 'bg-[#9CAF88]/20 text-[#2D3748] font-medium shadow-sm border border-[#9CAF88]/30'
                                    : 'text-gray-700 hover:bg-[#9CAF88]/10 hover:text-[#2D3748]'
                            } ${sidebarCollapsed.value ? 'justify-center px-2 py-3' : 'gap-3 px-4 py-3'}`}
                            title={sidebarCollapsed.value ? 'Writing Desk' : ''}
                        >
                            <LuHome class="w-5 h-5"/>
                            {!sidebarCollapsed.value && (
                                <span class="opacity-0 animate-[fadeIn_0.3s_ease-in-out_0.1s_forwards]">Writing Desk</span>
                            )}
                        </button>

                        <button
                            onClick$={async () => {
                                await nav(`${userPath}/writers`);
                                sidebarOpen.value = false;
                            }}
                            class={`w-full cursor-pointer flex items-center rounded-xl transition-all duration-200 ${
                                location.url.pathname === `${userPath}/writers` || location.url.pathname === `${userPath}/writers/`
                                    ? 'bg-[#9CAF88]/20 text-[#2D3748] font-medium shadow-sm border border-[#9CAF88]/30'
                                    : 'text-gray-700 hover:bg-[#9CAF88]/10 hover:text-[#2D3748]'
                            } ${sidebarCollapsed.value ? 'justify-center px-2 py-3' : 'gap-3 px-4 py-3'}`}
                            title={sidebarCollapsed.value ? 'Writers Circle' : ''}
                        >
                            <LuHeart class="w-5 h-5"/>
                            {!sidebarCollapsed.value && (
                                <span class="opacity-0 animate-[fadeIn_0.3s_ease-in-out_0.15s_forwards]">Writers Circle</span>
                            )}
                        </button>

                        <button
                            onClick$={async () => {
                                await nav(`${userPath}/websites`);
                                sidebarOpen.value = false;
                            }}
                            class={`w-full cursor-pointer flex items-center rounded-xl transition-all duration-200 ${
                                location.url.pathname.includes('websites')
                                    ? "bg-[#9CAF88]/20 text-[#2D3748] font-medium shadow-sm border border-[#9CAF88]/30"
                                    : "text-gray-700 hover:bg-[#9CAF88]/10 hover:text-[#2D3748]"
                            } ${sidebarCollapsed.value ? 'justify-center px-2 py-3' : 'gap-3 px-4 py-3'}`}
                            title={sidebarCollapsed.value ? 'Websites' : ''}
                        >
                            <LuGlobe class="w-5 h-5"/>
                            {!sidebarCollapsed.value && (
                                <span class="opacity-0 animate-[fadeIn_0.3s_ease-in-out_0.2s_forwards]">Websites</span>
                            )}
                        </button>

                        <button
                            onClick$={async () => {
                                await nav(`${userPath}/readers`);
                                sidebarOpen.value = false;
                            }}
                            class={`w-full cursor-pointer flex items-center rounded-xl transition-all duration-200 ${
                                location.url.pathname.includes('readers')
                                    ? 'bg-[#9CAF88]/20 text-[#2D3748] font-medium shadow-sm border border-[#9CAF88]/30'
                                    : 'text-gray-700 hover:bg-[#9CAF88]/10 hover:text-[#2D3748]'
                            } ${sidebarCollapsed.value ? 'justify-center px-2 py-3' : 'gap-3 px-4 py-3'}`}
                            title={sidebarCollapsed.value ? 'Readers' : ''}
                        >
                            <LuUsers class="w-5 h-5"/>
                            {!sidebarCollapsed.value && (
                                <span class="opacity-0 animate-[fadeIn_0.3s_ease-in-out_0.25s_forwards]">Readers</span>
                            )}
                        </button>

                        <button
                            onClick$={async () => {
                                await nav(`${userPath}/settings`);
                                sidebarOpen.value = false;
                            }}
                            class={`w-full cursor-pointer flex items-center rounded-xl transition-all duration-200 ${
                                location.url.pathname.includes('settings')
                                    ? 'bg-[#9CAF88]/20 text-[#2D3748] font-medium shadow-sm border border-[#9CAF88]/30'
                                    : 'text-gray-700 hover:bg-[#9CAF88]/10 hover:text-[#2D3748]'
                            } ${sidebarCollapsed.value ? 'justify-center px-2 py-3' : 'gap-3 px-4 py-3'}`}
                            title={sidebarCollapsed.value ? 'Settings' : ''}
                        >
                            <LuSettings class="w-5 h-5"/>
                            {!sidebarCollapsed.value && (
                                <span class="opacity-0 animate-[fadeIn_0.3s_ease-in-out_0.3s_forwards]">Settings</span>
                            )}
                        </button>
                    </div>
                    
                    {/* Literary bottom section */}
                    <div class="absolute bottom-0 left-0 right-0 p-4 border-t border-gray-300 bg-white">
                        {sidebarCollapsed.value ? (
                            <div class="flex flex-col items-center gap-2">
                                <div class="w-8 h-8 bg-[#9CAF88]/20 rounded-full flex items-center justify-center">
                                    <span class="text-sm font-medium text-[#2D3748]">
                                        {authContext.user ? authContext.user.first_name.charAt(0).toUpperCase() : 'U'}
                                    </span>
                                </div>
                                <Form action={logoutAction}>
                                    <button
                                        type="submit"
                                        class="p-1 text-gray-500 hover:text-gray-700 transition-colors cursor-pointer"
                                        title="Sign out"
                                    >
                                        <LuLogOut class="w-4 h-4"/>
                                    </button>
                                </Form>
                            </div>
                        ) : (
                            <div class="flex items-center justify-between text-sm">
                                <span class="text-gray-600 opacity-0 animate-[fadeIn_0.3s_ease-in-out_0.1s_forwards]">{authContext.user ? `${authContext.user.first_name} ${authContext.user.last_name}` : 'User'}</span>
                                <Form action={logoutAction}>
                                    <button
                                        type="submit"
                                        class="flex cursor-pointer items-center gap-1 text-gray-500 hover:text-gray-700 transition-colors"
                                        title="Sign out"
                                    >
                                        <LuLogOut class="w-4 h-4"/>
                                        <span class="opacity-0 animate-[fadeIn_0.3s_ease-in-out_0.15s_forwards]">Sign out</span>
                                    </button>
                                </Form>
                            </div>
                        )}
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
            <div class={`transition-all duration-300 ${
                sidebarCollapsed.value ? 'lg:pl-16' : 'lg:pl-64'
            }`}>
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
