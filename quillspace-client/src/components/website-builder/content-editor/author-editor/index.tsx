import {component$, useSignal, useTask$, $} from "@builder.io/qwik";
import {AuthorProfile, AuthorProfileSchema} from "~/api/schema";
import {routeAction$, Form, zod$, z} from "@builder.io/qwik-city";


export interface AuthorEditorProps {
    authors: AuthorProfile[]
}

export const useUpdateAuthor = routeAction$(async (data, requestEvent) => {
    try {
        // TODO: Replace with actual API call to your backend
        console.log('Updating author:', data);
        
        // Example API call structure:
        // const response = await fetch(`${API_BASE_URL}/api/authors/${data.authorId}`, {
        //     method: 'PUT',
        //     headers: {
        //         'Content-Type': 'application/json',
        //         'Authorization': `Bearer ${token}`
        //     },
        //     body: JSON.stringify(data)
        // });
        
        // if (!response.ok) {
        //     throw new Error('Failed to update author');
        // }
        
        return {
            success: true,
            message: 'Author updated successfully'
        };
    } catch (error) {
        console.error('Error updating author:', error);
        return {
            success: false,
            message: 'Failed to update author'
        };
    }
}, zod$(
    AuthorProfileSchema.omit({
        _id: true,
        _owner: true,
        _createdDate: true,
        _updatedDate: true
    }).extend({
        authorId: z.string()
    })
));

export default component$<AuthorEditorProps>((props) => {
    const authors = useSignal<AuthorProfile[]>([]);
    const editingAuthor = useSignal<AuthorProfile | null>(null);
    const isEditing = useSignal(false);
    const updateAuthorAction = useUpdateAuthor();
    
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

    const handleFormSubmit = $(() => {
        // Form submission is handled by the action
        // Update local state after successful submission
        if (editingAuthor.value) {
            const index = authors.value.findIndex(a => a._id === editingAuthor.value!._id);
            if (index !== -1) {
                authors.value[index] = { ...editingAuthor.value };
                authors.value = [...authors.value]; // Trigger reactivity
            }
        }
        cancelEditing();
    });

    // Watch for action completion
    useTask$(({ track }) => {
        track(() => updateAuthorAction.value);
        if (updateAuthorAction.value?.success) {
            // Action completed successfully
            handleFormSubmit();
        }
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
                    <div class="flex justify-end space-x-2 pt-2">
                        {isEditing.value && editingAuthor.value?._id === author._id ? (
                            <Form action={updateAuthorAction} class="flex space-x-2">
                                {/* Hidden fields for form data */}
                                <input type="hidden" name="authorId" value={author._id} />
                                <input type="hidden" name="name" value={editingAuthor.value?.name || ''} />
                                <input type="hidden" name="tagline" value={editingAuthor.value?.tagline || ''} />
                                <input type="hidden" name="email" value={editingAuthor.value?.email || ''} />
                                <input type="hidden" name="phone" value={editingAuthor.value?.phone || ''} />
                                <input type="hidden" name="address" value={editingAuthor.value?.address || ''} />
                                <input type="hidden" name="bio" value={stripHtmlTags(editingAuthor.value?.bio || '')} />
                                <input type="hidden" name="x" value={editingAuthor.value?.x || ''} />
                                <input type="hidden" name="facebook" value={editingAuthor.value?.facebook || ''} />
                                <input type="hidden" name="instagram" value={editingAuthor.value?.instagram || ''} />
                                <input type="hidden" name="portraitImage" value={editingAuthor.value?.portraitImage || ''} />
                                
                                <button
                                    type="button"
                                    onClick$={cancelEditing}
                                    class="px-3 py-1.5 text-xs font-medium text-gray-600 bg-gray-100 rounded hover:bg-gray-200 transition-colors"
                                >
                                    Cancel
                                </button>
                                <button
                                    type="submit"
                                    disabled={updateAuthorAction.isRunning}
                                    class="px-3 py-1.5 text-xs font-medium text-white bg-[#9CAF88] rounded hover:bg-[#8BA077] transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                                >
                                    {updateAuthorAction.isRunning ? 'Saving...' : 'Save Changes'}
                                </button>
                            </Form>
                        ) : (
                            <button
                                type="button"
                                onClick$={() => startEditing(author)}
                                class="px-3 py-1.5 text-xs font-medium text-[#9CAF88] bg-[#9CAF88]/10 rounded hover:bg-[#9CAF88]/20 transition-colors"
                            >
                                Edit Profile
                            </button>
                        )}
                    </div>

                    {/* Display action feedback */}
                    {updateAuthorAction.value && (
                        <div class={`mt-2 p-2 text-xs rounded ${
                            updateAuthorAction.value.success 
                                ? 'bg-green-50 text-green-700 border border-green-200'
                                : 'bg-red-50 text-red-700 border border-red-200'
                        }`}>
                            {updateAuthorAction.value.message}
                        </div>
                    )}
                </div>
            ))}
        </div>
    );
})