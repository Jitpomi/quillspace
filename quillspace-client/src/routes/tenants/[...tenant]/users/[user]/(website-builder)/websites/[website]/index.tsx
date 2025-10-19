import {
  component$,
  useSignal,
  $,
  useVisibleTask$,
  useComputed$,
  useOnDocument,
  useTask$,
  isServer, useOnWindow
} from '@builder.io/qwik';
import { routeLoader$ } from "@builder.io/qwik-city";
import { getAuthToken, getTenantInfo, getUserInfo } from "~/utils/auth";
import { 
  LuEye,
  LuExternalLink,
  LuRefreshCw,
  LuX,
  LuMaximize2,
  LuMinimize2,
  LuArrowLeft,
  LuPencil,
  LuSave,
  LuType,
  LuImage,
  LuDollarSign
} from '@qwikest/icons/lucide';

export const useConnectedWebsite = routeLoader$(async (requestEventAction) => {
  const { cookie, params } = requestEventAction;
  const user = await getUserInfo(cookie);
  const tenant = await getTenantInfo(cookie);
  const token = getAuthToken(cookie);
  if (!user || !tenant) {
    throw new Error('User not authenticated');
  }

  try {
    // Call the backend API to get connected websites
    const response = await fetch(`${process.env.BACKEND_URL || 'http://localhost:3001'}/api/connected-websites/websites/${params.website}`, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      throw new Error(`Failed to fetch websites: ${response.statusText}`);
    }

    const website = await response.json();
    return { success: true, website };
  } catch (error) {
    console.error('Error fetching connected websites:', error);
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Unknown error',
      website: null
    };
  }
});

