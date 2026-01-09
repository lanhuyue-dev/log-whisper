import React from 'react';
import { Files, Box, Server, Search, Settings } from 'lucide-react';
import { useAppStore, Activity } from '../../store/appStore';

export const ActivityBar = () => {
  const { activeActivity, setActiveActivity } = useAppStore();

  const items: { id: Activity; icon: React.ElementType; label: string }[] = [
    { id: 'files', icon: Files, label: '文件' },
    { id: 'docker', icon: Box, label: 'Docker' },
    { id: 'k8s', icon: Server, label: 'Kubernetes' },
    { id: 'search', icon: Search, label: '搜索' },
  ];

  return (
    <div className="w-12 flex flex-col items-center py-2 bg-gray-100 dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 z-10 select-none">
      <div className="flex-1 flex flex-col gap-4">
        {items.map((item) => (
          <button
            key={item.id}
            onClick={() => setActiveActivity(item.id)}
            className={`p-2 rounded-md transition-colors relative group ${
              activeActivity === item.id
                ? 'text-blue-600 dark:text-blue-400 bg-white dark:bg-gray-700 shadow-sm'
                : 'text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'
            }`}
            title={item.label}
          >
            <item.icon size={24} strokeWidth={1.5} />
            {activeActivity === item.id && (
              <div className="absolute left-0 top-2 bottom-2 w-0.5 bg-blue-500 rounded-r-full" />
            )}
          </button>
        ))}
      </div>
      
      <div className="flex flex-col gap-4">
        <button className="p-2 text-gray-500 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-200">
          <Settings size={20} strokeWidth={1.5} />
        </button>
      </div>
    </div>
  );
};
