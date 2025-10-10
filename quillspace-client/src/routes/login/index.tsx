import { component$ } from "@builder.io/qwik";
import {LuRocket} from "@qwikest/icons/lucide";
import {routeLoader$} from "@builder.io/qwik-city";
import {InitialValues} from "@modular-forms/qwik";
import {LoginRequest} from "~/api/schema";
import LoginForm from "~/components/login-form";

export const useFormLoader = routeLoader$<InitialValues<LoginRequest>>(() => ({
    email: '',
    password: '',
}));

export default component$(() => {
const login = useFormLoader();
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
              <LoginForm {...login} />
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