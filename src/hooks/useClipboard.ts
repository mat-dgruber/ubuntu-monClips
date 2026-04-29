import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export interface ClipItem {
  id: number;
  content: string;
  created_at: number;
  pinned: boolean;
}

export function useClipboard() {
  const [items, setItems] = useState<ClipItem[]>([]);
  const [searchQuery, setSearchQuery] = useState('');

  const fetchItems = useCallback(async (query?: string) => {
    try {
      const result = await invoke<ClipItem[]>('get_clipboard_items', {
        searchQuery: query || null
      });
      setItems(result);
    } catch (e) {
      console.error("Failed to fetch items", e);
    }
  }, []);

  useEffect(() => {
    fetchItems(searchQuery);
  }, [searchQuery, fetchItems]);

  useEffect(() => {
    const unlisten = listen('clipboard_updated', () => {
      fetchItems(searchQuery);
    });
    return () => {
      unlisten.then(f => f());
    };
  }, [searchQuery, fetchItems]);

  const togglePin = async (id: number) => {
    await invoke('toggle_item_pin', { id });
    await fetchItems(searchQuery);
  };

  const deleteItem = async (id: number) => {
    await invoke('delete_clipboard_item', { id });
    await fetchItems(searchQuery);
  };

  const copyToClipboard = async (content: string) => {
    await invoke('write_to_clipboard', { content });
  };

  const openUrlFunc = async (url: string) => {
      // Using tauri shell api via dynamic import
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
    openUrl: openUrlFunc
  };
}
