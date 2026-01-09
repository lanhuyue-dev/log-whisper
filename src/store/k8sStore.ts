import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/tauri';

export interface K8sPodInfo {
  name: string;
  namespace: string;
  status: string;
  restart_count: number;
}

interface K8sState {
  namespaces: string[];
  currentNamespace: string;
  pods: K8sPodInfo[];
  isLoading: boolean;
  error: string | null;
  
  loadNamespaces: () => Promise<void>;
  loadPods: (ns: string) => Promise<void>;
  setNamespace: (ns: string) => void;
}

export const useK8sStore = create<K8sState>((set, get) => ({
  namespaces: ['default'],
  currentNamespace: 'default',
  pods: [],
  isLoading: false,
  error: null,

  loadNamespaces: async () => {
    try {
      const list = await invoke<{name: string}[]>('get_k8s_namespaces');
      set({ namespaces: list.map(n => n.name) });
    } catch (e) {
      console.error(e);
      set({ namespaces: ['default', 'kube-system'] });
    }
  },

  loadPods: async (ns) => {
    set({ isLoading: true, error: null });
    try {
      const list = await invoke<K8sPodInfo[]>('get_k8s_pods', { namespace: ns });
      set({ pods: list, isLoading: false });
    } catch (e: any) {
      set({ error: e.toString(), isLoading: false });
    }
  },
  
  setNamespace: (ns) => {
    set({ currentNamespace: ns });
    get().loadPods(ns);
  }
}));
