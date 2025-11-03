import {
  component$,
  useSignal,
  $,
  noSerialize,
  NoSerialize,
  QRL,
  useTask$,
  useOnDocument, isServer
} from "@builder.io/qwik";
import {Jodit} from 'jodit';

// Count syllables in a word (simple approximation)
const countSyllables = (word: string): number => {
  word = word.toLowerCase();
  if (word.length <= 3) return 1;

  const vowels = 'aeiouy';
  let syllables = 0;
  let previousWasVowel = false;

  for (let i = 0; i < word.length; i++) {
    const isVowel = vowels.includes(word[i]);
    if (isVowel && !previousWasVowel) {
      syllables++;
    }
    previousWasVowel = isVowel;
  }

  // Adjust for silent 'e'
  if (word.endsWith('e')) syllables--;

  return Math.max(1, syllables);
};

// Convert Wix Rich Text to HTML for Jodit
const wixToHtml =  (content: WixRichTextContent | string): string => {
  if (typeof content === 'string') {
    try {
      const parsed = JSON.parse(content);
      return wixToHtml(parsed);
    } catch {
      return content;
    }
  }

  if (!content?.nodes) return '';

  return content.nodes
      .map(node => {
        if (node.type === 'PARAGRAPH' && node.nodes) {
          const text = node.nodes
              .map(textNode => textNode.textData?.text || '')
              .join('');
          return `<p>${text}</p>`;
        }
        return '';
      })
      .join('');
};
// Convert HTML to Wix Rich Text format
const htmlToWix = (html: string): WixRichTextContent => {
  // Simple HTML to Wix conversion
  const tempDiv = document.createElement('div');
  tempDiv.innerHTML = html;

  const paragraphs = Array.from(tempDiv.querySelectorAll('p'));

  if (paragraphs.length === 0) {
    // No paragraphs, treat as single paragraph
    const text = tempDiv.textContent || '';
    return {
      nodes: [{
        type: "PARAGRAPH",
        id: "1",
        nodes: [{
          type: "TEXT",
          id: "2",
          textData: {
            text: text.trim(),
            decorations: []
          }
        }]
      }],
      documentStyle: {}
    };
  }

  const nodes = paragraphs.map((p, index) => ({
    type: "PARAGRAPH",
    id: (index * 2 + 1).toString(),
    nodes: [{
      type: "TEXT",
      id: (index * 2 + 2).toString(),
      textData: {
        text: p.textContent || '',
        decorations: []
      }
    }]
  }));

  return {
    nodes,
    documentStyle: {}
  };
};


export interface WixRichTextNode {
  type: string;
  id: string;
  nodes?: WixRichTextNode[];
  textData?: {
    text: string;
    decorations: any[];
  };
}

export interface WixRichTextContent {
  nodes: WixRichTextNode[];
  documentStyle: Record<string, any>;
}

export interface WritingStats {
  readabilityScore: number;
  sentences: number;
  words: number;
  characters: number;
  paragraphs: number;
  readingTime: number; // in minutes
  suggestions: WritingSuggestion[];
}

export interface WritingSuggestion {
  type: 'grammar' | 'style' | 'clarity' | 'tone' | 'readability';
  message: string;
  start: number;
  end: number;
  severity: 'error' | 'warning' | 'suggestion';
  replacement?: string;
}

export interface AIRichTextEditorProps {
  value?: WixRichTextContent | string;
  onInput$?: QRL<(content: WixRichTextContent) => void>;
  placeholder?: string;
  disabled?: boolean;
  class?: string;
  enableAI?: boolean;
  saplingApiKey?: string;
}

