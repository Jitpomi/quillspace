import { component$ } from "@builder.io/qwik";
import type { DocumentHead } from "@builder.io/qwik-city";
import { LuBookOpen, LuPenTool, LuUsers, LuTrendingUp, LuShield, LuArrowRight, LuStar, LuQuote, LuHeart, LuFeather, LuBarChart3, LuGlobe } from "@qwikest/icons/lucide";
import { WrapBalancer } from "qwikjs-wrap-balancer";

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
            <div class="mb-12 text-center">
              <div style="min-height: 240px;" class="hero-text-container flex flex-col justify-center items-center">
                <h2 class="stable-text text-6xl md:text-8xl font-bold mb-6 leading-none tracking-tight">
                  Your Writing
                  <br class="hidden sm:block" />
                  <span class="text-[#9CAF88] relative">
                    Sanctuary
                    <div class="absolute -bottom-2 left-0 right-0 h-1 bg-[#9CAF88]/30 rounded-full"></div>
                  </span>
                </h2>
              </div>
              
              {/* Tagline with Better Emphasis */}
              <div class="max-w-4xl mx-auto text-center">
                <div style="min-height: 100px; text-wrap: balance;" class="flex flex-col justify-center items-center">
                  <p class="text-2xl md:text-3xl font-light text-gray-100 mb-6 leading-relaxed">
                    A quiet place to build your book, share your voice, and feel less alone on the way there.
                  </p>
                </div>
                <div class="relative">
                  <div style="min-height: 70px; text-wrap: balance;" class="flex flex-col justify-center items-center">
                    <p class="text-xl md:text-2xl text-gray-200 leading-relaxed italic">
                      Here, writers shape their stories, follow their books into the world, and connect with readers who read with soul.
                    </p>
                  </div>
                  <div style="min-height: 70px; text-wrap: balance;" class="flex flex-col justify-center items-center">
                    <div class="mt-6 text-2xl md:text-3xl font-medium text-[#9CAF88]">
                      Where language matters, and stories are given the room to breathe.
                    </div>
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
                  Step Inside
                  <LuArrowRight class="w-6 h-6 group-hover:translate-x-1 transition-transform" />
                </span>
                <div class="absolute inset-0 bg-gradient-to-r from-transparent via-white/10 to-transparent -translate-x-full group-hover:translate-x-full transition-transform duration-1000"></div>
              </a>
              <a 
                href="#features" 
                class="group border-2 border-[#9CAF88] text-[#9CAF88] px-12 py-6 rounded-2xl font-semibold text-xl hover:bg-[#9CAF88] hover:text-white transition-all duration-300 flex items-center gap-3 backdrop-blur-sm"
              >
                <LuHeart class="w-6 h-6 group-hover:scale-110 transition-transform" />
                Take a Look Around
              </a>
            </div>
            
            {/* Trust Indicators */}
            <div class="mb-16">
              <p class="text-gray-300 text-lg mb-4">Something quiet is beginning here. A place for writers who take their work seriously.</p>
              <div class="flex justify-center items-center gap-8 text-gray-400">
                <div class="flex items-center gap-2">
                  <div class="flex -space-x-2">
                    <div class="w-8 h-8 bg-[#9CAF88] rounded-full border-2 border-white"></div>
                    <div class="w-8 h-8 bg-[#7a8c6f] rounded-full border-2 border-white"></div>
                    <div class="w-8 h-8 bg-[#9CAF88] rounded-full border-2 border-white"></div>
                  </div>
                  <span class="text-sm">Early writers gathering</span>
                </div>
                <div class="flex items-center gap-2">
                  <LuStar class="w-5 h-5 text-[#9CAF88] fill-current" />
                  <span class="text-sm">Built with care</span>
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
              <div style="min-height: 120px;" class="flex flex-col justify-center">
                <WrapBalancer>
                  <h2 class="text-4xl md:text-5xl font-bold text-[#2D3748] mb-8 leading-tight">
                    Where Stories Settle—
                    <span class="text-[#9CAF88]">and Begin to Grow</span>
                  </h2>
                </WrapBalancer>
              </div>
              <div style="min-height: 100px; text-wrap: balance;" class="flex flex-col justify-center">
                <p class="text-xl text-gray-700 mb-10 leading-relaxed">
                  This is where your stories take shape—slowly, honestly—and begin to find their way into the hands they were meant for. A quiet space to write deeply, share deliberately, and connect with readers who read like it matters.
                </p>
              </div>
              <div class="space-y-6 mb-10">
                <div class="flex items-start gap-4">
                  <div class="w-8 h-8 bg-[#9CAF88]/20 rounded-full flex items-center justify-center flex-shrink-0 mt-1">
                    <LuFeather class="w-5 h-5 text-[#9CAF88]" />
                  </div>
                  <div>
                    <h4 class="font-semibold text-[#2D3748] mb-1">Keep Your Worlds in One Place</h4>
                    <p class="text-gray-600">All your stories, characters, and tangled threads—gathered, clear, and close at hand.</p>
                  </div>
                </div>
                <div class="flex items-start gap-4">
                  <div class="w-8 h-8 bg-[#9CAF88]/20 rounded-full flex items-center justify-center flex-shrink-0 mt-1">
                    <LuBarChart3 class="w-5 h-5 text-[#9CAF88]" />
                  </div>
                  <div>
                    <h4 class="font-semibold text-[#2D3748] mb-1">See Where Your Stories Land</h4>
                    <p class="text-gray-600">Follow your book's quiet ripple—who it's reaching, and how it's being held.</p>
                  </div>
                </div>
                <div class="flex items-start gap-4">
                  <div class="w-8 h-8 bg-[#9CAF88]/20 rounded-full flex items-center justify-center flex-shrink-0 mt-1">
                    <LuGlobe class="w-5 h-5 text-[#9CAF88]" />
                  </div>
                  <div>
                    <h4 class="font-semibold text-[#2D3748] mb-1">Build the Home Your Voice Deserves</h4>
                    <p class="text-gray-600">A space that reflects the shape of your writing life—clear, beautiful, unmistakably yours.</p>
                  </div>
                </div>
                <div class="flex items-start gap-4">
                  <div class="w-8 h-8 bg-[#9CAF88]/20 rounded-full flex items-center justify-center flex-shrink-0 mt-1">
                    <LuHeart class="w-5 h-5 text-[#9CAF88]" />
                  </div>
                  <div>
                    <h4 class="font-semibold text-[#2D3748] mb-1">Find the Readers Who Feel What You Wrote</h4>
                    <p class="text-gray-600">The ones who linger on your language. The ones who wait for what's next.</p>
                  </div>
                </div>
              </div>
              <div class="flex flex-col sm:flex-row gap-4">
                <a 
                  href="/login"
                  class="bg-[#9CAF88] text-white px-8 py-4 rounded-xl font-bold hover:bg-[#8ba077] transition-all duration-300 inline-flex items-center justify-center shadow-lg"
                >
                  Step Into the Quiet
                  <LuArrowRight class="w-5 h-5 ml-2" />
                </a>
                <a 
                  href="#features" 
                  class="border-2 border-[#9CAF88] text-[#9CAF88] px-8 py-4 rounded-xl font-semibold hover:bg-[#9CAF88] hover:text-white transition-all duration-300 inline-flex items-center justify-center"
                >
                  See What's Here
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
                    <div class="font-bold text-[#2D3748] text-lg">Early Access</div>
                    <div class="text-sm text-gray-600">Join the beginning</div>
                  </div>
                </div>
              </div>
              
              <div class="absolute bottom-6 left-6 bg-white/95 backdrop-blur-sm rounded-xl shadow-xl p-4 max-w-xs">
                <div class="flex items-center gap-3">
                  <div class="w-12 h-12 bg-[#9CAF88]/10 rounded-full flex items-center justify-center">
                    <LuFeather class="w-6 h-6 text-[#9CAF88]" />
                  </div>
                  <div>
                    <div class="font-bold text-[#2D3748] text-lg">Your Story</div>
                    <div class="text-sm text-gray-600">Starts here</div>
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
            <WrapBalancer>
              <h2 class="text-4xl md:text-5xl font-bold text-[#2D3748] mb-6">
                Build something that lasts.
              </h2>
            </WrapBalancer>
            <WrapBalancer>
              <p class="text-xl text-gray-600 max-w-3xl mx-auto">
                The tools and space you need to craft work that endures, connects, and matters.
              </p>
            </WrapBalancer>
          </div>
          
          <div class="grid md:grid-cols-2 lg:grid-cols-3 gap-8">
            
            {/* Novel Management */}
            <div class="bg-white rounded-2xl p-8 shadow-lg hover:shadow-2xl transition-all duration-300 text-center group hover:-translate-y-2">
              <div class="w-20 h-20 bg-gradient-to-br from-[#9CAF88] to-[#7a8c6f] rounded-full flex items-center justify-center mx-auto mb-6 group-hover:scale-110 transition-transform">
                <LuFeather class="w-10 h-10 text-white" />
              </div>
              <h3 class="text-2xl font-bold text-[#2D3748] mb-4">Craft Your Novel</h3>
              <p class="text-gray-600 leading-relaxed mb-6">
                A canvas made for deep work—without distraction, with everything you need to shape the story only you can tell.
              </p>
              <div class="text-[#9CAF88] font-semibold">Organize • Plan • Create</div>
            </div>

            {/* Performance Analytics */}
            <div class="bg-white rounded-2xl p-8 shadow-lg hover:shadow-2xl transition-all duration-300 text-center group hover:-translate-y-2">
              <div class="w-20 h-20 bg-gradient-to-br from-[#9CAF88] to-[#7a8c6f] rounded-full flex items-center justify-center mx-auto mb-6 group-hover:scale-110 transition-transform">
                <LuBarChart3 class="w-10 h-10 text-white" />
              </div>
              <h3 class="text-2xl font-bold text-[#2D3748] mb-4">Track Your Book's Path</h3>
              <p class="text-gray-600 leading-relaxed mb-6">
                See where your work has been, and where readers are finding it. Understand what moves them, what keeps them reading.
              </p>
              <div class="text-[#9CAF88] font-semibold">Track • Analyze • Grow</div>
            </div>

            {/* Reader Connection */}
            <div class="bg-white rounded-2xl p-8 shadow-lg hover:shadow-2xl transition-all duration-300 text-center group hover:-translate-y-2">
              <div class="w-20 h-20 bg-gradient-to-br from-[#9CAF88] to-[#7a8c6f] rounded-full flex items-center justify-center mx-auto mb-6 group-hover:scale-110 transition-transform">
                <LuHeart class="w-10 h-10 text-white" />
              </div>
              <h3 class="text-2xl font-bold text-[#2D3748] mb-4">Grow Your Circle</h3>
              <p class="text-gray-600 leading-relaxed mb-6">
                Connect with readers and writers who care about story, not noise. Build relationships that matter.
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

      {/* Values Section - Writer Community */}
      <section class="py-20 bg-[#2D3748] text-white">
        <div class="max-w-6xl mx-auto px-4">
          <div class="text-center mb-12">
            <WrapBalancer>
              <h2 class="text-3xl md:text-4xl font-bold mb-4">Built for Writers Who Care About the Work</h2>
            </WrapBalancer>
            <WrapBalancer>
              <p class="text-xl text-gray-300">Where the first words matter as much as the last.</p>
            </WrapBalancer>
          </div>
          <div class="grid grid-cols-2 md:grid-cols-4 gap-8 text-center">
            <div class="group">
              <div class="w-20 h-20 bg-gradient-to-br from-[#9CAF88] to-[#7a8c6f] rounded-full flex items-center justify-center mx-auto mb-4 group-hover:scale-110 transition-transform">
                <LuFeather class="w-10 h-10 text-white" />
              </div>
              <div class="text-lg font-semibold text-gray-200">Craft-Focused</div>
              <div class="text-sm text-gray-300 mt-1">Tools built for writers who take their work seriously.</div>
            </div>
            <div class="group">
              <div class="w-20 h-20 bg-gradient-to-br from-[#9CAF88] to-[#7a8c6f] rounded-full flex items-center justify-center mx-auto mb-4 group-hover:scale-110 transition-transform">
                <LuHeart class="w-10 h-10 text-white" />
              </div>
              <div class="text-lg font-semibold text-gray-200">Reader Connection</div>
              <div class="text-sm text-gray-300 mt-1">Find the readers who truly appreciate your voice.</div>
            </div>
            <div class="group">
              <div class="w-20 h-20 bg-gradient-to-br from-[#9CAF88] to-[#7a8c6f] rounded-full flex items-center justify-center mx-auto mb-4 group-hover:scale-110 transition-transform">
                <LuGlobe class="w-10 h-10 text-white" />
              </div>
              <div class="text-lg font-semibold text-gray-200">Beautiful Presence</div>
              <div class="text-sm text-gray-300 mt-1">Author websites that honor the quality of your writing.</div>
            </div>
            <div class="group">
              <div class="w-20 h-20 bg-gradient-to-br from-[#9CAF88] to-[#7a8c6f] rounded-full flex items-center justify-center mx-auto mb-4 group-hover:scale-110 transition-transform">
                <LuShield class="w-10 h-10 text-white" />
              </div>
              <div class="text-lg font-semibold text-gray-200">Privacy First</div>
              <div class="text-sm text-gray-300 mt-1">Your work stays yours. No algorithms, no exploitation.</div>
            </div>
          </div>
        </div>
      </section>

      {/* Writer Success Stories */}
      <section class="py-20 bg-white">
        <div class="max-w-6xl mx-auto px-4">
          <div class="text-center mb-16">
            <WrapBalancer>
              <h2 class="text-4xl md:text-5xl font-bold text-[#2c1810] mb-6">What's Possible When You Begin</h2>
            </WrapBalancer>
            <WrapBalancer>
              <p class="text-xl text-gray-600 max-w-3xl mx-auto">
                Every finished story started exactly where you are now—with someone willing to begin.
              </p>
            </WrapBalancer>
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
                    <div class="font-bold text-[#2c1810]">5,000</div>
                    <div class="text-xs text-gray-600">Readers who stayed</div>
                  </div>
                </div>
              </div>
              
              <div class="absolute bottom-6 right-6 bg-white/95 backdrop-blur-sm rounded-xl shadow-xl p-4">
                <div class="flex items-center gap-3">
                  <div class="w-10 h-10 bg-[#d4af37]/10 rounded-full flex items-center justify-center">
                    <LuBookOpen class="w-5 h-5 text-[#d4af37]" />
                  </div>
                  <div>
                    <div class="font-bold text-[#2c1810]">3 Books</div>
                    <div class="text-xs text-gray-600">Finished</div>
                  </div>
                </div>
              </div>
            </div>
            
            {/* Right - Success Story */}
            <div>
              <div class="mb-8">
                <LuQuote class="w-12 h-12 text-[#d4af37] mb-4" />
                <WrapBalancer>
                  <blockquote class="text-2xl text-gray-700 font-light italic leading-relaxed mb-6">
                    "I used to get lost between notebooks, drafts, and ideas I couldn't finish. QuillSpace helped me build a system that didn't feel like a system—just space to write, return, and stay with it long enough to finish. Now three books are done, and somehow 5,000 readers are waiting for the next."
                  </blockquote>
                </WrapBalancer>
                <div class="flex items-center gap-4">
                  <img 
                    src="https://images.unsplash.com/photo-1438761681033-6461ffad8d80?ixlib=rb-4.0.3&auto=format&fit=crop&w=100&q=80" 
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
                    <h4 class="font-semibold text-[#2c1810] mb-1">From Chaos to Completion</h4>
                    <p class="text-gray-600">Turned years of stuck drafts into three finished novels.</p>
                  </div>
                </div>
                <div class="flex items-start gap-3">
                  <LuUsers class="w-6 h-6 text-[#d4af37] flex-shrink-0 mt-1" />
                  <div>
                    <h4 class="font-semibold text-[#2c1810] mb-1">Readers Who Stay</h4>
                    <p class="text-gray-600">Found a loyal audience who actually read—and return.</p>
                  </div>
                </div>
                <div class="flex items-start gap-3">
                  <LuTrendingUp class="w-6 h-6 text-[#d4af37] flex-shrink-0 mt-1" />
                  <div>
                    <h4 class="font-semibold text-[#2c1810] mb-1">Progress That Feels Real</h4>
                    <p class="text-gray-600">Watched her words take root, reader by reader.</p>
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
          <WrapBalancer>
            <h2 class="text-4xl md:text-5xl font-bold text-[#2c1810] mb-6">Start Where You Are. Stay With It.</h2>
          </WrapBalancer>
          <WrapBalancer>
            <p class="text-xl text-gray-600 mb-16 max-w-3xl mx-auto">
              From the first sentence to the final send, QuillSpace gives your writing a home.
            </p>
          </WrapBalancer>
          
          <div class="grid md:grid-cols-2 lg:grid-cols-4 gap-8">
            <div class="text-center group">
              <div class="w-20 h-20 bg-gradient-to-br from-[#d4af37] to-[#b8941f] rounded-full flex items-center justify-center mx-auto mb-6 group-hover:scale-110 transition-transform">
                <LuFeather class="w-10 h-10 text-white" />
              </div>
              <h3 class="text-xl font-bold text-[#2c1810] mb-3">Gather the Threads</h3>
              <p class="text-gray-600 leading-relaxed">
                All your characters, arcs, and ideas—kept close, clear, and ready to return to.
              </p>
            </div>
            
            <div class="text-center group">
              <div class="w-20 h-20 bg-gradient-to-br from-[#d4af37] to-[#b8941f] rounded-full flex items-center justify-center mx-auto mb-6 group-hover:scale-110 transition-transform">
                <LuPenTool class="w-10 h-10 text-white" />
              </div>
              <h3 class="text-xl font-bold text-[#2c1810] mb-3">Write at Your Pace. Return When You're Ready.</h3>
              <p class="text-gray-600 leading-relaxed">
                Set a rhythm that feels real—and stay close to the work without burning out.
              </p>
            </div>
            
            <div class="text-center group">
              <div class="w-20 h-20 bg-gradient-to-br from-[#d4af37] to-[#b8941f] rounded-full flex items-center justify-center mx-auto mb-6 group-hover:scale-110 transition-transform">
                <LuGlobe class="w-10 h-10 text-white" />
              </div>
              <h3 class="text-xl font-bold text-[#2c1810] mb-3">Make a Place for Your Voice</h3>
              <p class="text-gray-600 leading-relaxed">
                Create an author site that sounds like you—and gives your words somewhere to live.
              </p>
            </div>
            
            <div class="text-center group">
              <div class="w-20 h-20 bg-gradient-to-br from-[#d4af37] to-[#b8941f] rounded-full flex items-center justify-center mx-auto mb-6 group-hover:scale-110 transition-transform">
                <LuHeart class="w-10 h-10 text-white" />
              </div>
              <h3 class="text-xl font-bold text-[#2c1810] mb-3">Find the Readers Who Feel It Too</h3>
              <p class="text-gray-600 leading-relaxed">
                Connect with the ones who linger on your lines—and wait for what's next.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* Writer Testimonials Section */}
      <section class="py-20 bg-white">
        <div class="max-w-6xl mx-auto px-4 text-center">
          <h2 class="text-4xl md:text-5xl font-bold text-[#2c1810] mb-6">What Writers Need</h2>
          <p class="text-xl text-gray-600 mb-16 max-w-3xl mx-auto">
            It's not just the tools. It's what happens when writing stops feeling so alone.
          </p>
          
          <div class="grid md:grid-cols-2 lg:grid-cols-3 gap-8">
            <div class="bg-gradient-to-br from-amber-50 to-white p-8 rounded-2xl shadow-lg hover:shadow-xl transition-shadow">
              <div class="flex items-center mb-6">
                {[...Array(5)].map((_, i) => (
                  <LuStar key={i} class="w-5 h-5 text-[#d4af37] fill-current" />
                ))}
              </div>
              <p class="text-gray-700 mb-6 italic text-lg leading-relaxed">
                "Before QuillSpace, I had fifteen years of scattered notes—and no finished story. Now I've published three novels, and the readers keep showing up. I don't know what changed exactly—only that I finally stayed with the writing long enough to finish."
              </p>
              <div class="flex items-center gap-3">
                <img 
                  src="https://images.unsplash.com/photo-1438761681033-6461ffad8d80?ixlib=rb-4.0.3&auto=format&fit=crop&w=100&q=80" 
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
                "I used to write into the void—not knowing what landed or who was reading. Now I can see what chapters hit hardest. It's not just data—it's a quiet way to feel the echo."
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
                "All my work used to be scattered—some on social, some on sales pages. Now it lives in one place that actually feels like me. And my readers? They're not just finding the books—they're staying."
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
          
          <WrapBalancer>
            <h2 class="text-4xl md:text-6xl font-bold mb-8 leading-tight">
              The Writing's Yours.
              <span class="text-[#d4af37]"> The Space Is Ours.</span>
            </h2>
          </WrapBalancer>
          
          <WrapBalancer>
            <p class="text-xl md:text-2xl text-amber-200/90 mb-12 leading-relaxed max-w-4xl mx-auto">
              Some are just starting. Others are nearly done. All of them are still writing—right here.
              <br class="hidden md:block"/>
              <span class="text-[#d4af37] font-medium">Where words find warmth, and stories find their voice.</span>
            </p>
          </WrapBalancer>
          
          <div class="flex flex-col sm:flex-row gap-6 justify-center items-center">
            <a 
              href="/login"
              class="bg-[#d4af37] text-[#2c1810] px-10 py-5 rounded-xl font-bold text-xl hover:bg-[#b8941f] transition-all duration-300 shadow-2xl transform hover:scale-105 hover:shadow-[#d4af37]/25"
            >
              Step Into the Quiet
            </a>
            <a 
              href="#features" 
              class="border-2 border-[#d4af37] text-[#d4af37] px-10 py-5 rounded-xl font-semibold text-xl hover:bg-[#d4af37] hover:text-[#2c1810] transition-all duration-300 flex items-center gap-3"
            >
              <LuHeart class="w-6 h-6" />
              Take a Look Around
            </a>
          </div>
          
          <div class="mt-12 text-amber-200/70">
            <p class="text-lg">Early access is open. Join the writers who are starting something real.</p>
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
  title: "QuillSpace - Your Writing Sanctuary | Where Language Matters",
  meta: [
    {
      name: "description",
      content: "A quiet place to build your book, share your voice, and feel less alone on the way there. Where writers shape their stories, follow their books into the world, and connect with readers who read with soul.",
    },
    {
      name: "keywords",
      content: "writing sanctuary, novel management, author platform, book performance tracking, reader connection, writing organization, author (website-builder) builder, creative writing tools, manuscript management, writer community",
    },
    {
      property: "og:title",
      content: "QuillSpace - Your Writing Sanctuary | Where Language Matters",
    },
    {
      property: "og:description", 
      content: "A quiet place to build your book, share your voice, and feel less alone on the way there. Where language matters, and stories are given the room to breathe.",
    },
    {
      property: "og:image",
      content: "https://images.unsplash.com/photo-1455390582262-044cdead277a?ixlib=rb-4.0.3&auto=format&fit=crop&w=1200&q=80",
    },
    {
      property: "og:type",
      content: "(website-builder)",
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
      content: "A quiet place to build your book, share your voice, and feel less alone on the way there. Where language matters, and stories are given the room to breathe.",
    },
  ],
};
