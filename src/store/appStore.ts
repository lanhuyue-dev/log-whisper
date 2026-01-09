import { create } from 'zustand';

export type Activity = 'files' | 'docker' | 'k8s' | 'search';

interface AppState {
  activeActivity: Activity;
  setActiveActivity: (activity: Activity) => void;
  
  theme: 'light' | 'dark';
  setTheme: (theme: 'light' | 'dark') => void;
  toggleTheme: () => void;
}

export const useAppStore = create<AppState>((set) => ({
  activeActivity: 'files',
  setActiveActivity: (activity) => set({ activeActivity: activity }),
  
  theme: 'light',
  setTheme: (theme) => set({ theme }),
  toggleTheme: () => set((state) => {
    const newTheme = state.theme === 'light' ? 'dark' : 'light';
    // 更新 DOM
    document.documentElement.classList.toggle('dark', newTheme === 'dark');
    return { theme: newTheme };
  }),
}));
