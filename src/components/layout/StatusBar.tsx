import React from 'react';
import { Wifi, Bell } from 'lucide-react';

export const StatusBar = () => {
  return (
    <div className="h-6 bg-blue-600 dark:bg-blue-800 text-white flex items-center px-3 text-xs justify-between select-none">
      <div className="flex items-center gap-4">
        <div className="flex items-center gap-1">
          <Wifi size={12} />
          <span>已连接</span>
        </div>
        <span>master*</span>
      </div>
      
      <div className="flex items-center gap-4">
        <span>UTF-8</span>
        <div className="flex items-center gap-1 cursor-pointer hover:bg-blue-700 px-1 rounded">
          <Bell size={12} />
          <span>0</span>
        </div>
      </div>
    </div>
  );
};
