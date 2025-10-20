import {component$, useSignal, useTask$, $} from "@builder.io/qwik";
import {AuthorProfile} from "~/api/schema";


export interface AuthorEditorProps {
    authors: AuthorProfile[]
}

export default component$<AuthorEditorProps>((props) => {
    const authors = useSignal<AuthorProfile[]>([]);
    const editingAuthor = useSignal<AuthorProfile | null>(null);
    const isEditing = useSignal(false);
    
    useTask$(({track}) => {
        track(()=>props.authors);
        authors.value = props.authors
    })

    const startEditing = $((author: AuthorProfile) => {
        editingAuthor.value = { ...author }; // Create a copy for editing
        isEditing.value = true;
    });

    const cancelEditing = $(() => {
        editingAuthor.value = null;
        isEditing.value = false;
    });

    const saveChanges = $(() => {
        if (editingAuthor.value) {
            // Find and update the author in the array
            const index = authors.value.findIndex(a => a._id === editingAuthor.value!._id);
            if (index !== -1) {
                authors.value[index] = { ...editingAuthor.value };
                authors.value = [...authors.value]; // Trigger reactivity
            }
            // TODO: Call API to save changes to backend
            console.log('Saving author changes:', editingAuthor.value);
        }
        cancelEditing();
    });

    const stripHtmlTags = (html: string): string => {
        return html.replace(/<[^>]*>/g, '').replace(/&nbsp;/g, ' ').trim();
    };

    return (
        <div class="space-y-4">
            <div class="text-sm font-medium text-gray-700 mb-3">Author Profile</div>

            {authors.value.map((author) => (
                <div key={author._id} class="space-y-3 p-4 border border-gray-200 rounded-lg">
                    {/* Author Name */}
                    <div>
                        <label class="block text-xs font-medium text-gray-600 mb-1">Author Name</label>
                        <input
                            type="text"
                            value={isEditing.value && editingAuthor.value?._id === author._id 
                                ? editingAuthor.value.name 
                                : author.name}
                            onInput$={(e) => {
                                if (isEditing.value && editingAuthor.value?._id === author._id) {
                                    editingAuthor.value = {
                                        ...editingAuthor.value,
                                        name: (e.target as HTMLInputElement).value
                                    };
                                }
                            }}
                            disabled={!isEditing.value || editingAuthor.value?._id !== author._id}
                            class="w-full px-3 py-2 text-sm border border-gray-300 rounded focus:outline-none focus:ring-[0.5px] focus:ring-[#9CAF88]/30 focus:border-[#9CAF88] disabled:bg-gray-50"
                        />
                    </div>

                    {/* Tagline */}
                    <div>
                        <label class="block text-xs font-medium text-gray-600 mb-1">Tagline</label>
                        <input
                            type="text"
                            value={isEditing.value && editingAuthor.value?._id === author._id 
                                ? editingAuthor.value.tagline 
                                : author.tagline}
                            onInput$={(e) => {
                                if (isEditing.value && editingAuthor.value?._id === author._id) {
                                    editingAuthor.value = {
                                        ...editingAuthor.value,
                                        tagline: (e.target as HTMLInputElement).value
                                    };
                                }
                            }}
                            disabled={!isEditing.value || editingAuthor.value?._id !== author._id}
                            class="w-full px-3 py-2 text-sm border border-gray-300 rounded focus:outline-none focus:ring-[0.5px] focus:ring-[#9CAF88]/30 focus:border-[#9CAF88] disabled:bg-gray-50"
                        />
                    </div>

                    {/* Email */}
                    <div>
                        <label class="block text-xs font-medium text-gray-600 mb-1">Email</label>
                        <input
                            type="email"
                            value={isEditing.value && editingAuthor.value?._id === author._id 
                                ? editingAuthor.value.email 
                                : author.email}
                            onInput$={(e) => {
                                if (isEditing.value && editingAuthor.value?._id === author._id) {
                                    editingAuthor.value = {
                                        ...editingAuthor.value,
                                        email: (e.target as HTMLInputElement).value
                                    };
                                }
                            }}
                            disabled={!isEditing.value || editingAuthor.value?._id !== author._id}
                            class="w-full px-3 py-2 text-sm border border-gray-300 rounded focus:outline-none focus:ring-[0.5px] focus:ring-[#9CAF88]/30 focus:border-[#9CAF88] disabled:bg-gray-50"
                        />
                    </div>

                    {/* Phone */}
                    <div>
                        <label class="block text-xs font-medium text-gray-600 mb-1">Phone</label>
                        <input
                            type="tel"
                            value={isEditing.value && editingAuthor.value?._id === author._id 
                                ? editingAuthor.value.phone || '' 
                                : author.phone || ''}
                            onInput$={(e) => {
                                if (isEditing.value && editingAuthor.value?._id === author._id) {
                                    editingAuthor.value = {
                                        ...editingAuthor.value,
                                        phone: (e.target as HTMLInputElement).value
                                    };
                                }
                            }}
                            disabled={!isEditing.value || editingAuthor.value?._id !== author._id}
                            class="w-full px-3 py-2 text-sm border border-gray-300 rounded focus:outline-none focus:ring-[0.5px] focus:ring-[#9CAF88]/30 focus:border-[#9CAF88] disabled:bg-gray-50"
                        />
                    </div>

                    {/* Address */}
                    <div>
                        <label class="block text-xs font-medium text-gray-600 mb-1">Address</label>
                        <input
                            type="text"
                            value={isEditing.value && editingAuthor.value?._id === author._id 
                                ? editingAuthor.value.address || '' 
                                : author.address || ''}
                            onInput$={(e) => {
                                if (isEditing.value && editingAuthor.value?._id === author._id) {
                                    editingAuthor.value = {
                                        ...editingAuthor.value,
                                        address: (e.target as HTMLInputElement).value
                                    };
                                }
                            }}
                            disabled={!isEditing.value || editingAuthor.value?._id !== author._id}
                            class="w-full px-3 py-2 text-sm border border-gray-300 rounded focus:outline-none focus:ring-[0.5px] focus:ring-[#9CAF88]/30 focus:border-[#9CAF88] disabled:bg-gray-50"
                        />
                    </div>

                    {/* Bio */}
                    <div>
                        <label class="block text-xs font-medium text-gray-600 mb-1">Bio</label>
                        <textarea
                            rows={4}
                            value={isEditing.value && editingAuthor.value?._id === author._id 
                                ? stripHtmlTags(editingAuthor.value.bio) 
                                : stripHtmlTags(author.bio)}
                            onInput$={(e) => {
                                if (isEditing.value && editingAuthor.value?._id === author._id) {
                                    editingAuthor.value = {
                                        ...editingAuthor.value,
                                        bio: (e.target as HTMLTextAreaElement).value
                                    };
                                }
                            }}
                            disabled={!isEditing.value || editingAuthor.value?._id !== author._id}
                            placeholder="Author biography..."
                            class="w-full px-3 py-2 text-sm border border-gray-300 rounded focus:outline-none focus:ring-[0.5px] focus:ring-[#9CAF88]/30 focus:border-[#9CAF88] disabled:bg-gray-50"
                        />
                    </div>

                    {/* Social Media Links */}
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                        <div>
                            <label class="block text-xs font-medium text-gray-600 mb-1">Twitter/X</label>
                            <input
                                type="url"
                                value={isEditing.value && editingAuthor.value?._id === author._id 
                                    ? editingAuthor.value.x 
                                    : author.x}
                                onInput$={(e) => {
                                    if (isEditing.value && editingAuthor.value?._id === author._id) {
                                        editingAuthor.value = {
                                            ...editingAuthor.value,
                                            x: (e.target as HTMLInputElement).value
                                        };
                                    }
                                }}
                                disabled={!isEditing.value || editingAuthor.value?._id !== author._id}
                                placeholder="https://x.com/username"
                                class="w-full px-3 py-2 text-sm border border-gray-300 rounded focus:outline-none focus:ring-[0.5px] focus:ring-[#9CAF88]/30 focus:border-[#9CAF88] disabled:bg-gray-50"
                            />
                        </div>
                        <div>
                            <label class="block text-xs font-medium text-gray-600 mb-1">Facebook</label>
                            <input
                                type="url"
                                value={isEditing.value && editingAuthor.value?._id === author._id 
                                    ? editingAuthor.value.facebook || '' 
                                    : author.facebook || ''}
                                onInput$={(e) => {
                                    if (isEditing.value && editingAuthor.value?._id === author._id) {
                                        editingAuthor.value = {
                                            ...editingAuthor.value,
                                            facebook: (e.target as HTMLInputElement).value
                                        };
                                    }
                                }}
                                disabled={!isEditing.value || editingAuthor.value?._id !== author._id}
                                placeholder="https://facebook.com/username"
                                class="w-full px-3 py-2 text-sm border border-gray-300 rounded focus:outline-none focus:ring-[0.5px] focus:ring-[#9CAF88]/30 focus:border-[#9CAF88] disabled:bg-gray-50"
                            />
                        </div>
                    </div>

                    {/* Action Buttons */}
                    <div class="flex gap-2 pt-2">
                        {isEditing.value && editingAuthor.value?._id === author._id ? (
                            <>
                                <button
                                    onClick$={saveChanges}
                                    class="px-3 py-1 bg-[#9CAF88] text-white text-xs rounded hover:bg-[#8BA079] transition-colors">
                                    Save Changes
                                </button>
                                <button
                                    onClick$={cancelEditing}
                                    class="px-3 py-1 bg-gray-200 text-gray-700 text-xs rounded hover:bg-gray-300 transition-colors">
                                    Cancel
                                </button>
                            </>
                        ) : (
                            <button
                                onClick$={() => startEditing(author)}
                                class="px-3 py-1 bg-[#9CAF88] text-white text-xs rounded hover:bg-[#8BA079] transition-colors">
                                Edit Profile
                            </button>
                        )}
                    </div>
                </div>
            ))}
        </div>
    );
})