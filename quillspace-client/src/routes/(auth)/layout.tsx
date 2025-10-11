import { component$, Slot } from '@builder.io/qwik';
import type { RequestHandler } from '@builder.io/qwik-city';
import { isAuthenticated, getAuthenticatedUserPath } from '~/utils/auth';

// Route guard function to check authentication and redirect if needed
const checkAuthAndRedirect: RequestHandler = async ({ cookie, redirect }) => {
  if (isAuthenticated(cookie)) {
    const redirectPath = getAuthenticatedUserPath(cookie);
    throw redirect(302, redirectPath);
  }
};

export const onGet: RequestHandler = async ({ cacheControl, cookie, redirect }) => {
  // Route guard: Redirect authenticated users away from auth pages
  await checkAuthAndRedirect({ cookie, redirect } as any);

  // Control caching for this request for best performance and to reduce hosting costs:
  // https://qwik.builder.io/docs/caching/
  cacheControl({
    // Always serve a cached response by default, up to a week stale
    staleWhileRevalidate: 60 * 60 * 24 * 7,
    // Max once every 5 seconds, revalidate on the server to get a fresh version of this page
    maxAge: 5,
  });
};

// Apply route guard to all HTTP methods
export const onPost: RequestHandler = checkAuthAndRedirect;
export const onPut: RequestHandler = checkAuthAndRedirect;
export const onPatch: RequestHandler = checkAuthAndRedirect;
export const onDelete: RequestHandler = checkAuthAndRedirect;

export default component$(() => {
  return (
    <>
      <Slot />
    </>
  );
});
