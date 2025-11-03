import { component$ } from '@builder.io/qwik';
import { DocumentHead } from '@builder.io/qwik-city';

export default component$(() => {
  return (
    <div class="max-w-6xl mx-auto space-y-8">
      <div class="text-center py-12">
        <h1 class="text-4xl font-serif font-bold text-[#2D3748] mb-4">
          Settings
        </h1>
        <p class="text-lg text-gray-600 max-w-2xl mx-auto">
          Customize your QuillSpace experience and manage your account preferences.
        </p>
      </div>
      
      <div class="bg-[#FEFCF7] rounded-xl border border-[#E8E2D4] p-8 text-center">
        <h2 class="text-2xl font-serif font-semibold text-[#2D3748] mb-4">
          Coming Soon
        </h2>
        <p class="text-gray-600">
          User settings and preferences are currently under development. Stay tuned for updates!
        </p>
      </div>
    </div>
  );
});

export const head: DocumentHead = {
  title: 'Settings - QuillSpace',
  meta: [
    {
      name: 'description',
      content: 'Manage your QuillSpace account settings and preferences.',
    },
  ],
};
