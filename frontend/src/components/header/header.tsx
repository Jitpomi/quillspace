import { component$, useSignal } from '@builder.io/qwik';
import { Link } from '@builder.io/qwik-city';
import { LuMenu, LuX, LuRocket } from '@qwikest/icons/lucide';

export default component$(() => {
  const isMenuOpen = useSignal(false);

  return (
    <header class="bg-white shadow-sm border-b border-gray-200">
      <nav class="container mx-auto px-4 py-4">
        <div class="flex justify-between items-center">
          {/* Logo */}
          <Link href="/" class="flex items-center gap-2 text-xl font-bold text-gray-900">
            <LuRocket class="w-6 h-6 text-blue-600" />
            QuillSpace
          </Link>

          {/* Desktop Navigation */}
          <div class="hidden md:flex items-center gap-8">
            <Link href="/dashboard" class="text-gray-600 hover:text-gray-900 transition-colors">
              Dashboard
            </Link>
            <Link href="/content" class="text-gray-600 hover:text-gray-900 transition-colors">
              Content
            </Link>
            <Link href="/analytics" class="text-gray-600 hover:text-gray-900 transition-colors">
              Analytics
            </Link>
            <Link href="/settings" class="text-gray-600 hover:text-gray-900 transition-colors">
              Settings
            </Link>
            <Link 
              href="/login" 
              class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg transition-colors"
            >
              Sign In
            </Link>
          </div>

          {/* Mobile Menu Button */}
          <button
            class="md:hidden p-2"
            onClick$={() => (isMenuOpen.value = !isMenuOpen.value)}
          >
            {isMenuOpen.value ? (
              <LuX class="w-6 h-6" />
            ) : (
              <LuMenu class="w-6 h-6" />
            )}
          </button>
        </div>

        {/* Mobile Navigation */}
        {isMenuOpen.value && (
          <div class="md:hidden mt-4 pb-4 border-t border-gray-200">
            <div class="flex flex-col gap-4 pt-4">
              <Link 
                href="/dashboard" 
                class="text-gray-600 hover:text-gray-900 transition-colors"
                onClick$={() => (isMenuOpen.value = false)}
              >
                Dashboard
              </Link>
              <Link 
                href="/content" 
                class="text-gray-600 hover:text-gray-900 transition-colors"
                onClick$={() => (isMenuOpen.value = false)}
              >
                Content
              </Link>
              <Link 
                href="/analytics" 
                class="text-gray-600 hover:text-gray-900 transition-colors"
                onClick$={() => (isMenuOpen.value = false)}
              >
                Analytics
              </Link>
              <Link 
                href="/settings" 
                class="text-gray-600 hover:text-gray-900 transition-colors"
                onClick$={() => (isMenuOpen.value = false)}
              >
                Settings
              </Link>
              <Link 
                href="/login" 
                class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg transition-colors text-center"
                onClick$={() => (isMenuOpen.value = false)}
              >
                Sign In
              </Link>
            </div>
          </div>
        )}
      </nav>
    </header>
  );
});
