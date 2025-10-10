import { component$, Slot } from '@builder.io/qwik';
import {RequestEvent, routeLoader$} from '@builder.io/qwik-city';
import type { RequestHandler } from '@builder.io/qwik-city';

export const onGet: RequestHandler = async ( requestEvent: RequestEvent) => {
   const { cacheControl ,  env, url} = requestEvent;
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
