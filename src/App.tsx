import { useRef } from 'react';
import { useClipboard } from './hooks/useClipboard';
import { Search, Trash2, Copy, ExternalLink, Star } from 'lucide-react';
import { Toaster } from 'react-hot-toast';
import { useVirtualizer } from '@tanstack/react-virtual';

function App() {
  const { 
    items, 
    searchQuery, 
    setSearchQuery, 
    togglePin, 
    deleteItem, 
    copyToClipboard, 
    openUrl,
    loadMore,
    hasMore,
    isLoading
  } = useClipboard();

  const parentRef = useRef<HTMLDivElement>(null);

  const rowVirtualizer = useVirtualizer({
    count: items.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 100, // Estimated height of a clipboard item
    overscan: 5,
  });

  const isUrl = (str: string) => /^https?:\/\//i.test(str);

  return (
    <div className="flex flex-col h-screen bg-gray-950 text-gray-100 font-sans selection:bg-blue-500/30 overflow-hidden">
      <Toaster position="bottom-right" toastOptions={{
        style: {
          background: '#1f2937',
          color: '#f3f4f6',
          border: '1px solid #374151'
        }
      }} />
      
      <header className="p-4 bg-gray-900/50 backdrop-blur-md border-b border-gray-800 shrink-0 z-20">
        <div className="relative group">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500 group-focus-within:text-blue-400 transition-colors" />
          <input 
            type="text" 
            placeholder="Search clipboard history..." 
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-10 pr-4 py-2 bg-gray-800 border border-gray-700 rounded-lg shadow-inner focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus:border-blue-500 transition-all placeholder:text-gray-600"
          />
        </div>
      </header>
      
      <main 
        ref={parentRef}
        className="flex-1 overflow-y-auto p-4 scrollbar-thin scrollbar-thumb-gray-800 scrollbar-track-transparent"
        onScroll={(e) => {
          const target = e.currentTarget;
          if (target.scrollHeight - target.scrollTop <= target.clientHeight + 100) {
            loadMore();
          }
        }}
      >
        {items.length === 0 && !isLoading ? (
          <div className="flex flex-col items-center justify-center h-full text-gray-500 space-y-2 opacity-50">
            <Copy className="w-12 h-12" />
            <p className="text-lg font-medium">No clips found</p>
            <p className="text-sm">Copy something to see it here!</p>
          </div>
        ) : (
          <div
            style={{
              height: `${rowVirtualizer.getTotalSize()}px`,
              width: '100%',
              position: 'relative',
            }}
          >
            {rowVirtualizer.getVirtualItems().map((virtualItem) => {
              const item = items[virtualItem.index];
              return (
                <div
                  key={virtualItem.key}
                  data-index={virtualItem.index}
                  ref={rowVirtualizer.measureElement}
                  className="absolute top-0 left-0 w-full pb-3"
                  style={{
                    transform: `translateY(${virtualItem.start}px)`,
                  }}
                >
                  <div className="group flex flex-col p-4 bg-gray-900 border border-gray-800 rounded-xl shadow-sm hover:border-gray-600 hover:bg-gray-800/80 transition-all duration-200 relative overflow-hidden">
                    <div className="flex justify-between items-start mb-2">
                      <div className="flex items-center space-x-2">
                        <span className="text-[10px] uppercase tracking-wider text-gray-500 font-bold">
                          {new Date(item.created_at * 1000).toLocaleString()}
                        </span>
                        <span className={`text-[9px] px-1.5 py-0.5 rounded-full font-bold uppercase tracking-tighter ${
                          item.category === 'URL' ? 'bg-blue-500/20 text-blue-400' :
                          item.category === 'Code' ? 'bg-purple-500/20 text-purple-400' :
                          item.category === 'Color' ? 'bg-green-500/20 text-green-400' :
                          'bg-gray-500/20 text-gray-400'
                        }`}>
                          {item.category}
                        </span>
                      </div>
                      <div className="flex space-x-1 opacity-0 group-hover:opacity-100 focus-within:opacity-100 transition-opacity">
                        {isUrl(item.content) && (
                          <button 
                            onClick={() => openUrl(item.content)}
                            className="p-1.5 rounded-md hover:bg-blue-500/20 text-blue-400 transition-colors"
                            title="Open URL"
                          >
                            <ExternalLink className="w-4 h-4" />
                          </button>
                        )}
                        <button 
                          onClick={() => togglePin(item.id)}
                          className={`p-1.5 rounded-md transition-colors ${item.pinned ? 'bg-amber-500/20 text-amber-400' : 'hover:bg-amber-500/10 text-gray-500 hover:text-amber-400'}`}
                          title={item.pinned ? 'Unpin' : 'Pin'}
                        >
                          <Star className={`w-4 h-4 ${item.pinned ? 'fill-current' : ''}`} />
                        </button>
                        <button 
                          onClick={() => deleteItem(item.id)}
                          className="p-1.5 rounded-md hover:bg-red-500/20 text-gray-500 hover:text-red-400 transition-colors"
                          title="Delete"
                        >
                          <Trash2 className="w-4 h-4" />
                        </button>
                      </div>
                    </div>

                    <div 
                      className="cursor-pointer group/content"
                      onClick={() => copyToClipboard(item.content)}
                    >
                      <p className="text-sm text-gray-300 whitespace-pre-wrap break-words line-clamp-6 group-hover/content:text-white transition-colors">
                        {item.content}
                      </p>
                      <div className="absolute inset-x-0 bottom-0 h-1 bg-blue-500 transform scale-x-0 group-hover/content:scale-x-100 transition-transform origin-left" />
                    </div>
                    
                    {item.pinned && (
                      <div className="absolute top-0 right-0 w-8 h-8 flex items-center justify-center pointer-events-none">
                        <div className="absolute top-0 right-0 border-[16px] border-t-amber-500/20 border-r-amber-500/20 border-l-transparent border-b-transparent" />
                      </div>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        )}
        
        {hasMore && !isLoading && items.length > 0 && (
           <div className="py-4 text-center">
              <button 
                onClick={loadMore}
                className="text-sm text-gray-500 hover:text-blue-400 transition-colors"
              >
                Load more clips
              </button>
           </div>
        )}

        {isLoading && (
          <div className="py-4 text-center text-sm text-blue-500 animate-pulse">
            Loading...
          </div>
        )}
      </main>
      
      <footer className="px-4 py-2 bg-gray-900 border-t border-gray-800 text-[10px] text-gray-600 flex justify-between items-center shrink-0">
        <span>Press <kbd className="px-1.5 py-0.5 bg-gray-800 rounded border border-gray-700 text-gray-400">Alt + C</kbd> to toggle window</span>
        <div className="flex items-center space-x-3">
          <span>{items.length} items</span>
          <div className={`w-2 h-2 rounded-full ${isLoading ? 'bg-blue-500 animate-pulse' : 'bg-green-500'}`} />
        </div>
      </footer>
    </div>
  );
}

export default App;
