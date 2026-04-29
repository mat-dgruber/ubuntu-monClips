import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import toast from 'react-hot-toast';

export interface ClipItem {
  id: number;
  content: string;
  created_at: number;
  pinned: boolean;
  category: string;
}

const ITEMS_PER_PAGE = 50;

export function useClipboard() {
  const [items, setItems] = useState<ClipItem[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [debouncedQuery, setDebouncedQuery] = useState('');
  const [hasMore, setHasMore] = useState(true);
  const [isLoading, setIsLoading] = useState(false);
  const offsetRef = useRef(0);

  // Debounce search query
  useEffect(() => {
    const timer = setTimeout(() => {
      setDebouncedQuery(searchQuery);
    }, 300);
    return () => clearTimeout(timer);
  }, [searchQuery]);

  const fetchItems = useCallback(async (query: string, isAppend: boolean = false) => {
    if (isLoading) return;
    
    setIsLoading(true);
    try {
      const currentOffset = isAppend ? offsetRef.current : 0;
      const result = await invoke<ClipItem[]>('get_clipboard_items', {
        searchQuery: query || null,
        limit: ITEMS_PER_PAGE,
        offset: currentOffset
      });

      if (isAppend) {
        setItems(prev => [...prev, ...result]);
        offsetRef.current += result.length;
      } else {
        setItems(result);
        offsetRef.current = result.length;
      }
      
      setHasMore(result.length === ITEMS_PER_PAGE);
    } catch (e) {
      console.error("Failed to fetch items", e);
    } finally {
      setIsLoading(false);
    }
  }, [isLoading]);

  // Initial fetch or search query change
  useEffect(() => {
    fetchItems(debouncedQuery, false);
  }, [debouncedQuery]);

  // Listen for background updates (new clips)
  useEffect(() => {
    const unlisten = listen('clipboard_updated', () => {
      // For background updates, we usually want to refresh the first page
      fetchItems(debouncedQuery, false);
    });
    return () => {
      unlisten.then(f => f());
    };
  }, [debouncedQuery, fetchItems]);

  const loadMore = useCallback(() => {
    if (hasMore && !isLoading) {
      fetchItems(debouncedQuery, true);
    }
  }, [hasMore, isLoading, debouncedQuery, fetchItems]);

  const togglePin = async (id: number) => {
    await invoke('toggle_item_pin', { id });
    await fetchItems(debouncedQuery, false);
  };

  const deleteItem = async (id: number) => {
    await invoke('delete_clipboard_item', { id });
    await fetchItems(debouncedQuery, false);
  };

  const copyToClipboard = async (content: string) => {
    try {
      await invoke('write_to_clipboard', { content });
      toast.success('Copied to clipboard!');
    } catch (e) {
      console.error("Failed to copy", e);
      toast.error('Failed to copy to clipboard');
    }
  };

  const openUrlFunc = async (url: string) => {
      const { openUrl } = await import('@tauri-apps/plugin-opener');
      await openUrl(url);
  }

  return {
    items,
    searchQuery,
    setSearchQuery,
    togglePin,
    deleteItem,
    copyToClipboard,
    openUrl: openUrlFunc,
    loadMore,
    hasMore,
    isLoading
  };
}
