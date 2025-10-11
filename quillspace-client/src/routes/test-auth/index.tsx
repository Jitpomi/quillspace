import { component$ } from "@builder.io/qwik";
import { routeAction$, Form } from "@builder.io/qwik-city";
import { setAuthToken, clearAuthToken, isAuthenticated } from "~/utils/auth";

// Action to simulate login (set auth token)
export const useLoginAction = routeAction$(async (data, { cookie, redirect }) => {
  // Set a dummy JWT token for testing
  const dummyToken = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3OC05MGFiLWNkZWYtMTIzNC01Njc4OTBhYmNkZWYiLCJlbWFpbCI6InRlc3RAdGVzdC5jb20iLCJuYW1lIjoiVGVzdCBVc2VyIiwicm9sZSI6ImFkbWluIiwidGVuYW50X2lkIjoiMTExMTExMTEtMTExMS0xMTExLTExMTEtMTExMTExMTExMTExIiwiZXhwIjoxNzM1NjgwMDAwLCJpYXQiOjE3MjgwODAwMDAsImlzcyI6InF1aWxsc3BhY2UifQ.dummy-signature";
  
  // Set auth token
  setAuthToken(cookie, dummyToken);
  
  // Set user info cookie (matching login form structure)
  const dummyUser = {
    id: "12345678-90ab-cdef-1234-567890abcdef",
    email: "test@test.com",
    first_name: "Test",
    last_name: "User",
    role: "admin"
  };
  
  cookie.set('user_info', JSON.stringify(dummyUser), {
    httpOnly: false,
    secure: false,
    sameSite: 'lax',
    maxAge: 60 * 60 * 24 * 7,
    path: '/',
  });
  
  // Set tenant info cookie (matching login form structure)
  const dummyTenant = {
    id: "11111111-1111-1111-1111-111111111111",
    name: "Test Tenant",
    slug: "test-tenant"
  };
  
  cookie.set('tenant_info', JSON.stringify(dummyTenant), {
    httpOnly: false,
    secure: false,
    sameSite: 'lax',
    maxAge: 60 * 60 * 24 * 7,
    path: '/',
  });
  
  return { success: true };
});

// Action to simulate logout (clear auth token)
export const useLogoutAction = routeAction$(async (data, { cookie }) => {
  clearAuthToken(cookie);
  cookie.delete('user_info', { path: '/' });
  cookie.delete('tenant_info', { path: '/' });
  return { success: true };
});

export default component$(() => {
  const loginAction = useLoginAction();
  const logoutAction = useLogoutAction();

  return (
    <div class="min-h-screen bg-gray-100 py-12 px-4">
      <div class="max-w-md mx-auto bg-white rounded-lg shadow-lg p-8">
        <h1 class="text-2xl font-bold mb-6 text-center">Auth Testing</h1>
        
        <div class="space-y-4">
          <div class="text-center">
            <p class="text-sm text-gray-600 mb-4">
              Use this page to test authentication and route guards.
            </p>
          </div>

          <Form action={loginAction} class="space-y-4">
            <button 
              type="submit"
              class="w-full bg-blue-600 text-white py-2 px-4 rounded hover:bg-blue-700 transition-colors"
            >
              Simulate Login (Set Auth Token)
            </button>
          </Form>

          <Form action={logoutAction} class="space-y-4">
            <button 
              type="submit"
              class="w-full bg-red-600 text-white py-2 px-4 rounded hover:bg-red-700 transition-colors"
            >
              Simulate Logout (Clear Auth Token)
            </button>
          </Form>

          <div class="mt-6 p-4 bg-gray-50 rounded">
            <h3 class="font-semibold mb-2">Testing Instructions:</h3>
            <ol class="text-sm text-gray-700 space-y-1 list-decimal list-inside">
              <li>Click "Simulate Login" to set auth token + user/tenant info</li>
              <li>Try to visit <a href="/login" class="text-blue-600 hover:underline">/login</a></li>
              <li>You should be redirected to <code class="bg-gray-200 px-1 rounded">/tenants/test-tenant/users/12345678-90ab-cdef-1234-567890abcdef</code></li>
              <li>Click "Simulate Logout" to clear all auth cookies</li>
              <li>Now you should be able to access <a href="/login" class="text-blue-600 hover:underline">/login</a> normally</li>
            </ol>
          </div>

          {loginAction.value?.success && (
            <div class="p-3 bg-green-100 text-green-700 rounded">
              ✅ Auth cookies set! Try visiting /login - you'll be redirected to /tenants/test-tenant/users/12345678-90ab-cdef-1234-567890abcdef
            </div>
          )}

          {logoutAction.value?.success && (
            <div class="p-3 bg-blue-100 text-blue-700 rounded">
              ✅ Auth token cleared! You can now access /login.
            </div>
          )}
        </div>
      </div>
    </div>
  );
});
