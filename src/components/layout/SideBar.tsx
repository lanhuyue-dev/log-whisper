import React from 'react';
import { useAppStore } from '../../store/appStore';
import { DockerView } from '../sidebar/DockerView';
import { FilesView } from '../sidebar/FilesView';
import { K8sView } from '../sidebar/K8sView';

const SearchView = () => (
  <div className="flex flex-col h-full">
    <div className="h-9 flex items-center px-4 border-b border-gray-200 dark:border-gray-800">
      <span className="text-xs font-semibold uppercase tracking-wider text-gray-500">Search</span>
    </div>
    <div className="p-4 text-sm text-gray-400">Search view coming soon</div>
  </div>
);

export const SideBar = () => {
  const { activeActivity } = useAppStore();

  switch (activeActivity) {
    case 'files': return <FilesView />;
    case 'docker': return <DockerView />;
    case 'k8s': return <K8sView />;
    case 'search': return <SearchView />;
    default: return <FilesView />;
  }
};