export default component$(() => {
  const websiteData = useConnectedWebsite();
  
  const isFullscreen = useSignal(false);
  type EditorMode = 'preview' | 'edit';
  const editorMode = useSignal<EditorMode>('edit');
  const isLoading = useSignal(true);
  const showEditSidebar = useSignal(false);
  const expandedAccordion = useSignal<string | null>(null);
  
  // Interactive editing state
  const selectedElement = useSignal<string | null>(null);
  const editPopup = useSignal<{
    show: boolean;
    x: number;
    y: number;
    type: 'text' | 'image' | 'book' | 'price';
    content: string;
    elementId: string;
  }>({
    show: false,
    x: 0,
    y: 0,
    type: 'text',
    content: '',
    elementId: ''
  });

  const handleIframeLoad = $(() => {
    console.log('Iframe loaded successfully');
    isLoading.value = false;
  });

  const handleIframeError = $(() => {
    isLoading.value = false;
    console.log('Iframe failed to load');
  });

  // Initial loading timeout
  useOnDocument('load', $(() => {
    isLoading.value = false;
  }))

  const toggleFullscreen = $(() => {
    isFullscreen.value = !isFullscreen.value;
  });

  const switchMode = $((mode: EditorMode) => {
    editorMode.value = mode;
    isLoading.value = true;
    
    // Reset editing state when switching modes
    editPopup.value = { ...editPopup.value, show: false };
    selectedElement.value = null;
    
    // Fallback timeout to stop loading after 10 seconds
    setTimeout(() => {
      if (isLoading.value) {
        console.log('Iframe load timeout, stopping loading state');
        isLoading.value = false;
      }
    }, 10000);
  });

  // Interactive editing functions
  // const handleElementClick = $((event: MouseEvent, elementType: string, content: string, elementId: string) => {
  //   if ((editorMode.value as EditorMode) !== 'edit') return;
  //
  //   event.preventDefault();
  //   event.stopPropagation();
  //
  //   const rect = (event.target as HTMLElement).getBoundingClientRect();
  //   const iframeRect = document.getElementById('website-iframe')?.getBoundingClientRect();
  //
  //   if (iframeRect) {
  //     editPopup.value = {
  //       show: true,
  //       x: rect.left - iframeRect.left + rect.width / 2,
  //       y: rect.top - iframeRect.top - 10,
  //       type: elementType as 'text' | 'image' | 'book' | 'price',
  //       content: content,
  //       elementId: elementId
  //     };
  //     selectedElement.value = elementId;
  //   }
  // });

  const saveEdit = $((newContent: string) => {
    // Here you would save the changes to your backend/Wix
    console.log('Saving:', editPopup.value.elementId, newContent);
    
    // Update the iframe content (this would be done via API in real implementation)
    const iframe = document.getElementById('website-iframe') as HTMLIFrameElement;
    if (iframe && iframe.contentDocument) {
      const element = iframe.contentDocument.getElementById(editPopup.value.elementId);
      if (element) {
        element.textContent = newContent;
      }
    }
    
    // Close popup
    editPopup.value = { ...editPopup.value, show: false };
    selectedElement.value = null;
  });

  const closeEditPopup = $(() => {
    editPopup.value = { ...editPopup.value, show: false };
    selectedElement.value = null;
  });

  // Analyze what element was clicked to determine edit type and content
  const analyzeClickPosition = $((x: number, y: number, iframe: HTMLIFrameElement) => {
    try {
      // Try to access iframe content (may fail due to cross-origin)
      const iframeDoc = iframe.contentDocument;
      if (iframeDoc) {
        const element = iframeDoc.elementFromPoint(x, y) as HTMLElement;
        if (element) {
          return analyzeElement(element);
        }
      }
    } catch (error) {
      console.log('Cross-origin restriction, using position-based detection: ',error);
    }
    
    // Fallback: Use position-based heuristics
    return analyzeByPosition(x, y, iframe);
  });

  // Analyze HTML element to determine type and content
  const analyzeElement = $((element: HTMLElement) => {
    const tagName = element.tagName.toLowerCase();
    const className = element.className.toLowerCase();
    const textContent = element.textContent?.trim() || '';
    
    // Image detection
    if (tagName === 'img') {
      return {
        type: 'image' as const,
        content: (element as HTMLImageElement).src,
        elementId: `img-${Date.now()}`,
        title: 'Edit Image'
      };
    }
    
    // Price detection
    if (className.includes('price') || className.includes('cost') || 
        textContent.includes('$') || textContent.includes('€') || textContent.includes('£')) {
      return {
        type: 'price' as const,
        content: textContent,
        elementId: `price-${Date.now()}`,
        title: 'Edit Price'
      };
    }
    
    // Button/Link detection
    if (tagName === 'button' || tagName === 'a' || className.includes('button') || className.includes('btn')) {
      return {
        type: 'text' as const,
        content: textContent,
        elementId: `button-${Date.now()}`,
        title: 'Edit Button Text'
      };
    }
    
    // Heading detection
    if (['h1', 'h2', 'h3', 'h4', 'h5', 'h6'].includes(tagName)) {
      return {
        type: 'text' as const,
        content: textContent,
        elementId: `heading-${Date.now()}`,
        title: 'Edit Heading'
      };
    }
    
    // Default text content
    return {
      type: 'text' as const,
      content: textContent || 'Click to edit this content',
      elementId: `text-${Date.now()}`,
      title: 'Edit Text'
    };
  });

  // Fallback analysis based on click position when cross-origin blocks access
  const analyzeByPosition = $((x: number, y: number, iframe: HTMLIFrameElement) => {
    const rect = iframe.getBoundingClientRect();
    const relativeX = x / rect.width;
    const relativeY = y / rect.height;
    
    // Top area likely to be navigation/headers
    if (relativeY < 0.2) {
      return {
        type: 'text' as const,
        content: 'Navigation or Header Text',
        elementId: `nav-${Date.now()}`,
        title: 'Edit Navigation'
      };
    }
    
    // Center area likely to be main content
    if (relativeY > 0.2 && relativeY < 0.8) {
      // Left side might be images, right side text
      if (relativeX < 0.4) {
        return {
          type: 'image' as const,
          content: 'https://via.placeholder.com/300x400',
          elementId: `img-${Date.now()}`,
          title: 'Edit Image'
        };
      } else {
        return {
          type: 'text' as const,
          content: 'Main content text',
          elementId: `content-${Date.now()}`,
          title: 'Edit Content'
        };
      }
    }
    
    // Bottom area likely to be footer or buttons
    return {
      type: 'text' as const,
      content: 'Footer or Button Text',
      elementId: `footer-${Date.now()}`,
      title: 'Edit Footer'
    };
  });

  // Double-click to edit with better event handling
  useTask$(({ track }) => {
    track(() => editorMode.value);
    track(() => isLoading.value);
    if(isServer) return;
    const iframe = document.getElementById('website-iframe') as HTMLIFrameElement;
    if (!iframe) return;
    
    // Add double-click handler with debugging
    const handleDoubleClick = async (e: MouseEvent) => {
      console.log('Double-click detected!', editorMode.value);
      
      if ((editorMode.value as EditorMode) !== 'edit') return;
      
      e.preventDefault();
      e.stopPropagation();
      
      const rect = iframe.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;
      
      console.log('Click position:', x, y);
      
      // Try to get the element at the click position from iframe
      const elementInfo = await analyzeClickPosition(x, y, iframe);
      
      console.log('Element info:', elementInfo);
      
      // Show edit popup at click position
      editPopup.value = {
        show: true,
        x: x,
        y: y,
        type: elementInfo.type,
        content: elementInfo.content,
        elementId: elementInfo.elementId
      };
    };
    
    // Add event listeners to multiple elements for better coverage
    const container = iframe.parentElement;
    if (container) {
      container.addEventListener('dblclick', handleDoubleClick);
      
      return () => {
        container.removeEventListener('dblclick', handleDoubleClick);
      };
    }
  });

  const handleMessage = $((event: MessageEvent) => {
    // Only process QuillSpace messages
    if (event.data.type !== 'QUILLSPACE_ELEMENT_CLICK') return;

    // Only process in edit mode
    if ((editorMode.value as EditorMode) !== 'edit') return;

    const elementData = event.data.data;
    console.log('Received element data from iframe:', elementData);

    // Show edit popup with the exact element data
    editPopup.value = {
      show: true,
      x: elementData.position.x,
      y: elementData.position.y,
      type: elementData.type,
      content: elementData.content,
      elementId: elementData.elementId
    };
  });

  useOnWindow('message',handleMessage)


  if (!websiteData.value.success) {
    return (
      <div class="min-h-screen bg-gray-50 flex items-center justify-center">
        <div class="text-center">
          <div class="w-16 h-16 bg-red-100 rounded-full flex items-center justify-center mx-auto mb-4">
            <LuX class="w-8 h-8 text-red-600" />
          </div>
          <h2 class="text-xl font-semibold text-gray-900 mb-2">Unable to load website</h2>
          <p class="text-gray-600">{websiteData.value.error}</p>
        </div>
      </div>
    );
  }


  // // Get URLs directly without $ wrapper since they're synchronous
  // const editorUrl = useComputed$(() => {
  //   if (!websiteData.value.success || !websiteData.value.website) return '';
  //
  //   const website = websiteData.value.website;
  //
  //   // First try to use the edit_url from metadata if available
  //   if (website.metadata?.edit_url || website.metadata?.editUrl) {
  //     return website.metadata.edit_url || website.metadata.editUrl;
  //   }
  //
  //   // Construct the proper Wix editor URL
  //   // The correct format is: https://manage.wix.com/dashboard/{site-id}/home
  //   // Or for direct editing: https://editor.wix.com/html/editor/web/renderer/edit/{site-id}?metaSiteId={site-id}
  //   const siteId = website.external_site_id;
  //   if (siteId) {
  //     // Try the manage dashboard first (more reliable)
  //     return `https://manage.wix.com/dashboard/${siteId}/home`;
  //   }
  //
  //   return '';
  // });

  const previewUrl = useComputed$(() => {
    if (!websiteData.value.success || !websiteData.value.website) return '';

    const website = websiteData.value.website;
    return website.url || website.metadata?.view_url || website.metadata?.viewUrl || '';
  });


  const website = useComputed$(() => websiteData.value.website);
  // const currentUrl = useComputed$(() => {
  //   return ((editorMode.value as EditorMode) === 'edit') ? editorUrl.value : previewUrl.value;
  // });

  return (
    <div class={`${isFullscreen.value || (editorMode.value as EditorMode) === 'preview' ? 'fixed inset-0 z-50' : 'min-h-screen'} bg-gray-50`}>
      {/* Header - Hidden in preview mode */}
      {(editorMode.value as EditorMode) === 'edit' && (
        <div class="bg-white border-b border-gray-200 sticky top-0 z-10">
          <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <div class="flex items-center justify-between h-16">
              <div class="flex items-center gap-4">
                {/* Back Button - Always visible */}
                <a
                  href="../"
                  class="flex items-center gap-2 p-2 text-gray-400 hover:text-gray-600 rounded-lg transition-colors"
                  title="Back to Websites"
                >
                  <LuArrowLeft class="w-5 h-5" />
                  {!isFullscreen.value && <span class="text-sm">Back</span>}
                </a>
                
                
                <div>
                  <h1 class="text-xl font-semibold text-gray-900">{website.value?.name || 'Website Editor'}</h1>
                </div>
              </div>
              
              {/* Toolbar Buttons */}
              <div class="flex items-center gap-1">
                <button
                  onClick$={() => switchMode('edit')}
                  class={`p-2 rounded-lg transition-colors ${
                    (editorMode.value as EditorMode) === 'edit'
                      ? 'bg-[#9CAF88] text-white'
                      : 'text-gray-500 hover:text-gray-700 hover:bg-gray-100'
                  }`}
                  title="Edit Mode"
                >
                  <LuPencil class="w-4 h-4" />
                </button>
                
                <button
                  onClick$={() => switchMode('preview')}
                  class={`p-2 rounded-lg transition-colors ${
                    (editorMode.value as EditorMode) === 'preview'
                      ? 'bg-[#9CAF88] text-white'
                      : 'text-gray-500 hover:text-gray-700 hover:bg-gray-100'
                  }`}
                  title="Preview"
                >
                  <LuEye class="w-4 h-4" />
                </button>


                <button
                  onClick$={() => {
                    isLoading.value = true;
                    const iframe = document.querySelector('#website-iframe') as HTMLIFrameElement;
                    if (iframe) {
                      iframe.src = website.value.url;
                    }
                  }}
                  class="p-2 rounded-lg text-gray-500 hover:text-gray-700 hover:bg-gray-100 transition-colors"
                  title="Refresh"
                >
                  <LuRefreshCw class="w-4 h-4" />
                </button>

                <button
                  onClick$={toggleFullscreen}
                  class="p-2 rounded-lg text-gray-500 hover:text-gray-700 hover:bg-gray-100 transition-colors"
                  title={isFullscreen.value ? 'Exit Fullscreen' : 'Enter Fullscreen'}
                >
                  {isFullscreen.value ? (
                    <LuMinimize2 class="w-4 h-4" />
                  ) : (
                    <LuMaximize2 class="w-4 h-4" />
                  )}
                </button>

                {website.value?.url && (
                  <a
                    href={website.value.url}
                    target="_blank"
                    rel="noopener noreferrer"
                    class="p-2 rounded-lg text-gray-500 hover:text-gray-700 hover:bg-gray-100 transition-colors"
                    title="Open in New Tab"
                  >
                    <LuExternalLink class="w-4 h-4" />
                  </a>
                )}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Floating Controls for Preview Mode */}
      {(editorMode.value as EditorMode) === 'preview' && (
        <div class="fixed top-4 right-4 z-50 flex items-center gap-1 bg-white bg-opacity-90 backdrop-blur-sm rounded-lg p-1 shadow-lg border border-gray-200">
          <button
            onClick$={() => switchMode('edit')}
            class="p-2 rounded-lg text-gray-600 hover:text-gray-900 hover:bg-gray-100 transition-colors"
            title="Edit Website"
          >
            <LuPencil class="w-4 h-4" />
          </button>

          <button
            onClick$={() => {
              isLoading.value = true;
              const iframe = document.querySelector('#website-iframe') as HTMLIFrameElement;
              if (iframe) {
                iframe.src = website.value.url;
              }
            }}
            class="p-2 rounded-lg text-gray-600 hover:text-gray-900 hover:bg-gray-100 transition-colors"
            title="Refresh"
          >
            <LuRefreshCw class="w-4 h-4" />
          </button>

          {website.value?.url && (
            <a
              href={website.value.url}
              target="_blank"
              rel="noopener noreferrer"
              class="p-2 rounded-lg text-gray-600 hover:text-gray-900 hover:bg-gray-100 transition-colors"
              title="Open in New Tab"
            >
              <LuExternalLink class="w-4 h-4" />
            </a>
          )}
          
          <a
            href="../"
            class="p-2 rounded-lg text-gray-600 hover:text-gray-900 hover:bg-gray-100 transition-colors"
            title="Back to Websites"
          >
            <LuArrowLeft class="w-4 h-4" />
          </a>
        </div>
      )}

      {/* Editor Content */}
      <div class={`relative ${
        (editorMode.value as EditorMode) === 'preview' ? 'h-screen' : 
        isFullscreen.value ? 'h-[calc(100vh-64px)]' : 'h-[calc(100vh-128px)]'
      }`}>
        {/* Loading Overlay */}
        {isLoading.value && (
          <div class="absolute inset-0 bg-white bg-opacity-75 flex items-center justify-center z-10">
            <div class="text-center">
              <LuRefreshCw class="w-8 h-8 text-[#9CAF88] animate-spin mx-auto mb-4" />
              <p class="text-gray-600">
                {(editorMode.value as EditorMode) === 'edit' ? 'Loading Interactive Editor...' : 'Loading Preview...'}
              </p>
            </div>
          </div>
        )}



        {/* Website Iframe */}
        {previewUrl.value ? (
          <iframe
            id="website-iframe"
            src={previewUrl.value}
            class="w-full h-full border-0"
            onLoad$={handleIframeLoad}
            onError$={handleIframeError}
            title={`${(editorMode.value as EditorMode) === 'edit' ? 'Interactive Editor' : 'Website Preview'} - ${website.value?.name || 'Website'}`}
            sandbox="allow-same-origin allow-scripts allow-forms"
          />
        ) : (
          <div class="w-full h-full flex items-center justify-center bg-gray-100">
            <div class="text-center">
              <div class="text-gray-500 mb-2">No website URL found</div>
              <div class="text-sm text-gray-400">
                Website data: {JSON.stringify(websiteData.value, null, 2)}
              </div>
            </div>
          </div>
        )}

        {/* Floating Action Button - Only visible in edit mode */}
        {(editorMode.value as EditorMode) === 'edit' && (
          <button
            onClick$={() => showEditSidebar.value = true}
            class="fixed bottom-6 right-6 z-40 bg-[#9CAF88] text-white p-4 rounded-full shadow-lg hover:bg-[#8BA079] transition-all duration-200 hover:scale-105"
            title="Edit Website Content"
          >
            <LuPencil class="w-6 h-6" />
          </button>
        )}

        {/* Debug Info */}
        {process.env.NODE_ENV === 'development' && (
          <div class="absolute bottom-4 left-4 bg-black bg-opacity-75 text-white text-xs p-2 rounded max-w-xs">
            <div>URL: {previewUrl.value || 'No URL'}</div>
            <div>Mode: {editorMode.value}</div>
            <div>Loading: {isLoading.value ? 'Yes' : 'No'}</div>
          </div>
        )}

        {/* Edit Modal */}
        {showEditSidebar.value && (
          <div class="fixed inset-0 z-50 flex items-center justify-center">
            {/* Backdrop */}
            <div 
              class="absolute inset-0"
              onClick$={() => showEditSidebar.value = false}
            ></div>
            
            {/* Modal */}
            <div class="relative bg-white rounded-2xl shadow-lg border border-gray-100 w-[800px] max-h-[80vh] overflow-hidden">
              {/* Header */}
              <div class="px-8 py-6 border-b border-gray-50 bg-gradient-to-r from-gray-50/50 to-[#9CAF88]/5">
                <div class="flex items-center gap-3">
                  <div class="w-10 h-10 rounded-full bg-[#9CAF88]/10 flex items-center justify-center">
                    <svg class="w-5 h-5 text-[#9CAF88]" fill="currentColor" viewBox="0 0 24 24">
                      <path d="M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zM20.71 7.04c.39-.39.39-1.02 0-1.41l-2.34-2.34c-.39-.39-1.02-.39-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z"/>
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
                      class="w-full flex items-center justify-between p-5 text-left hover:bg-[#9CAF88]/5 transition-colors rounded-xl"
                      onClick$={() => {
                        expandedAccordion.value = expandedAccordion.value === 'books' ? null : 'books';
                      }}
                    >
                      <div class="flex items-center gap-3">
                        <span class="text-xl">📚</span>
                        <div>
                          <div class="font-medium text-gray-900 text-sm">Books Management</div>
                          <div class="text-xs text-gray-500 mt-0.5">Show/hide books, edit details, set featured books</div>
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
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"></path>
                      </svg>
                    </button>
                    
                    {/* Books Content */}
                    {expandedAccordion.value === 'books' && (
                      <div class="border-t border-gray-200 p-4 bg-gray-50">
                        <div class="space-y-3">
                          <div class="text-sm font-medium text-gray-700 mb-3">Manage Your Books</div>
                          
                          {/* Book List */}
                          <div class="space-y-2">
                            <div class="flex items-center justify-between p-3 bg-white rounded border">
                              <div class="flex items-center gap-3">
                                <div class="w-8 h-10 bg-[#9CAF88]/10 rounded flex items-center justify-center text-xs">📖</div>
                                <div>
                                  <div class="font-medium text-sm">The Missing Corpse</div>
                                  <div class="text-xs text-gray-500">Book 2 of 2: The General's Project</div>
                                </div>
                              </div>
                              <div class="flex items-center gap-2">
                                <button class="text-xs px-2 py-1 bg-green-100 text-green-700 rounded">Featured</button>
                                <button class="text-xs px-2 py-1 bg-gray-100 text-gray-600 rounded hover:bg-gray-200">Edit</button>
                              </div>
                            </div>
                          </div>
                          
                          <button class="w-full p-2 border-2 border-dashed border-gray-300 rounded text-sm text-gray-500 hover:border-gray-400 hover:text-gray-600">
                            + Add New Book
                          </button>
                        </div>
                      </div>
                    )}
                  </div>

                  {/* Author Accordion */}
                  <div class="bg-white border border-gray-100 rounded-xl shadow-sm hover:shadow-md hover:border-[#9CAF88]/30 transition-all duration-200">
                    <button 
                      class="w-full flex items-center justify-between p-5 text-left hover:bg-[#9CAF88]/5 transition-colors rounded-xl"
                      onClick$={() => {
                        expandedAccordion.value = expandedAccordion.value === 'author' ? null : 'author';
                      }}
                    >
                      <div class="flex items-center gap-3">
                        <span class="text-xl">👤</span>
                        <div>
                          <div class="font-medium text-gray-900 text-sm">Author Information</div>
                          <div class="text-xs text-gray-500 mt-0.5">Update name, bio, photo, and contact information</div>
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
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"></path>
                      </svg>
                    </button>
                    
                    {/* Author Content */}
                    {expandedAccordion.value === 'author' && (
                      <div class="border-t border-gray-200 p-4 bg-gray-50">
                        <div class="space-y-4">
                          <div class="text-sm font-medium text-gray-700 mb-3">Author Profile</div>
                          
                          <div class="space-y-3">
                            <div>
                              <label class="block text-xs font-medium text-gray-600 mb-1">Author Name</label>
                              <input 
                                type="text" 
                                value="Yasin Kakande" 
                                class="w-full px-3 py-2 text-sm border border-gray-300 rounded focus:outline-none focus:ring-[0.5px] focus:ring-[#9CAF88]/30 focus:border-[#9CAF88]"
                              />
                            </div>
                            
                            <div>
                              <label class="block text-xs font-medium text-gray-600 mb-1">Bio</label>
                              <textarea 
                                rows={3}
                                class="w-full px-3 py-2 text-sm border border-gray-300 rounded focus:outline-none focus:ring-[0.5px] focus:ring-[#9CAF88]/30 focus:border-[#9CAF88]"
                                placeholder="Author biography..."
                              ></textarea>
                            </div>
                            
                            <div class="flex gap-2">
                              <button class="px-3 py-1 bg-[#9CAF88] text-white text-xs rounded hover:bg-[#8BA079]">
                                Save Changes
                              </button>
                              <button class="px-3 py-1 bg-gray-200 text-gray-700 text-xs rounded hover:bg-gray-300">
                                Cancel
                              </button>
                            </div>
                          </div>
                        </div>
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
                          <div class="text-xs text-gray-400 mt-0.5">Customize colors and fonts - Coming soon</div>
                        </div>
                      </div>
                      <svg class="w-4 h-4 text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"></path>
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
                          <div class="text-xs text-gray-400 mt-0.5">Visual page editing - Coming soon</div>
                        </div>
                      </div>
                      <svg class="w-4 h-4 text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"></path>
                      </svg>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Edit Popup */}
        {editPopup.value.show && (
          <div 
            class="absolute z-30 bg-white rounded-lg shadow-xl border border-gray-200 p-4 min-w-80"
            style={{
              left: `${editPopup.value.x - 160}px`,
              top: `${editPopup.value.y}px`,
              transform: 'translateY(-100%)'
            }}
          >
            <div class="flex items-center justify-between mb-3">
              <div class="flex items-center gap-2">
                {editPopup.value.type === 'text' && <LuType class="w-4 h-4 text-[#9CAF88]" />}
                {editPopup.value.type === 'image' && <LuImage class="w-4 h-4 text-[#9CAF88]" />}
                {editPopup.value.type === 'price' && <LuDollarSign class="w-4 h-4 text-[#9CAF88]" />}
                <span class="text-sm font-medium text-gray-700">
                  {(editPopup.value as any).title || `Edit ${editPopup.value.type}`}
                </span>
              </div>
              <button 
                onClick$={closeEditPopup}
                class="text-gray-400 hover:text-gray-600"
              >
                <LuX class="w-4 h-4" />
              </button>
            </div>

            {editPopup.value.type === 'text' || editPopup.value.type === 'price' ? (
              <div class="space-y-3">
                <textarea
                  value={editPopup.value.content}
                  rows={editPopup.value.type === 'price' ? 1 : 3}
                  class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-[#9CAF88] focus:border-transparent text-sm"
                  placeholder={editPopup.value.type === 'price' ? 'Enter price (e.g., $19.99)' : 'Enter text content'}
                  onInput$={(e) => {
                    editPopup.value = {
                      ...editPopup.value,
                      content: (e.target as HTMLTextAreaElement).value
                    };
                  }}
                />
                <div class="flex gap-2">
                  <button
                    onClick$={() => saveEdit(editPopup.value.content)}
                    class="flex-1 bg-[#9CAF88] text-white py-2 px-3 rounded-lg hover:bg-[#8BA079] transition-colors text-sm font-medium"
                  >
                    <LuSave class="w-4 h-4 inline mr-1" />
                    Save
                  </button>
                  <button
                    onClick$={closeEditPopup}
                    class="px-3 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors text-sm"
                  >
                    Cancel
                  </button>
                </div>
              </div>
            ) : editPopup.value.type === 'image' ? (
              <div class="space-y-3">
                <div class="text-sm text-gray-600 mb-2">Current image:</div>
                <img 
                  src={editPopup.value.content} 
                  alt="Current" 
                  class="w-full h-24 object-cover rounded border"
                />
                <input
                  type="url"
                  value={editPopup.value.content}
                  class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-[#9CAF88] focus:border-transparent text-sm"
                  placeholder="Enter image URL"
                  onInput$={(e) => {
                    editPopup.value = {
                      ...editPopup.value,
                      content: (e.target as HTMLInputElement).value
                    };
                  }}
                />
                <div class="flex gap-2">
                  <button
                    onClick$={() => saveEdit(editPopup.value.content)}
                    class="flex-1 bg-[#9CAF88] text-white py-2 px-3 rounded-lg hover:bg-[#8BA079] transition-colors text-sm font-medium"
                  >
                    <LuSave class="w-4 h-4 inline mr-1" />
                    Update Image
                  </button>
                  <button
                    onClick$={closeEditPopup}
                    class="px-3 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors text-sm"
                  >
                    Cancel
                  </button>
                </div>
              </div>
            ) : null}
          </div>
        )}
      </div>

      {/* Help Text */}
      {!isFullscreen.value && (
        <div class="bg-gray-50 border-t border-gray-200 px-4 py-3">
          <div class="max-w-7xl mx-auto">
            <p class="text-sm text-gray-600 text-center">
              {(editorMode.value as EditorMode) === 'edit' ? (
                <>
                  <span class="font-medium">Edit Mode:</span> Double-click any element on your website to edit its content directly. 
                  Changes sync automatically to your Wix site.
                </>
              ) : (
                <>
                  <span class="font-medium">Preview Mode:</span> See how your website looks to visitors. 
                  Switch to Edit mode to make changes.
                </>
              )}
            </p>
          </div>
        </div>
      )}
    </div>
  );
});

