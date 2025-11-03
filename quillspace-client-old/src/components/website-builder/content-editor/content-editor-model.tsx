import {component$, useSignal} from "@builder.io/qwik";
import BooksEditor from "~/components/website-builder/content-editor/books-editor";
import AuthorEditor from "~/components/website-builder/content-editor/author-editor";
import {AuthorProfile, Book} from "~/api/schema";

export interface ContentEditorModelProps {
    books: Book[]
    authors: AuthorProfile[]
    updateAuthorAction: any // Action from route
}

export default component$<ContentEditorModelProps>((props)=> {

    const expandedAccordion = useSignal<string | null>(null);
    
    return (
        <div
            class="relative bg-white rounded-2xl shadow-lg border border-gray-100 w-[800px] max-h-[80vh] overflow-hidden">
            {/* Header */}
            <div class="px-8 py-6 border-b border-gray-50 bg-gradient-to-r from-gray-50/50 to-[#9CAF88]/5">
                <div class="flex items-center gap-3">
                    <div class="w-10 h-10 rounded-full bg-[#9CAF88]/10 flex items-center justify-center">
                        <svg class="w-5 h-5 text-[#9CAF88]" fill="currentColor" viewBox="0 0 24 24">
                            <path
                                d="M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zM20.71 7.04c.39-.39.39-1.02 0-1.41l-2.34-2.34c-.39-.39-1.02-.39-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z"/>
                        </svg>
                    </div>
                    <div>
                        <h2 class="text-xl font-medium text-gray-800">Editing Wizard</h2>
                        <p class="text-sm text-gray-500">Manage your website content</p>
                    </div>
                </div>
            </div>

            {/* Content */}
            <div class="p-8 overflow-y-auto max-h-[calc(80vh-120px)]">
                <div class="space-y-4">
                    {/* Books Accordion */}
                    <div class="bg-white border border-gray-100 rounded-xl shadow-sm hover:shadow-md hover:border-[#9CAF88]/30 transition-all duration-200">
                        <button
                            class="w-full flex items-center justify-between p-5 text-left hover:bg-[#9CAF88]/5 transition-colors rounded-xl cursor-pointer"
                            onClick$={() => {
                                expandedAccordion.value = expandedAccordion.value === 'books' ? null : 'books';
                            }}
                        >
                            <div class="flex items-center gap-3">
                                <span class="text-xl">📚</span>
                                <div>
                                    <div class="font-medium text-gray-900 text-sm">Books Management</div>
                                    <div class="text-xs text-gray-500 mt-0.5">Show/hide books, edit details, set
                                        featured books
                                    </div>
                                </div>
                            </div>
                            <svg
                                class={`w-4 h-4 text-gray-400 transition-transform ${
                                    expandedAccordion.value === 'books' ? 'rotate-90' : ''
                                }`}
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                      d="M9 5l7 7-7 7"></path>
                            </svg>
                        </button>

                        {/* Books Content */}
                        {expandedAccordion.value === 'books' && (
                            <div class="border-t border-gray-200 p-4 bg-gray-50">
                                <BooksEditor books={props.books}/>
                            </div>
                        )}
                    </div>

                    {/* Author Accordion */}
                    <div class="bg-white border border-gray-100 rounded-xl shadow-sm hover:shadow-md hover:border-[#9CAF88]/30 transition-all duration-200">
                        <button
                            class="w-full flex items-center justify-between p-5 text-left hover:bg-[#9CAF88]/5 transition-colors rounded-xl cursor-pointer"
                            onClick$={() => {
                                expandedAccordion.value = expandedAccordion.value === 'author' ? null : 'author';
                            }}
                        >
                            <div class="flex items-center gap-3">
                                <span class="text-xl">👤</span>
                                <div>
                                    <div class="font-medium text-gray-900 text-sm">Author Information</div>
                                    <div class="text-xs text-gray-500 mt-0.5">Update name, bio, photo, and contact
                                        information
                                    </div>
                                </div>
                            </div>
                            <svg
                                class={`w-4 h-4 text-gray-400 transition-transform ${
                                    expandedAccordion.value === 'author' ? 'rotate-90' : ''
                                }`}
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                      d="M9 5l7 7-7 7"></path>
                            </svg>
                        </button>

                        {/* Author Content */}
                        {expandedAccordion.value === 'author' && (
                            <div class="border-t border-gray-200 p-4 bg-gray-50">
                                <AuthorEditor authors={props.authors} updateAuthorAction={props.updateAuthorAction}/>
                            </div>
                        )}
                    </div>

                    {/* Theme Accordion - Coming Soon */}
                    <div class="border border-gray-200 rounded-lg bg-gray-50">
                        <div class="w-full flex items-center justify-between p-4 text-left rounded-lg">
                            <div class="flex items-center gap-3">
                                <span class="text-xl opacity-50">🎨</span>
                                <div>
                                    <div class="font-medium text-gray-500 text-sm">Theme & Styling</div>
                                    <div class="text-xs text-gray-400 mt-0.5">Customize colors and fonts - Coming
                                        soon
                                    </div>
                                </div>
                            </div>
                            <svg class="w-4 h-4 text-gray-300" fill="none" stroke="currentColor"
                                 viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                      d="M9 5l7 7-7 7"></path>
                            </svg>
                        </div>
                    </div>

                    {/* Copy Accordion - Coming Soon */}
                    <div class="border border-gray-200 rounded-lg bg-gray-50">
                        <div class="w-full flex items-center justify-between p-4 text-left rounded-lg">
                            <div class="flex items-center gap-3">
                                <span class="text-xl opacity-50">✏️</span>
                                <div>
                                    <div class="font-medium text-gray-500 text-sm">Advanced Editing</div>
                                    <div class="text-xs text-gray-400 mt-0.5">Visual page editing - Coming soon
                                    </div>
                                </div>
                            </div>
                            <svg class="w-4 h-4 text-gray-300" fill="none" stroke="currentColor"
                                 viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                      d="M9 5l7 7-7 7"></path>
                            </svg>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    )
})