export default component$<AIRichTextEditorProps>((props) => {
  const editorRef = useSignal<HTMLDivElement>();
  const joditInstance = useSignal<NoSerialize<any>>();
  const currentContent = useSignal('');
  const writingStats = useSignal<WritingStats>({
    readabilityScore: 0,
    sentences: 0,
    words: 0,
    characters: 0,
    paragraphs: 0,
    readingTime: 0,
    suggestions: []
  });
  const isAnalyzing = useSignal(false);
  const showStats = useSignal(false);
  const isLoaded = useSignal(false);

  useOnDocument('load',$(()=>{
    isLoaded.value = true;
  }))

  // Initialize Jodit Editor
  useTask$(async ({track}) => {
   track(()=> isLoaded.value);
   track(()=> editorRef.value);
   if(isServer || !editorRef.value || !isLoaded.value) return;
      const config = {
        readonly: props.disabled || false,
        placeholder: props.placeholder || 'Start writing...',
        height: 200,
        minHeight: 150,
        toolbar: true,
        toolbarSticky: false,
        showCharsCounter: true,
        showWordsCounter: true,
        showXPathInStatusbar: false,
        beautifyHTML: true,
        theme: 'default',
        colors: {
          greyscale: [
            '#000000', '#434343', '#666666', '#999999', '#B7B7B7', '#CCCCCC', '#D9D9D9', '#EFEFEF', '#F3F3F3', '#FFFFFF'
          ],
          palette: [
            '#9CAF88', '#8BA077', '#7A9166', '#698255', '#587344', // QuillSpace greens
            '#FF5722', '#FF9800', '#FFC107', '#FFEB3B', '#CDDC39',
            '#8BC34A', '#4CAF50', '#009688', '#00BCD4', '#03A9F4',
            '#2196F3', '#3F51B5', '#673AB7', '#9C27B0', '#E91E63'
          ]
        },
        buttons: [
          'bold', 'italic', 'underline', '|',
          'ul', 'ol', '|',
          'link', 'unlink', '|',
          'align', '|',
          'undo', 'redo', '|',
          'hr', '|',
          'fullsize',
          {
            name: 'ai-assist',
            iconURL: 'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjQiIGhlaWdodD0iMjQiIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPHBhdGggZD0iTTEyIDJMMTMuMDkgOC4yNkwyMSA5TDEzLjA5IDE1Ljc0TDEyIDIyTDEwLjkxIDE1Ljc0TDMgOUwxMC45MSA4LjI2TDEyIDJaIiBzdHJva2U9IiM5Q0FGODA4IiBzdHJva2Utd2lkdGg9IjIiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCIvPgo8L3N2Zz4K',
            tooltip: 'AI Writing Assistant',
            exec: () => {
              analyzeWriting();
            }
          }
        ],
        events: {
          afterInit: () => {
            console.log('Jodit editor initialized');
          },
          change: (value: string) => {
            currentContent.value = value;
            handleContentChange(value);
          }
        }
      };

      const editor = Jodit.make(editorRef.value, config);
      joditInstance.value = noSerialize(editor);

      // Set initial content
      if (props.value) {
        const initialContent = wixToHtml(props.value);
        editor.value = initialContent;
        currentContent.value = initialContent;
      }

  });


  // Handle content changes
  const handleContentChange = $(async (html: string) => {
    const wixContent = htmlToWix(html);
    props.onInput$?.(wixContent);
    
    // Update writing stats
    await updateWritingStats(html);
  });

  // Update writing statistics (Hemingway-style analysis)
  const updateWritingStats = $((html: string) => {
    const text = html.replace(/<[^>]*>/g, '').trim();
    
    if (!text) {
      writingStats.value = {
        readabilityScore: 0,
        sentences: 0,
        words: 0,
        characters: 0,
        paragraphs: 0,
        readingTime: 0,
        suggestions: []
      };
      return;
    }

    const sentences = text.split(/[.!?]+/).filter(s => s.trim().length > 0);
    const words = text.split(/\s+/).filter(w => w.length > 0);
    const paragraphs = html.split('</p>').filter(p => p.trim().length > 0);
    
    // Simple readability score (Flesch-like)
    const avgWordsPerSentence = words.length / Math.max(sentences.length, 1);
    const avgSyllablesPerWord = words.reduce((acc, word) => {
      return acc + countSyllables(word);
    }, 0) / Math.max(words.length, 1);
    
    const readabilityScore = Math.max(0, Math.min(100, 
      206.835 - (1.015 * avgWordsPerSentence) - (84.6 * avgSyllablesPerWord)
    ));

    // Reading time (average 200 words per minute)
    const readingTime = Math.ceil(words.length / 200);

    // Generate Hemingway-style suggestions
    const suggestions: WritingSuggestion[] = [];
    
    // Check for long sentences (Hemingway principle)
    sentences.forEach((sentence, ) => {
      const sentenceWords = sentence.trim().split(/\s+/).length;
      if (sentenceWords > 20) {
        suggestions.push({
          type: 'readability',
          message: `This sentence is hard to read. Consider breaking it into shorter sentences.`,
          start: 0,
          end: sentence.length,
          severity: 'warning'
        });
      }
    });

    // Check for passive voice indicators
    const passiveIndicators = ['was', 'were', 'been', 'being', 'is', 'are', 'am'];
    passiveIndicators.forEach(indicator => {
      if (text.toLowerCase().includes(indicator)) {
        suggestions.push({
          type: 'style',
          message: `Consider using active voice instead of passive voice.`,
          start: 0,
          end: 0,
          severity: 'suggestion'
        });
      }
    });

    // Check for adverbs (Hemingway dislikes them)
    const adverbs = text.match(/\w+ly\b/gi) || [];
    if (adverbs.length > 0) {
      suggestions.push({
        type: 'style',
        message: `Consider removing adverbs (${adverbs.slice(0, 3).join(', ')}) for stronger writing.`,
        start: 0,
        end: 0,
        severity: 'suggestion'
      });
    }

    writingStats.value = {
      readabilityScore: Math.round(readabilityScore),
      sentences: sentences.length,
      words: words.length,
      characters: text.length,
      paragraphs: paragraphs.length,
      readingTime,
      suggestions
    };
  });

  // Analyze writing with Sapling AI
  const analyzeWriting = $(async () => {
    if (!props.enableAI || !props.saplingApiKey || !currentContent.value) return;
    
    isAnalyzing.value = true;
    
    try {
      const text = currentContent.value.replace(/<[^>]*>/g, '').trim();
      
      // Call Sapling API for grammar and style checking
      const response = await fetch('https://api.sapling.ai/api/v1/edits', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${props.saplingApiKey}`
        },
        body: JSON.stringify({
          text: text,
          session_id: 'quillspace-editor'
        })
      });
      
      if (response.ok) {
        const data = await response.json();
        
        // Convert Sapling suggestions to our format
        const aiSuggestions: WritingSuggestion[] = data.edits?.map((edit: any) => ({
          type: edit.error_type === 'Grammar' ? 'grammar' : 'style',
          message: edit.general_error_type || edit.error_type,
          start: edit.sentence_start + edit.start,
          end: edit.sentence_start + edit.end,
          severity: edit.error_type === 'Grammar' ? 'error' : 'suggestion',
          replacement: edit.replacement
        })) || [];
        
        // Merge with existing suggestions
        writingStats.value = {
          ...writingStats.value,
          suggestions: [...writingStats.value.suggestions, ...aiSuggestions]
        };
      }
    } catch (error) {
      console.error('Sapling AI analysis failed:', error);
    } finally {
      isAnalyzing.value = false;
    }
  });

  const getReadabilityGrade = (score: number): string => {
    if (score >= 90) return 'Very Easy';
    if (score >= 80) return 'Easy';
    if (score >= 70) return 'Fairly Easy';
    if (score >= 60) return 'Standard';
    if (score >= 50) return 'Fairly Difficult';
    if (score >= 30) return 'Difficult';
    return 'Very Difficult';
  };

  const getScoreColor = (score: number): string => {
    if (score >= 70) return 'text-green-600';
    if (score >= 50) return 'text-yellow-600';
    return 'text-red-600';
  };

  return (
    <div class={`relative ${props.class || ''}`}>
      {/* Jodit Editor Container */}
      <div ref={editorRef} />
      
      {/* Writing Statistics Panel */}
      <div class="mt-2 flex items-center justify-between">
        <button
          type="button"
          onClick$={() => showStats.value = !showStats.value}
          class="text-xs text-gray-500 hover:text-gray-700 flex items-center space-x-1"
        >
          <span>📊</span>
          <span>Writing Stats</span>
          <span class={`transform transition-transform ${showStats.value ? 'rotate-180' : ''}`}>▼</span>
        </button>
        
        {props.enableAI && (
          <button
            type="button"
            onClick$={analyzeWriting}
            disabled={isAnalyzing.value || !currentContent.value.trim()}
            class="text-xs px-2 py-1 bg-[#9CAF88] text-white rounded hover:bg-[#8BA077] disabled:opacity-50 disabled:cursor-not-allowed flex items-center space-x-1"
          >
            <span>✨</span>
            <span>{isAnalyzing.value ? 'Analyzing...' : 'AI Check'}</span>
          </button>
        )}
      </div>

      {/* Expanded Stats Panel */}
      {showStats.value && (
        <div class="mt-2 p-3 bg-gray-50 rounded-lg border text-xs">
          <div class="grid grid-cols-2 md:grid-cols-4 gap-3 mb-3">
            <div>
              <div class="font-medium text-gray-600">Words</div>
              <div class="text-lg font-semibold">{writingStats.value.words}</div>
            </div>
            <div>
              <div class="font-medium text-gray-600">Sentences</div>
              <div class="text-lg font-semibold">{writingStats.value.sentences}</div>
            </div>
            <div>
              <div class="font-medium text-gray-600">Reading Time</div>
              <div class="text-lg font-semibold">{writingStats.value.readingTime}m</div>
            </div>
            <div>
              <div class="font-medium text-gray-600">Readability</div>
              <div class={`text-lg font-semibold ${getScoreColor(writingStats.value.readabilityScore)}`}>
                {getReadabilityGrade(writingStats.value.readabilityScore)}
              </div>
            </div>
          </div>
          
          {/* Writing Suggestions */}
          {writingStats.value.suggestions.length > 0 && (
            <div>
              <div class="font-medium text-gray-600 mb-2">Writing Suggestions</div>
              <div class="space-y-1 max-h-32 overflow-y-auto">
                {writingStats.value.suggestions.slice(0, 5).map((suggestion, index) => (
                  <div key={index} class={`p-2 rounded text-xs ${
                    suggestion.severity === 'error' ? 'bg-red-50 text-red-700 border border-red-200' :
                    suggestion.severity === 'warning' ? 'bg-yellow-50 text-yellow-700 border border-yellow-200' :
                    'bg-blue-50 text-blue-700 border border-blue-200'
                  }`}>
                    <div class="flex items-start space-x-1">
                      <span class="font-medium capitalize">{suggestion.type}:</span>
                      <span>{suggestion.message}</span>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
      
      <div class="text-xs text-gray-500 mt-1">
        Professional rich text editor with AI-powered writing assistance
      </div>
    </div>
  );
});
