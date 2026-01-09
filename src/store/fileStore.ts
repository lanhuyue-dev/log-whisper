import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/tauri';
import { homeDir, sep, dirname } from '@tauri-apps/api/path';

export interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number | null;
}

interface FileState {
  currentPath: string; // 必须是有效的 string
  entries: FileEntry[];
  isLoading: boolean;
  error: string | null;
  
  init: () => Promise<void>;
  loadDir: (path: string) => Promise<void>;
  goUp: () => Promise<void>;
}

export const useFileStore = create<FileState>((set, get) => ({
  currentPath: '',
  entries: [],
  isLoading: false,
  error: null,

  init: async () => {
    try {
      const home = await homeDir();
      await get().loadDir(home);
    } catch (e) {
      console.error(e);
    }
  },

  loadDir: async (path) => {
    set({ isLoading: true, error: null });
    try {
      const entries = await invoke<FileEntry[]>('read_dir', { path });
      set({ entries, currentPath: path, isLoading: false });
    } catch (e: any) {
      set({ error: e.toString(), isLoading: false });
    }
  },
  
  goUp: async () => {
    const { currentPath, loadDir } = get();
    if (!currentPath) return;
    
    try {
      // 简单的父目录计算
      const parent = await dirname(currentPath);
      if (parent !== currentPath) {
        await loadDir(parent);
      }
    } catch (e) {
      console.error(e);
    }
  }
}));
