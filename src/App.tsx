import { useClipboard } from './hooks/useClipboard';

function App() {
  const { items, searchQuery, setSearchQuery, togglePin, deleteItem, copyToClipboard, openUrl } = useClipboard();

  const isUrl = (str: string) => /^https?:\/\//i.test(str);

  return (
    <div className="flex flex-col h-screen bg-gray-50 text-gray-900">
      <header className="p-4 bg-white border-b sticky top-0 z-10">
        <input
          type="text"
          placeholder="Search clips..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="w-full p-2 border rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
      </header>

      <main className="flex-1 overflow-y-auto p-4 space-y-2">
        {items.length === 0 ? (
          <p className="text-center text-gray-500 mt-10">No clips found.</p>
        ) : (
          items.map(item => (
            <div key={item.id} className="group flex items-start p-3 bg-white border rounded-md shadow-sm hover:border-blue-300 relative pr-16">
               <div
                  className="flex-1 overflow-hidden cursor-pointer"
                  onClick={() => isUrl(item.content) ? openUrl(item.content) : copyToClipboard(item.content)}
                >
                  <p className="whitespace-pre-wrap break-words text-sm">{item.content}</p>
               </div>

               <div className="absolute right-2 top-2 flex space-x-1 opacity-0 group-hover:opacity-100 transition-opacity">
                  <button
                    onClick={() => togglePin(item.id)}
                    className={`p-1 rounded hover:bg-gray-100 ${item.pinned ? 'text-blue-500' : 'text-gray-400'}`}
                  >
                    {item.pinned ? '★' : '☆'}
                  </button>
                  <button
                    onClick={() => deleteItem(item.id)}
                    className="p-1 rounded text-red-400 hover:bg-red-50 hover:text-red-600"
                  >
                    ✕
                  </button>
               </div>
            </div>
          ))
        )}
      </main>
    </div>
  );
}

export default App;
