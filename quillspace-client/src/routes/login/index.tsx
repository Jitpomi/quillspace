import { component$ } from "@builder.io/qwik";
import {LuFeather, LuHeart} from "@qwikest/icons/lucide";
import {RequestHandler, routeLoader$} from "@builder.io/qwik-city";
import {InitialValues} from "@modular-forms/qwik";
import {LoginRequest} from "~/api/schema";
import LoginForm from "~/components/login-form";

export const useFormLoader = routeLoader$<InitialValues<LoginRequest>>(() => ({
    email: '',
    password: '',
}));


export const onGet: RequestHandler = async ({ cookie, json }) => {
    let count = cookie.get('Qwik.demo.count')?.number() || 0;
    count++;
    cookie.set('Qwik.demo.count', count);
    json(200, { count });
};

export default component$(() => {
const login = useFormLoader();
  return (
      <div class="min-h-screen relative bg-gradient-to-br from-[#1a365d] via-[#2d3748] to-[#1a202c] text-white flex items-center justify-center p-4 overflow-hidden">
          {/* Background Image with Same Library Atmosphere */}
          <div class="absolute inset-0 z-0">
              <img 
                  src="https://images.unsplash.com/photo-1481627834876-b7833e8f5570?ixlib=rb-4.0.3&auto=format&fit=crop&w=2070&q=80" 
                  alt="Cozy library with warm lighting - your writing sanctuary" 
                  class="w-full h-full object-cover opacity-45"
              />
              <div class="absolute inset-0 bg-gradient-to-br from-[#1a365d]/70 via-[#2d3748]/65 to-[#1a202c]/75"></div>
              
              {/* Floating Elements */}
              <div class="absolute top-20 left-10 w-2 h-2 bg-[#9CAF88]/30 rounded-full animate-pulse"></div>
              <div class="absolute top-40 right-20 w-1 h-1 bg-[#9CAF88]/40 rounded-full animate-pulse delay-1000"></div>
              <div class="absolute bottom-32 left-20 w-1.5 h-1.5 bg-[#9CAF88]/20 rounded-full animate-pulse delay-2000"></div>
          </div>

          <div class="relative z-10 max-w-md w-full">
              {/* Header with Library Vibe */}
              <div class="text-center mb-8">
                  <div class="flex justify-center items-center gap-3 mb-6">
                      <div class="relative">
                          <LuFeather class="w-12 h-12 text-[#9CAF88] drop-shadow-lg" />
                          <div class="absolute -inset-2 bg-[#9CAF88]/10 rounded-full blur-xl"></div>
                      </div>
                      <h1 class="text-3xl font-bold tracking-wide">QuillSpace</h1>
                  </div>
                  
                  <h2 class="text-2xl md:text-3xl font-bold mb-4 leading-tight">
                      Welcome to Your
                      <span class="text-[#9CAF88]"> Sanctuary</span>
                  </h2>
                  
                  <p class="text-lg text-gray-200 mb-2 leading-relaxed">
                      Your personal library, filled with fellow writers
                  </p>
                  <p class="text-base text-gray-300 italic">
                      Where words find warmth and stories find their voice
                  </p>
              </div>

              {/* Login Form Container with Glass Effect */}
              <div class="bg-white/10 backdrop-blur-sm rounded-2xl p-8 shadow-2xl border border-white/20">
                  <LoginForm {...login} />
              </div>
              
              {/* Footer */}
              <div class="text-center mt-6">
                  <p class="text-sm text-gray-300">
                      Don't have an account?{' '}
                      <a href="#" class="font-medium text-[#9CAF88] hover:text-[#8ba077] transition-colors">
                          Contact your administrator
                      </a>
                  </p>
              </div>
              
              {/* Back to Home Link */}
              <div class="text-center mt-4">
                  <a href="/" class="inline-flex items-center gap-2 text-sm text-gray-400 hover:text-[#9CAF88] transition-colors">
                      <LuHeart class="w-4 h-4" />
                      Back to sanctuary
                  </a>
              </div>
          </div>
      </div>
  );
});