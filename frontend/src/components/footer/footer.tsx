import { component$ } from '@builder.io/qwik';
import { Link } from '@builder.io/qwik-city';
import { LuRocket } from '@qwikest/icons/lucide';
import { SiGithub, SiTwitter, SiLinkedin } from '@qwikest/icons/simpleicons';

export default component$(() => {
  return (
    <footer class="bg-gray-900 text-white">
      <div class="container mx-auto px-4 py-12">
        <div class="grid grid-cols-1 md:grid-cols-4 gap-8">
          {/* Brand */}
          <div class="space-y-4">
            <div class="flex items-center gap-2">
              <LuRocket class="w-6 h-6 text-blue-400" />
              <span class="text-xl font-bold">QuillSpace</span>
            </div>
            <p class="text-gray-400">
              High-performance multi-tenant publishing platform built with modern technology.
            </p>
            <div class="flex gap-4">
              <a href="#" class="text-gray-400 hover:text-white transition-colors">
                <SiGithub class="w-5 h-5" />
              </a>
              <a href="#" class="text-gray-400 hover:text-white transition-colors">
                <SiTwitter class="w-5 h-5" />
              </a>
              <a href="#" class="text-gray-400 hover:text-white transition-colors">
                <SiLinkedin class="w-5 h-5" />
              </a>
            </div>
          </div>

          {/* Product */}
          <div class="space-y-4">
            <h3 class="text-lg font-semibold">Product</h3>
            <div class="space-y-2">
              <Link href="/features" class="block text-gray-400 hover:text-white transition-colors">
                Features
              </Link>
              <Link href="/pricing" class="block text-gray-400 hover:text-white transition-colors">
                Pricing
              </Link>
              <Link href="/integrations" class="block text-gray-400 hover:text-white transition-colors">
                Integrations
              </Link>
              <Link href="/api" class="block text-gray-400 hover:text-white transition-colors">
                API
              </Link>
            </div>
          </div>

          {/* Resources */}
          <div class="space-y-4">
            <h3 class="text-lg font-semibold">Resources</h3>
            <div class="space-y-2">
              <Link href="/docs" class="block text-gray-400 hover:text-white transition-colors">
                Documentation
              </Link>
              <Link href="/blog" class="block text-gray-400 hover:text-white transition-colors">
                Blog
              </Link>
              <Link href="/tutorials" class="block text-gray-400 hover:text-white transition-colors">
                Tutorials
              </Link>
              <Link href="/support" class="block text-gray-400 hover:text-white transition-colors">
                Support
              </Link>
            </div>
          </div>

          {/* Company */}
          <div class="space-y-4">
            <h3 class="text-lg font-semibold">Company</h3>
            <div class="space-y-2">
              <Link href="/about" class="block text-gray-400 hover:text-white transition-colors">
                About
              </Link>
              <Link href="/careers" class="block text-gray-400 hover:text-white transition-colors">
                Careers
              </Link>
              <Link href="/contact" class="block text-gray-400 hover:text-white transition-colors">
                Contact
              </Link>
              <Link href="/privacy" class="block text-gray-400 hover:text-white transition-colors">
                Privacy
              </Link>
            </div>
          </div>
        </div>

        <div class="border-t border-gray-800 mt-8 pt-8 flex flex-col md:flex-row justify-between items-center">
          <p class="text-gray-400 text-sm">
            © 2024 QuillSpace. Built with Rust, Qwik, and ClickHouse.
          </p>
          <div class="flex gap-6 mt-4 md:mt-0">
            <Link href="/terms" class="text-gray-400 hover:text-white text-sm transition-colors">
              Terms
            </Link>
            <Link href="/privacy" class="text-gray-400 hover:text-white text-sm transition-colors">
              Privacy
            </Link>
            <Link href="/cookies" class="text-gray-400 hover:text-white text-sm transition-colors">
              Cookies
            </Link>
          </div>
        </div>
      </div>
    </footer>
  );
});
