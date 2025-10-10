import { component$ } from "@builder.io/qwik";
import {routeAction$} from "@builder.io/qwik-city";
import {LoginResponse} from "~/api/schema";
import {LuRocket} from "@qwikest/icons/lucide";

export const useLogin = routeAction$(async (loginJson, requestEventAction): Promise<LoginResponse> => {
    const { env} = requestEventAction;
    const API_BASE_URL = env.get('VITE_API_BASE_URL');
    const response = await fetch(`${API_BASE_URL}/auth/login`,  {
        method: 'POST',
        body: JSON.stringify(loginJson),
    });
    return response.json();
});

export default component$(() => {
  return (
      <div class="min-h-screen bg-[#FEFCF7] flex items-center justify-center p-4">
          <div class="max-w-md w-full space-y-8">
              {/* Header */}
              <div class="text-center">
                  <div class="flex justify-center">
                      <LuRocket class="w-12 h-12 text-[#2D3748]"/>
                  </div>
                  <h2 class="mt-6 text-3xl font-serif font-semibold text-[#2D3748] leading-tight">
                      A personal home for your thoughts
                  </h2>
                  <p class="mt-3 text-base font-sans text-gray-600 leading-relaxed">
                      Where your words find warmth and your stories find their voice
                  </p>
              </div>

              {/* Login Form */}
              
              {/* Footer */}
              <div class="text-center">
                  <p class="text-sm text-gray-600">
                      Don't have an account?{' '}
                      <a href="#" class="font-medium text-[#7C9CBF] hover:text-[#6b8aab]">
                          Contact your administrator
                      </a>
                  </p>
              </div>
          </div>
      </div>
  );
});