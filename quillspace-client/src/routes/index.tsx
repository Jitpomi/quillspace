import { component$ } from "@builder.io/qwik";
import type { DocumentHead } from "@builder.io/qwik-city";
import { LuRocket, LuBookOpen, LuPenTool, LuUsers, LuTrendingUp, LuShield, LuArrowRight, LuCheck, LuStar, LuQuote, LuPlay, LuPhone, LuHeart, LuFeather, LuBarChart3, LuGlobe } from "@qwikest/icons/lucide";

export default component$(() => {
  return (
    <>
      {/* Hero Section - Writer's Sanctuary */}
      <section class="relative min-h-screen bg-gradient-to-br from-[#1a365d] via-[#2d3748] to-[#1a202c] text-white flex items-center overflow-hidden">
        {/* Background Image with Balanced Overlay */}
        <div class="absolute inset-0 z-0">
          <img 
            src="https://images.unsplash.com/photo-1481627834876-b7833e8f5570?ixlib=rb-4.0.3&auto=format&fit=crop&w=2070&q=80" 
            alt="Cozy library with warm lighting - your writing sanctuary with fellow authors" 
            class="w-full h-full object-cover opacity-45"
          />
          <div class="absolute inset-0 bg-gradient-to-br from-[#1a365d]/70 via-[#2d3748]/65 to-[#1a202c]/75"></div>
          
          {/* Floating Elements */}
          <div class="absolute top-20 left-10 w-2 h-2 bg-[#9CAF88]/30 rounded-full animate-pulse"></div>
          <div class="absolute top-40 right-20 w-1 h-1 bg-[#9CAF88]/40 rounded-full animate-pulse delay-1000"></div>
          <div class="absolute bottom-32 left-20 w-1.5 h-1.5 bg-[#9CAF88]/20 rounded-full animate-pulse delay-2000"></div>
        </div>
        
        <div class="relative z-10 max-w-7xl mx-auto px-4 py-20">
          <div class="text-center">
            {/* Logo with Animation */}
            <div class="flex justify-center items-center gap-4 mb-12 animate-fade-in">
              <div class="relative">
                <LuFeather class="w-16 h-16 text-[#9CAF88] drop-shadow-lg" />
                <div class="absolute -inset-2 bg-[#9CAF88]/10 rounded-full blur-xl"></div>
              </div>
              <h1 class="text-5xl font-bold tracking-wide">QuillSpace</h1>
            </div>
            
            {/* Main Headline with Better Typography */}
            <div class="mb-12">
              <h2 class="text-6xl md:text-8xl font-bold mb-6 leading-none tracking-tight">
                Your Writing
                <br class="hidden sm:block" />
                <span class="text-[#9CAF88] relative">
                  Sanctuary
                  <div class="absolute -bottom-2 left-0 right-0 h-1 bg-[#9CAF88]/30 rounded-full"></div>
                </span>
              </h2>
              
              {/* Tagline with Better Emphasis */}
              <div class="max-w-4xl mx-auto">
                <p class="text-2xl md:text-3xl font-light text-gray-100 mb-6 leading-relaxed">
                  Your personal library, but filled with fellow writers instead of loiterers
                </p>
                <div class="relative">
                  <p class="text-xl md:text-2xl text-gray-200 leading-relaxed italic">
                    Where writers manage their novels, track book performance, and connect with readers who truly understand the craft.
                  </p>
                  <div class="mt-6 text-2xl md:text-3xl font-medium text-[#9CAF88]">
                    Where words find warmth and stories find their voice.
                  </div>
                </div>
              </div>
            </div>
            
            {/* Enhanced CTA Buttons */}
            <div class="flex flex-col sm:flex-row gap-6 justify-center items-center mb-16">
              <a 
                href="/login" 
                class="group bg-[#9CAF88] text-white px-12 py-6 rounded-2xl font-bold text-xl hover:bg-[#8ba077] transition-all duration-300 shadow-2xl transform hover:scale-105 hover:shadow-[#9CAF88]/30 relative overflow-hidden"
              >
                <span class="relative z-10 flex items-center gap-3">
                  Enter Your Sanctuary
                  <LuArrowRight class="w-6 h-6 group-hover:translate-x-1 transition-transform" />
                </span>
                <div class="absolute inset-0 bg-gradient-to-r from-transparent via-white/10 to-transparent -translate-x-full group-hover:translate-x-full transition-transform duration-1000"></div>
              </a>
              <a 
                href="#features" 
                class="group border-2 border-[#9CAF88] text-[#9CAF88] px-12 py-6 rounded-2xl font-semibold text-xl hover:bg-[#9CAF88] hover:text-white transition-all duration-300 flex items-center gap-3 backdrop-blur-sm"
              >
                <LuHeart class="w-6 h-6 group-hover:scale-110 transition-transform" />
                Discover the Magic
              </a>
            </div>
            
            {/* Trust Indicators */}
            <div class="mb-16">
              <p class="text-gray-300 text-lg mb-4">Join 2,847+ writers in their creative sanctuary</p>
              <div class="flex justify-center items-center gap-8 text-gray-400">
                <div class="flex items-center gap-2">
                  <div class="flex -space-x-2">
                    <div class="w-8 h-8 bg-[#9CAF88] rounded-full border-2 border-white"></div>
                    <div class="w-8 h-8 bg-[#7a8c6f] rounded-full border-2 border-white"></div>
                    <div class="w-8 h-8 bg-[#9CAF88] rounded-full border-2 border-white"></div>
                  </div>
                  <span class="text-sm">Writing together now</span>
                </div>
                <div class="flex items-center gap-2">
                  <LuStar class="w-5 h-5 text-[#9CAF88] fill-current" />
                  <span class="text-sm">Loved by authors</span>
                </div>
              </div>
            </div>
            
            {/* Enhanced Core Features */}
            <div class="grid grid-cols-2 md:grid-cols-4 gap-6 max-w-6xl mx-auto">
              <div class="group text-center p-6 rounded-2xl bg-white/5 backdrop-blur-sm hover:bg-white/10 transition-all duration-300 hover:-translate-y-2">
                <div class="w-16 h-16 bg-gradient-to-br from-[#9CAF88] to-[#7a8c6f] rounded-2xl flex items-center justify-center mx-auto mb-4 group-hover:scale-110 transition-transform shadow-lg">
                  <LuFeather class="w-8 h-8 text-white" />
                </div>
                <h3 class="text-lg font-bold text-white mb-2">Novel Management</h3>
                <p class="text-sm text-gray-300">Organize every story, character, and plot thread</p>
              </div>
              
              <div class="group text-center p-6 rounded-2xl bg-white/5 backdrop-blur-sm hover:bg-white/10 transition-all duration-300 hover:-translate-y-2">
                <div class="w-16 h-16 bg-gradient-to-br from-[#9CAF88] to-[#7a8c6f] rounded-2xl flex items-center justify-center mx-auto mb-4 group-hover:scale-110 transition-transform shadow-lg">
                  <LuBarChart3 class="w-8 h-8 text-white" />
                </div>
                <h3 class="text-lg font-bold text-white mb-2">Performance Insights</h3>
                <p class="text-sm text-gray-300">Watch your books touch hearts worldwide</p>
              </div>
              
              <div class="group text-center p-6 rounded-2xl bg-white/5 backdrop-blur-sm hover:bg-white/10 transition-all duration-300 hover:-translate-y-2">
                <div class="w-16 h-16 bg-gradient-to-br from-[#9CAF88] to-[#7a8c6f] rounded-2xl flex items-center justify-center mx-auto mb-4 group-hover:scale-110 transition-transform shadow-lg">
                  <LuHeart class="w-8 h-8 text-white" />
                </div>
                <h3 class="text-lg font-bold text-white mb-2">Reader Community</h3>
                <p class="text-sm text-gray-300">Build lasting bonds with your audience</p>
              </div>
              
              <div class="group text-center p-6 rounded-2xl bg-white/5 backdrop-blur-sm hover:bg-white/10 transition-all duration-300 hover:-translate-y-2">
                <div class="w-16 h-16 bg-gradient-to-br from-[#9CAF88] to-[#7a8c6f] rounded-2xl flex items-center justify-center mx-auto mb-4 group-hover:scale-110 transition-transform shadow-lg">
                  <LuGlobe class="w-8 h-8 text-white" />
                </div>
                <h3 class="text-lg font-bold text-white mb-2">Author Website</h3>
                <p class="text-sm text-gray-300">Showcase your literary voice beautifully</p>
              </div>
            </div>
          </div>
        </div>
        
        {/* Scroll Indicator */}
        <div class="absolute bottom-8 left-1/2 transform -translate-x-1/2 animate-bounce">
          <div class="w-6 h-10 border-2 border-[#9CAF88]/50 rounded-full flex justify-center">
            <div class="w-1 h-3 bg-[#9CAF88] rounded-full mt-2 animate-pulse"></div>
          </div>
        </div>
      </section>

      {/* Second Section - Writer's Journey */}
      <section class="py-20 bg-white">
        <div class="max-w-6xl mx-auto px-4">
          <div class="grid lg:grid-cols-2 gap-16 items-center">
            {/* Left Content */}
            <div>
              <h2 class="text-4xl md:text-5xl font-bold text-[#2D3748] mb-8 leading-tight">
                Where Every Story
                <span class="text-[#9CAF88]"> Finds Its Home</span>
              </h2>
              <p class="text-xl text-gray-700 mb-10 leading-relaxed">
                This is where thoughts become stories, and stories find the hearts they're meant to touch. 
                Your personal sanctuary to nurture your novels, celebrate your successes, and build lasting connections with readers who cherish your words.
              </p>
              <div class="space-y-6 mb-10">
                <div class="flex items-start gap-4">
                  <div class="w-8 h-8 bg-[#9CAF88]/20 rounded-full flex items-center justify-center flex-shrink-0 mt-1">
                    <LuFeather class="w-5 h-5 text-[#9CAF88]" />
                  </div>
                  <div>
                    <h4 class="font-semibold text-[#2D3748] mb-1">Organize Your Literary Universe</h4>
                    <p class="text-gray-600">Keep all your novels, characters, and plot threads beautifully organized in one inspiring space</p>
                  </div>
                </div>
                <div class="flex items-start gap-4">
                  <div class="w-8 h-8 bg-[#9CAF88]/20 rounded-full flex items-center justify-center flex-shrink-0 mt-1">
                    <LuBarChart3 class="w-5 h-5 text-[#9CAF88]" />
                  </div>
                  <div>
                    <h4 class="font-semibold text-[#2D3748] mb-1">Celebrate Your Impact</h4>
                    <p class="text-gray-600">Watch as your books touch hearts and inspire minds with meaningful performance insights</p>
                  </div>
                </div>
                <div class="flex items-start gap-4">
                  <div class="w-8 h-8 bg-[#9CAF88]/20 rounded-full flex items-center justify-center flex-shrink-0 mt-1">
                    <LuGlobe class="w-5 h-5 text-[#9CAF88]" />
                  </div>
                  <div>
                    <h4 class="font-semibold text-[#2D3748] mb-1">Share Your Author Voice</h4>
                    <p class="text-gray-600">Build a beautiful author website that showcases your unique voice and literary journey</p>
                  </div>
                </div>
                <div class="flex items-start gap-4">
                  <div class="w-8 h-8 bg-[#9CAF88]/20 rounded-full flex items-center justify-center flex-shrink-0 mt-1">
                    <LuHeart class="w-5 h-5 text-[#9CAF88]" />
                  </div>
                  <div>
                    <h4 class="font-semibold text-[#2D3748] mb-1">Connect with Your Readers</h4>
                    <p class="text-gray-600">Build meaningful relationships with readers who love your stories and eagerly await your next book</p>
                  </div>
                </div>
              </div>
              <div class="flex flex-col sm:flex-row gap-4">
                <a 
                  href="/login" 
                  class="bg-[#9CAF88] text-white px-8 py-4 rounded-xl font-bold hover:bg-[#8ba077] transition-all duration-300 inline-flex items-center justify-center shadow-lg"
                >
                  Begin Your Journey
                  <LuArrowRight class="w-5 h-5 ml-2" />
                </a>
                <a 
                  href="#features" 
                  class="border-2 border-[#9CAF88] text-[#9CAF88] px-8 py-4 rounded-xl font-semibold hover:bg-[#9CAF88] hover:text-white transition-all duration-300 inline-flex items-center justify-center"
                >
                  Explore Features
                </a>
              </div>
            </div>
            
            {/* Right Image */}
            <div class="relative">
              <img 
                src="https://images.unsplash.com/photo-1434030216411-0b793f4b4173?ixlib=rb-4.0.3&auto=format&fit=crop&w=800&q=80" 
                alt="Cozy writing space with notebook and warm lighting" 
                class="w-full h-[500px] object-cover rounded-2xl shadow-2xl"
              />
              <div class="absolute inset-0 bg-gradient-to-t from-black/30 to-transparent rounded-2xl"></div>
              
              {/* Floating Stats Cards */}
              <div class="absolute top-6 right-6 bg-white/95 backdrop-blur-sm rounded-xl shadow-xl p-4 max-w-xs">
                <div class="flex items-center gap-3">
                  <div class="w-12 h-12 bg-[#9CAF88]/10 rounded-full flex items-center justify-center">
                    <LuHeart class="w-6 h-6 text-[#9CAF88]" />
                  </div>
                  <div>
                    <div class="font-bold text-[#2D3748] text-lg">2,847</div>
                    <div class="text-sm text-gray-600">Readers reached this month</div>
                  </div>
                </div>
              </div>
              
              <div class="absolute bottom-6 left-6 bg-white/95 backdrop-blur-sm rounded-xl shadow-xl p-4 max-w-xs">
                <div class="flex items-center gap-3">
                  <div class="w-12 h-12 bg-[#9CAF88]/10 rounded-full flex items-center justify-center">
                    <LuFeather class="w-6 h-6 text-[#9CAF88]" />
                  </div>
                  <div>
                    <div class="font-bold text-[#2D3748] text-lg">Chapter 12</div>
                    <div class="text-sm text-gray-600">Currently writing</div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section id="features" class="py-20 bg-gray-50">
        <div class="max-w-6xl mx-auto px-4">
          <div class="text-center mb-16">
            <h2 class="text-4xl md:text-5xl font-bold text-[#2D3748] mb-6">
              Everything You Need to Flourish as a Writer
            </h2>
            <p class="text-xl text-gray-600 max-w-3xl mx-auto">
              From organizing your first draft to celebrating bestseller status, QuillSpace provides the tools and insights to support your entire writing journey
            </p>
          </div>
          
          <div class="grid md:grid-cols-2 lg:grid-cols-3 gap-8">
            
            {/* Novel Management */}
            <div class="bg-white rounded-2xl p-8 shadow-lg hover:shadow-2xl transition-all duration-300 text-center group hover:-translate-y-2">
              <div class="w-20 h-20 bg-gradient-to-br from-[#9CAF88] to-[#7a8c6f] rounded-full flex items-center justify-center mx-auto mb-6 group-hover:scale-110 transition-transform">
                <LuFeather class="w-10 h-10 text-white" />
              </div>
              <h3 class="text-2xl font-bold text-[#2D3748] mb-4">Novel Organization</h3>
              <p class="text-gray-600 leading-relaxed mb-6">
                Keep your manuscripts, character profiles, plot outlines, and research notes beautifully organized. 
                Never lose track of your literary universe again.
              </p>
              <div class="text-[#9CAF88] font-semibold">Organize • Plan • Create</div>
            </div>

            {/* Performance Analytics */}
            <div class="bg-white rounded-2xl p-8 shadow-lg hover:shadow-2xl transition-all duration-300 text-center group hover:-translate-y-2">
              <div class="w-20 h-20 bg-gradient-to-br from-[#9CAF88] to-[#7a8c6f] rounded-full flex items-center justify-center mx-auto mb-6 group-hover:scale-110 transition-transform">
                <LuBarChart3 class="w-10 h-10 text-white" />
              </div>
              <h3 class="text-2xl font-bold text-[#2D3748] mb-4">Book Performance Insights</h3>
              <p class="text-gray-600 leading-relaxed mb-6">
                Track how your books are performing with meaningful analytics. See reader engagement, 
                sales trends, and discover what resonates most with your audience.
              </p>
              <div class="text-[#9CAF88] font-semibold">Track • Analyze • Grow</div>
            </div>

            {/* Reader Connection */}
            <div class="bg-white rounded-2xl p-8 shadow-lg hover:shadow-2xl transition-all duration-300 text-center group hover:-translate-y-2">
              <div class="w-20 h-20 bg-gradient-to-br from-[#9CAF88] to-[#7a8c6f] rounded-full flex items-center justify-center mx-auto mb-6 group-hover:scale-110 transition-transform">
                <LuHeart class="w-10 h-10 text-white" />
              </div>
              <h3 class="text-2xl font-bold text-[#2D3748] mb-4">Reader Community</h3>
              <p class="text-gray-600 leading-relaxed mb-6">
                Build meaningful connections with readers who love your work. 
                Engage with your audience and create a loyal community around your stories.
              </p>
              <div class="text-[#9CAF88] font-semibold">Connect • Engage • Inspire</div>
            </div>

            {/* Author Website */}
            <div class="bg-white rounded-2xl p-8 shadow-lg hover:shadow-2xl transition-all duration-300 text-center group hover:-translate-y-2">
              <div class="w-20 h-20 bg-gradient-to-br from-[#9CAF88] to-[#7a8c6f] rounded-full flex items-center justify-center mx-auto mb-6 group-hover:scale-110 transition-transform">
                <LuGlobe class="w-10 h-10 text-white" />
              </div>
              <h3 class="text-2xl font-bold text-[#2D3748] mb-4">Beautiful Author Website</h3>
              <p class="text-gray-600 leading-relaxed mb-6">
                Create a stunning author website that showcases your books, shares your story, 
                and helps readers discover your work. No technical skills required.
              </p>
              <div class="text-[#9CAF88] font-semibold">Showcase • Share • Shine</div>
            </div>

            {/* Writing Tools */}
            <div class="bg-white rounded-2xl p-8 shadow-lg hover:shadow-2xl transition-all duration-300 text-center group hover:-translate-y-2">
              <div class="w-20 h-20 bg-gradient-to-br from-[#9CAF88] to-[#7a8c6f] rounded-full flex items-center justify-center mx-auto mb-6 group-hover:scale-110 transition-transform">
                <LuPenTool class="w-10 h-10 text-white" />
              </div>
              <h3 class="text-2xl font-bold text-[#2D3748] mb-4">Writing Workflow</h3>
              <p class="text-gray-600 leading-relaxed mb-6">
                Streamline your writing process with tools designed for authors. 
                Set goals, track progress, and maintain momentum on your literary journey.
              </p>
              <div class="text-[#9CAF88] font-semibold">Write • Progress • Achieve</div>
            </div>

            {/* Secure & Private */}
            <div class="bg-white rounded-2xl p-8 shadow-lg hover:shadow-2xl transition-all duration-300 text-center group hover:-translate-y-2">
              <div class="w-20 h-20 bg-gradient-to-br from-[#9CAF88] to-[#7a8c6f] rounded-full flex items-center justify-center mx-auto mb-6 group-hover:scale-110 transition-transform">
                <LuShield class="w-10 h-10 text-white" />
              </div>
              <h3 class="text-2xl font-bold text-[#2D3748] mb-4">Secure & Private</h3>
              <p class="text-gray-600 leading-relaxed mb-6">
                Your creative work is protected with enterprise-grade security. 
                Your manuscripts, ideas, and reader data remain completely private and secure.
              </p>
              <div class="text-[#9CAF88] font-semibold">Protect • Secure • Trust</div>
            </div>
          </div>
        </div>
      </section>

      {/* Stats Section - Writer Community */}
      <section class="py-20 bg-[#2D3748] text-white">
        <div class="max-w-6xl mx-auto px-4">
          <div class="text-center mb-12">
            <h2 class="text-3xl md:text-4xl font-bold mb-4">Join a Thriving Community of Writers</h2>
            <p class="text-xl text-gray-300">Where stories come to life and writers find their voice</p>
          </div>
          <div class="grid grid-cols-2 md:grid-cols-4 gap-8 text-center">
            <div class="group">
              <div class="text-4xl md:text-5xl font-bold text-[#9CAF88] mb-2 group-hover:scale-110 transition-transform">2,847</div>
              <div class="text-lg font-semibold text-gray-200">Active Writers</div>
              <div class="text-sm text-gray-300 mt-1">Creating daily</div>
            </div>
            <div class="group">
              <div class="text-4xl md:text-5xl font-bold text-[#9CAF88] mb-2 group-hover:scale-110 transition-transform">15,692</div>
              <div class="text-lg font-semibold text-gray-200">Novels Managed</div>
              <div class="text-sm text-gray-300 mt-1">Stories in progress</div>
            </div>
            <div class="group">
              <div class="text-4xl md:text-5xl font-bold text-[#9CAF88] mb-2 group-hover:scale-110 transition-transform">1.2M</div>
              <div class="text-lg font-semibold text-gray-200">Reader Connections</div>
              <div class="text-sm text-gray-300 mt-1">Hearts touched</div>
            </div>
            <div class="group">
              <div class="text-4xl md:text-5xl font-bold text-[#9CAF88] mb-2 group-hover:scale-110 transition-transform">98%</div>
              <div class="text-lg font-semibold text-gray-200">Writer Satisfaction</div>
              <div class="text-sm text-gray-300 mt-1">Love their sanctuary</div>
            </div>
          </div>
        </div>
      </section>

      {/* Writer Success Stories */}
      <section class="py-20 bg-white">
        <div class="max-w-6xl mx-auto px-4">
          <div class="text-center mb-16">
            <h2 class="text-4xl md:text-5xl font-bold text-[#2c1810] mb-6">Stories of Success</h2>
            <p class="text-xl text-gray-600 max-w-3xl mx-auto">
              Real writers sharing how QuillSpace became their creative sanctuary and helped them achieve their dreams
            </p>
          </div>
          
          <div class="grid md:grid-cols-2 gap-16 items-center mb-16">
            {/* Left - Writer's Journey Visual */}
            <div class="relative">
              <img 
                src="https://images.unsplash.com/photo-1522202176988-66273c2fd55f?ixlib=rb-4.0.3&auto=format&fit=crop&w=800&q=80" 
                alt="Writer celebrating success with laptop and books" 
                class="w-full h-[400px] object-cover rounded-2xl shadow-2xl"
              />
              <div class="absolute inset-0 bg-gradient-to-t from-black/40 to-transparent rounded-2xl"></div>
              
              {/* Success Metrics Overlay */}
              <div class="absolute top-6 left-6 bg-white/95 backdrop-blur-sm rounded-xl shadow-xl p-4">
                <div class="flex items-center gap-3">
                  <div class="w-10 h-10 bg-[#d4af37]/10 rounded-full flex items-center justify-center">
                    <LuTrendingUp class="w-5 h-5 text-[#d4af37]" />
                  </div>
                  <div>
                    <div class="font-bold text-[#2c1810]">+347%</div>
                    <div class="text-xs text-gray-600">Reader growth</div>
                  </div>
                </div>
              </div>
              
              <div class="absolute bottom-6 right-6 bg-white/95 backdrop-blur-sm rounded-xl shadow-xl p-4">
                <div class="flex items-center gap-3">
                  <div class="w-10 h-10 bg-[#d4af37]/10 rounded-full flex items-center justify-center">
                    <LuBookOpen class="w-5 h-5 text-[#d4af37]" />
                  </div>
                  <div>
                    <div class="font-bold text-[#2c1810]">3 Novels</div>
                    <div class="text-xs text-gray-600">Completed</div>
                  </div>
                </div>
              </div>
            </div>
            
            {/* Right - Success Story */}
            <div>
              <div class="mb-8">
                <LuQuote class="w-12 h-12 text-[#d4af37] mb-4" />
                <blockquote class="text-2xl text-gray-700 font-light italic leading-relaxed mb-6">
                  "QuillSpace transformed my chaotic writing process into an organized, inspiring journey. 
                  I went from struggling to finish one chapter to completing three novels and building a community of 5,000 devoted readers."
                </blockquote>
                <div class="flex items-center gap-4">
                  <img 
                    src="https://images.unsplash.com/photo-1494790108755-2616b612b786?ixlib=rb-4.0.3&auto=format&fit=crop&w=100&q=80" 
                    alt="Sarah Chen, Romance Author" 
                    class="w-16 h-16 rounded-full object-cover"
                  />
                  <div>
                    <div class="font-bold text-[#2c1810] text-lg">Sarah Chen</div>
                    <div class="text-gray-600">Romance Author • 3 Published Novels</div>
                  </div>
                </div>
              </div>
              
              <div class="space-y-4">
                <div class="flex items-start gap-3">
                  <LuHeart class="w-6 h-6 text-[#d4af37] flex-shrink-0 mt-1" />
                  <div>
                    <h4 class="font-semibold text-[#2c1810] mb-1">From Scattered Notes to Bestseller</h4>
                    <p class="text-gray-600">Organized 15 years of story ideas into a coherent writing system</p>
                  </div>
                </div>
                <div class="flex items-start gap-3">
                  <LuUsers class="w-6 h-6 text-[#d4af37] flex-shrink-0 mt-1" />
                  <div>
                    <h4 class="font-semibold text-[#2c1810] mb-1">Built a Loyal Reader Community</h4>
                    <p class="text-gray-600">Connected with readers who eagerly await each new release</p>
                  </div>
                </div>
                <div class="flex items-start gap-3">
                  <LuTrendingUp class="w-6 h-6 text-[#d4af37] flex-shrink-0 mt-1" />
                  <div>
                    <h4 class="font-semibold text-[#2c1810] mb-1">Tracked Success & Growth</h4>
                    <p class="text-gray-600">Watched her books climb the charts with real-time insights</p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Your Writing Journey Section */}
      <section class="py-20 bg-gradient-to-b from-amber-50 to-white">
        <div class="max-w-6xl mx-auto px-4 text-center">
          <h2 class="text-4xl md:text-5xl font-bold text-[#2c1810] mb-6">Your Writing Journey Starts Here</h2>
          <p class="text-xl text-gray-600 mb-16 max-w-3xl mx-auto">
            From that first spark of inspiration to celebrating with your readers, QuillSpace supports every step of your creative journey
          </p>
          
          <div class="grid md:grid-cols-2 lg:grid-cols-4 gap-8">
            <div class="text-center group">
              <div class="w-20 h-20 bg-gradient-to-br from-[#d4af37] to-[#b8941f] rounded-full flex items-center justify-center mx-auto mb-6 group-hover:scale-110 transition-transform">
                <LuFeather class="w-10 h-10 text-white" />
              </div>
              <h3 class="text-xl font-bold text-[#2c1810] mb-3">Organize Your Ideas</h3>
              <p class="text-gray-600 leading-relaxed">
                Capture every character, plot twist, and world-building detail in your personal literary sanctuary. 
                Never lose a brilliant idea again.
              </p>
            </div>
            
            <div class="text-center group">
              <div class="w-20 h-20 bg-gradient-to-br from-[#d4af37] to-[#b8941f] rounded-full flex items-center justify-center mx-auto mb-6 group-hover:scale-110 transition-transform">
                <LuPenTool class="w-10 h-10 text-white" />
              </div>
              <h3 class="text-xl font-bold text-[#2c1810] mb-3">Write & Track Progress</h3>
              <p class="text-gray-600 leading-relaxed">
                Set writing goals, track your daily progress, and celebrate milestones as you bring your stories to life, 
                one chapter at a time.
              </p>
            </div>
            
            <div class="text-center group">
              <div class="w-20 h-20 bg-gradient-to-br from-[#d4af37] to-[#b8941f] rounded-full flex items-center justify-center mx-auto mb-6 group-hover:scale-110 transition-transform">
                <LuGlobe class="w-10 h-10 text-white" />
              </div>
              <h3 class="text-xl font-bold text-[#2c1810] mb-3">Share Your Voice</h3>
              <p class="text-gray-600 leading-relaxed">
                Build a beautiful author website that showcases your unique voice and helps readers discover the magic 
                you've created.
              </p>
            </div>
            
            <div class="text-center group">
              <div class="w-20 h-20 bg-gradient-to-br from-[#d4af37] to-[#b8941f] rounded-full flex items-center justify-center mx-auto mb-6 group-hover:scale-110 transition-transform">
                <LuHeart class="w-10 h-10 text-white" />
              </div>
              <h3 class="text-xl font-bold text-[#2c1810] mb-3">Connect & Celebrate</h3>
              <p class="text-gray-600 leading-relaxed">
                Watch your books find their audience, track your success, and build meaningful relationships with readers 
                who love your stories.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* Writer Testimonials Section */}
      <section class="py-20 bg-white">
        <div class="max-w-6xl mx-auto px-4 text-center">
          <h2 class="text-4xl md:text-5xl font-bold text-[#2c1810] mb-6">Writers Love Their Sanctuary</h2>
          <p class="text-xl text-gray-600 mb-16 max-w-3xl mx-auto">
            Hear from writers who found their creative home in QuillSpace and transformed their literary dreams into reality
          </p>
          
          <div class="grid md:grid-cols-2 lg:grid-cols-3 gap-8">
            <div class="bg-gradient-to-br from-amber-50 to-white p-8 rounded-2xl shadow-lg hover:shadow-xl transition-shadow">
              <div class="flex items-center mb-6">
                {[...Array(5)].map((_, i) => (
                  <LuStar key={i} class="w-5 h-5 text-[#d4af37] fill-current" />
                ))}
              </div>
              <p class="text-gray-700 mb-6 italic text-lg leading-relaxed">
                "QuillSpace became my creative sanctuary. I went from scattered notes to three published novels. 
                The organization tools and reader insights changed everything."
              </p>
              <div class="flex items-center gap-3">
                <img 
                  src="https://images.unsplash.com/photo-1494790108755-2616b612b786?ixlib=rb-4.0.3&auto=format&fit=crop&w=100&q=80" 
                  alt="Sarah Chen" 
                  class="w-12 h-12 rounded-full object-cover"
                />
                <div class="text-left">
                  <p class="font-bold text-[#2c1810]">Sarah Chen</p>
                  <p class="text-gray-600 text-sm">Romance Author • 3 Novels</p>
                </div>
              </div>
            </div>
            
            <div class="bg-gradient-to-br from-amber-50 to-white p-8 rounded-2xl shadow-lg hover:shadow-xl transition-shadow">
              <div class="flex items-center mb-6">
                {[...Array(5)].map((_, i) => (
                  <LuStar key={i} class="w-5 h-5 text-[#d4af37] fill-current" />
                ))}
              </div>
              <p class="text-gray-700 mb-6 italic text-lg leading-relaxed">
                "The performance insights helped me understand my readers better. I can see which chapters resonate most 
                and connect with my audience like never before."
              </p>
              <div class="flex items-center gap-3">
                <img 
                  src="https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?ixlib=rb-4.0.3&auto=format&fit=crop&w=100&q=80" 
                  alt="Michael Rodriguez" 
                  class="w-12 h-12 rounded-full object-cover"
                />
                <div class="text-left">
                  <p class="font-bold text-[#2c1810]">Michael Rodriguez</p>
                  <p class="text-gray-600 text-sm">Sci-Fi Author • 2 Series</p>
                </div>
              </div>
            </div>
            
            <div class="bg-gradient-to-br from-amber-50 to-white p-8 rounded-2xl shadow-lg hover:shadow-xl transition-shadow">
              <div class="flex items-center mb-6">
                {[...Array(5)].map((_, i) => (
                  <LuStar key={i} class="w-5 h-5 text-[#d4af37] fill-current" />
                ))}
              </div>
              <p class="text-gray-700 mb-6 italic text-lg leading-relaxed">
                "Building my author website was so easy! Now readers can discover all my books in one beautiful place. 
                My fan community has grown by 400%."
              </p>
              <div class="flex items-center gap-3">
                <img 
                  src="https://images.unsplash.com/photo-1438761681033-6461ffad8d80?ixlib=rb-4.0.3&auto=format&fit=crop&w=100&q=80" 
                  alt="Emma Thompson" 
                  class="w-12 h-12 rounded-full object-cover"
                />
                <div class="text-left">
                  <p class="font-bold text-[#2c1810]">Emma Thompson</p>
                  <p class="text-gray-600 text-sm">Mystery Author • 5 Books</p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Final CTA Section */}
      <section class="py-24 bg-gradient-to-br from-[#2c1810] via-[#3d2817] to-[#1a365d] text-white text-center relative overflow-hidden">
        {/* Background Pattern */}
        <div class="absolute inset-0 opacity-10">
          <img 
            src="https://images.unsplash.com/photo-1455390582262-044cdead277a?ixlib=rb-4.0.3&auto=format&fit=crop&w=2070&q=80" 
            alt="Cozy writing sanctuary" 
            class="w-full h-full object-cover"
          />
        </div>
        
        <div class="relative z-10 max-w-5xl mx-auto px-4">
          <div class="mb-8">
            <LuFeather class="w-16 h-16 text-[#d4af37] mx-auto mb-6" />
          </div>
          
          <h2 class="text-4xl md:text-6xl font-bold mb-8 leading-tight">
            Your Stories Are Waiting
            <span class="text-[#d4af37]"> to Be Told</span>
          </h2>
          
          <p class="text-xl md:text-2xl text-amber-200/90 mb-12 leading-relaxed max-w-4xl mx-auto">
            Join thousands of writers who have found their creative sanctuary in QuillSpace. 
            <br class="hidden md:block"/>
            <span class="text-[#d4af37] font-medium">Where words find warmth and stories find their voice.</span>
          </p>
          
          <div class="flex flex-col sm:flex-row gap-6 justify-center items-center">
            <a 
              href="/login" 
              class="bg-[#d4af37] text-[#2c1810] px-10 py-5 rounded-xl font-bold text-xl hover:bg-[#b8941f] transition-all duration-300 shadow-2xl transform hover:scale-105 hover:shadow-[#d4af37]/25"
            >
              Enter Your Sanctuary
            </a>
            <a 
              href="#features" 
              class="border-2 border-[#d4af37] text-[#d4af37] px-10 py-5 rounded-xl font-semibold text-xl hover:bg-[#d4af37] hover:text-[#2c1810] transition-all duration-300 flex items-center gap-3"
            >
              <LuHeart class="w-6 h-6" />
              Discover Your Home
            </a>
          </div>
          
          <div class="mt-12 text-amber-200/70">
            <p class="text-lg">Join 2,847+ writers in their creative sanctuary</p>
          </div>
        </div>
      </section>

      {/* Simple Footer */}
      <footer class="py-12 bg-[#2c1810] text-white text-center">
        <div class="max-w-6xl mx-auto px-4">
          <div class="flex justify-center items-center gap-3 mb-6">
            <LuFeather class="w-8 h-8 text-[#d4af37]" />
            <span class="text-2xl font-bold">QuillSpace</span>
          </div>
          <p class="text-amber-200/80 text-lg mb-2">
            Where words find warmth and stories find their voice
          </p>
          <p class="text-amber-200/60">
            © 2025 QuillSpace. Your Writing Sanctuary.
          </p>
        </div>
      </footer>
    </>
  );
});

