/**
 * Login Component for QuillSpace
 * Handles user authentication with the backend
 */

import { component$, useSignal, $ } from '@builder.io/qwik';
import { LuRocket, LuMail, LuLock, LuEye, LuEyeOff } from '@qwikest/icons/lucide';
import { api, setAuthToken, setTenantId } from '../../services/api';

export default component$(() => {
  // Form state
  const email = useSignal('');
  const password = useSignal('');
  const showPassword = useSignal(false);
  const isSubmitting = useSignal(false);
  const error = useSignal<string | null>(null);

  // Handle form submission
  const handleSubmit = $(async (event: Event) => {
    event.preventDefault();
    
    if (!email.value || !password.value) {
      error.value = 'Please enter both email and password';
      return;
    }

    isSubmitting.value = true;
    error.value = null;
    
    try {
      const response = await api.login(email.value, password.value);
      
      // Set tokens
      setAuthToken(response.token);
      setTenantId(response.tenant.id);
      
      // Reload the page to trigger authentication check
      window.location.reload();
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Login failed';
    } finally {
      isSubmitting.value = false;
    }
  });

  const togglePasswordVisibility = $(() => {
    showPassword.value = !showPassword.value;
  });

  return (
    <div class="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 flex items-center justify-center p-4">
      <div class="max-w-md w-full space-y-8">
        {/* Header */}
        <div class="text-center">
          <div class="flex justify-center">
            <LuRocket class="w-12 h-12 text-blue-600" />
          </div>
          <h2 class="mt-6 text-3xl font-bold text-gray-900">
            Welcome to QuillSpace
          </h2>
          <p class="mt-2 text-sm text-gray-600">
            Sign in to your multi-tenant CMS
          </p>
        </div>

        {/* Login Form */}
        <div class="bg-white rounded-xl shadow-lg p-8">
          <form onSubmit$={handleSubmit} class="space-y-6">
            {/* Error Message */}
            {error.value && (
              <div class="bg-red-50 border border-red-200 rounded-lg p-4">
                <div class="flex">
                  <div class="ml-3">
                    <h3 class="text-sm font-medium text-red-800">
                      Authentication Error
                    </h3>
                    <div class="mt-2 text-sm text-red-700">
                      {error.value}
                    </div>
                  </div>
                </div>
              </div>
            )}

            {/* Email Field */}
            <div>
              <label for="email" class="block text-sm font-medium text-gray-700">
                Email address
              </label>
              <div class="mt-1 relative">
                <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                  <LuMail class="h-5 w-5 text-gray-400" />
                </div>
                <input
                  id="email"
                  name="email"
                  type="email"
                  autoComplete="email"
                  required
                  value={email.value}
                  onInput$={(e) => email.value = (e.target as HTMLInputElement).value}
                  class="block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  placeholder="Enter your email"
                />
              </div>
            </div>

            {/* Password Field */}
            <div>
              <label for="password" class="block text-sm font-medium text-gray-700">
                Password
              </label>
              <div class="mt-1 relative">
                <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                  <LuLock class="h-5 w-5 text-gray-400" />
                </div>
                <input
                  id="password"
                  name="password"
                  type={showPassword.value ? 'text' : 'password'}
                  autoComplete="current-password"
                  required
                  value={password.value}
                  onInput$={(e) => password.value = (e.target as HTMLInputElement).value}
                  class="block w-full pl-10 pr-10 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  placeholder="Enter your password"
                />
                <button
                  type="button"
                  onClick$={togglePasswordVisibility}
                  class="absolute inset-y-0 right-0 pr-3 flex items-center"
                >
                  {showPassword.value ? (
                    <LuEyeOff class="h-5 w-5 text-gray-400 hover:text-gray-600" />
                  ) : (
                    <LuEye class="h-5 w-5 text-gray-400 hover:text-gray-600" />
                  )}
                </button>
              </div>
            </div>

            {/* Submit Button */}
            <div>
              <button
                type="submit"
                disabled={isSubmitting.value}
                class="w-full flex justify-center py-2 px-4 border border-transparent rounded-lg shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isSubmitting.value ? (
                  <div class="flex items-center">
                    <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
                    Signing in...
                  </div>
                ) : (
                  'Sign in'
                )}
              </button>
            </div>
          </form>

          {/* Demo Credentials */}
          <div class="mt-6 p-4 bg-gray-50 rounded-lg">
            <h4 class="text-sm font-medium text-gray-700 mb-2">Demo Credentials:</h4>
            <div class="text-xs text-gray-600 space-y-1">
              <div>Email: admin@demo.com</div>
              <div>Password: admin123</div>
            </div>
          </div>
        </div>

        {/* Footer */}
        <div class="text-center">
          <p class="text-sm text-gray-600">
            Don't have an account?{' '}
            <a href="#" class="font-medium text-blue-600 hover:text-blue-500">
              Contact your administrator
            </a>
          </p>
        </div>
      </div>
    </div>
  );
});
