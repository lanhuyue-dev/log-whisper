import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/tauri';

export interface ContainerInfo {
  id: string;
  name: string;
  image: string;
  state: string;
  status: string;
}

interface DockerState {
  containers: ContainerInfo[];
  isLoading: boolean;
  error: string | null;
  loadContainers: () => Promise<void>;
}

export const useDockerStore = create<DockerState>((set) => ({
  containers: [],
  isLoading: false,
  error: null,
  loadContainers: async () => {
    set({ isLoading: true, error: null });
    try {
      console.log('🐳 Fetching containers...');
      const list = await invoke<ContainerInfo[]>('get_containers');
      set({ containers: list, isLoading: false });
    } catch (e: any) {
      console.error('Failed to fetch containers:', e);
      set({ error: e.toString(), isLoading: false });
    }
  }
}));