export const head: DocumentHead = {
  title: "QuillSpace - Your Writing Sanctuary | Where Stories Find Their Voice",
  meta: [
    {
      name: "description",
      content: "Your personal writing sanctuary where you manage novels, track book performance, and connect with readers. Organize your thoughts, build your author website, and watch your stories touch hearts. Where words find warmth and stories find their voice.",
    },
    {
      name: "keywords",
      content: "writing sanctuary, novel management, author platform, book performance tracking, reader connection, writing organization, author website builder, creative writing tools, manuscript management, writer community",
    },
    {
      property: "og:title",
      content: "QuillSpace - Your Writing Sanctuary | Where Stories Find Their Voice",
    },
    {
      property: "og:description", 
      content: "Your personal writing sanctuary where you manage novels, track book performance, and connect with readers. Where words find warmth and stories find their voice.",
    },
    {
      property: "og:image",
      content: "https://images.unsplash.com/photo-1455390582262-044cdead277a?ixlib=rb-4.0.3&auto=format&fit=crop&w=1200&q=80",
    },
    {
      property: "og:type",
      content: "website",
    },
    {
      name: "twitter:card",
      content: "summary_large_image",
    },
    {
      name: "twitter:title",
      content: "QuillSpace - Your Writing Sanctuary",
    },
    {
      name: "twitter:description",
      content: "Your personal writing sanctuary where you manage novels, track book performance, and connect with readers. Where words find warmth and stories find their voice.",
    },
  ],
};
