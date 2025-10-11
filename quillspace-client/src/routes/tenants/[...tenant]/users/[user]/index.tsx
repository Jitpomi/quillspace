import { component$ } from "@builder.io/qwik";
import {LuBarChart3, LuFileText, LuUsers} from "@qwikest/icons/lucide";

export default component$(() => {
    return (
        <div class="max-w-6xl mx-auto space-y-8">
              {/* Writing Desk Welcome */}
              <div class="text-center py-8">
                <h2 class="text-4xl font-serif font-semibold text-[#2D3748] mb-3 leading-tight">Writing Desk</h2>
                <p class="text-lg font-sans text-gray-600 leading-relaxed max-w-2xl mx-auto">Your creative command center. See your progress, manage your work, and connect with your readers.</p>
              </div>

              {/* Quick Actions - Ultra Clean */}
              <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                <div 
                  class="bg-[#FEFCF7] rounded-xl border border-[#E8E2D4] p-8 text-center shadow-warm hover-lift transition-gentle cursor-pointer group"
                >
                  <div class="w-16 h-16 bg-[#9CAF88]/20 rounded-full flex items-center justify-center mx-auto mb-4 group-hover:bg-[#9CAF88]/30 transition-soft animate-breathe">
                    <LuFileText class="w-8 h-8 text-[#2D3748] group-hover:scale-110 transition-soft" />
                  </div>
                  <h3 class="text-xl font-serif font-semibold text-[#2D3748] mb-2">My Writing</h3>
                  <p class="font-sans text-gray-600 mb-4 leading-relaxed">Access your documents, drafts, and works in progress. Your stories await.</p>
                  <div class="text-[#9CAF88] font-medium font-sans">Open Writing →</div>
                </div>

                <div class="bg-[#FEFCF7] rounded-xl border border-[#E8E2D4] p-8 text-center shadow-warm hover-lift transition-gentle cursor-pointer group">
                  <div class="w-16 h-16 bg-[#7C9CBF]/20 rounded-full flex items-center justify-center mx-auto mb-4 group-hover:bg-[#7C9CBF]/30 transition-soft">
                    <LuUsers class="w-8 h-8 text-[#2D3748] group-hover:scale-110 transition-soft" />
                  </div>
                  <h3 class="text-xl font-serif font-semibold text-[#2D3748] mb-2">My Website</h3>
                  <p class="font-sans text-gray-600 mb-4 leading-relaxed">Build and customize your author platform. Share your voice with the world.</p>
                  <div class="text-[#7C9CBF] font-medium font-sans">Manage Site →</div>
                </div>

                <div class="bg-[#FEFCF7] rounded-xl border border-[#E8E2D4] p-8 text-center shadow-warm hover-lift transition-gentle cursor-pointer group">
                  <div class="w-16 h-16 bg-[#B8A9C9]/20 rounded-full flex items-center justify-center mx-auto mb-4 group-hover:bg-[#B8A9C9]/30 transition-soft">
                    <LuBarChart3 class="w-8 h-8 text-[#2D3748] group-hover:scale-110 transition-soft" />
                  </div>
                  <h3 class="text-xl font-serif font-semibold text-[#2D3748] mb-2">Readers</h3>
                  <p class="font-sans text-gray-600 mb-4 leading-relaxed">Connect with the souls who find meaning in your words and return for more.</p>
                  <div class="text-[#B8A9C9] font-medium font-sans">View Readers →</div>
                </div>
              </div>

              {/* Writing Progress */}
              <div class="bg-[#F7F3E9] rounded-xl border border-[#E8E2D4] p-8 shadow-warm">
                <h3 class="text-2xl font-serif font-semibold text-[#2D3748] mb-2 text-center">Your Writing Progress</h3>
                <p class="text-center text-gray-600 font-sans mb-6">Every word you write is a step forward. Every story shared is a gift to the world.</p>
                <div class="grid grid-cols-2 md:grid-cols-4 gap-8 text-center">
                  <div class="group cursor-pointer transition-gentle hover-lift">
                    <div class="text-4xl font-serif font-bold text-[#9CAF88] mb-1 group-hover:scale-110 transition-soft">12</div>
                    <div class="text-sm font-sans text-gray-600 font-medium">Words Written</div>
                    <div class="text-xs text-gray-500 mt-1">This Month</div>
                  </div>
                  <div class="group cursor-pointer transition-gentle hover-lift">
                    <div class="text-4xl font-serif font-bold text-[#7C9CBF] mb-1 group-hover:scale-110 transition-soft">2.3k</div>
                    <div class="text-sm font-sans text-gray-600 font-medium">Stories in Progress</div>
                    <div class="text-xs text-gray-500 mt-1">Drafts</div>
                  </div>
                  <div class="group cursor-pointer transition-gentle hover-lift">
                    <div class="text-4xl font-serif font-bold text-[#B8A9C9] mb-1 group-hover:scale-110 transition-soft">156</div>
                    <div class="text-sm font-sans text-gray-600 font-medium">Writing Sessions</div>
                    <div class="text-xs text-gray-500 mt-1">This Week</div>
                  </div>
                  <div class="group cursor-pointer transition-gentle hover-lift">
                    <div class="text-4xl font-serif font-bold text-[#2D3748] mb-1 group-hover:scale-110 transition-soft">4.2m</div>
                    <div class="text-sm font-sans text-gray-600 font-medium">Hours at the Desk</div>
                    <div class="text-xs text-gray-500 mt-1">Creating</div>
                  </div>
                </div>
              </div>
            </div>
          
    );
});