import { create } from 'zustand';

export interface LogTab {
  id: string;
  type: 'file' | 'docker' | 'k8s';
  title: string;
  subtitle?: string;
  target: string; // filePath or containerId or podName
  metadata?: any; // Extra info like namespace
}

interface TabState {
  tabs: LogTab[];
  activeTabId: string | null;
  
  openTab: (tab: LogTab) => void;
  closeTab: (id: string) => void;
  setActiveTab: (id: string) => void;
  closeAll: () => void;
}

export const useTabStore = create<TabState>((set, get) => ({
  tabs: [],
  activeTabId: null,

  openTab: (newTab) => {
    const { tabs } = get();
    // 检查是否已经打开
    const existing = tabs.find(t => t.id === newTab.id);
    if (existing) {
      set({ activeTabId: existing.id });
      return;
    }
    
    set({ 
      tabs: [...tabs, newTab],
      activeTabId: newTab.id
    });
  },

  closeTab: (id) => {
    const { tabs, activeTabId } = get();
    const newTabs = tabs.filter(t => t.id !== id);
    
    // 如果关闭的是当前激活的 Tab，尝试激活下一个
    let newActiveId = activeTabId;
    if (activeTabId === id) {
      newActiveId = newTabs.length > 0 ? newTabs[newTabs.length - 1].id : null;
    }
    
    set({ tabs: newTabs, activeTabId: newActiveId });
  },

  setActiveTab: (id) => set({ activeTabId: id }),
  
  closeAll: () => set({ tabs: [], activeTabId: null }),
}));
