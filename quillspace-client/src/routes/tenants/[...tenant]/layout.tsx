import { component$, Slot } from '@builder.io/qwik';
import {RequestEvent, routeLoader$} from '@builder.io/qwik-city';
import type { RequestHandler } from '@builder.io/qwik-city';
import { isAuthenticated } from '~/utils/auth';

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
    return (
        <>
            <Slot />
        </>
    );
});
