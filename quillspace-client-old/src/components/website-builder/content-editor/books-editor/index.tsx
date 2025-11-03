import {component$, useSignal, useTask$} from "@builder.io/qwik";
import {Book} from "~/api/schema";


export interface BooksEditorProps {
    books: Book[]
}

export default component$<BooksEditorProps>((props) => {
    const books = useSignal<Book[]>([]);

    useTask$(({track}) => {
        track(()=>props.books);
        books.value = props.books
    })
    
    return (
        <div class="space-y-3">
            <div class="text-sm font-medium text-gray-700 mb-3">Manage Your Books</div>

            {/* Sortable Book List */}
            <div class="space-y-2">
                {books.value.map((book, index) => (
                    <div
                        key={book._id}
                        draggable
                        class="flex items-center gap-3 p-3 bg-white rounded border hover:border-[#9CAF88]/30 transition-colors cursor-move"
                        onDragStart$={(e) => {
                            e.dataTransfer!.setData('text/plain', index.toString());
                            e.dataTransfer!.effectAllowed = 'move';
                        }}
                        onDragOver$={(e) => {
                            e.preventDefault();
                            e.dataTransfer!.dropEffect = 'move';
                        }}
                        onDrop$={(e) => {
                            e.preventDefault();
                            const draggedIndex = parseInt(e.dataTransfer!.getData('text/plain'));
                            const targetIndex = index;

                            if (draggedIndex !== targetIndex) {
                                const newBooks = [...books.value];
                                const [draggedBook] = newBooks.splice(draggedIndex, 1);
                                newBooks.splice(targetIndex, 0, draggedBook);
                                books.value = newBooks;
                            }
                        }}
                    >
                        {/* Drag Handle */}
                        <div
                            class="text-gray-400 hover:text-[#9CAF88] cursor-grab active:cursor-grabbing">
                            <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
                                <path
                                    d="M9 3h2v2H9V3zm0 4h2v2H9V7zm0 4h2v2H9v-2zm0 4h2v2H9v-2zm0 4h2v2H9v-2zm4-16h2v2h-2V3zm0 4h2v2h-2V7zm0 4h2v2h-2v-2zm0 4h2v2h-2v-2zm0 4h2v2h-2v-2z"/>
                            </svg>
                        </div>

                        {/* Book Icon */}
                        <div
                            class="w-8 h-10 bg-[#9CAF88]/10 rounded flex items-center justify-center text-xs flex-shrink-0">
                            📖
                        </div>

                        {/* Book Info */}
                        <div class="flex-1">
                            <div class="font-medium text-sm">{book.title}</div>
                            {/*<div class="text-xs text-gray-500">{book.subtitle}</div>*/}
                        </div>

                        {/* Actions */}
                        <div class="flex items-center gap-2">
                            {book.featured ? (
                                <button
                                    class="text-xs px-2 py-1 bg-green-100 text-green-700 rounded">Featured</button>
                            ) : (
                                <button
                                    class="text-xs px-2 py-1 bg-gray-100 text-gray-600 rounded hover:bg-green-100 hover:text-green-700"
                                    onClick$={() => {
                                        books.value = books.value.map(b =>
                                            b._id === book._id ? {
                                                ...b,
                                                featured: !b.featured
                                            } : {...b, featured: false}
                                        );
                                    }}
                                >
                                    Set Featured
                                </button>
                            )}
                            <button
                                class="text-xs px-2 py-1 bg-gray-100 text-gray-600 rounded hover:bg-gray-200">Edit
                            </button>
                        </div>
                    </div>
                ))}
            </div>

            <button
                class="w-full p-2 border-2 border-dashed border-gray-300 rounded text-sm text-gray-500 hover:border-[#9CAF88] hover:text-[#9CAF88] transition-colors"
                onClick$={() => {
                    // const newBook = {
                    //     id: Date.now().toString(),
                    //     title: 'New Book',
                    //     subtitle: 'Click edit to customize',
                    //     featured: false
                    // };
                    // books.value = [...books.value, newBook];
                }}
            >
                + Add New Book
            </button>
        </div>

    );
})