import { Panel, PanelGroup, PanelResizeHandle } from "react-resizable-panels";
import { ActivityBar } from "../components/layout/ActivityBar";
import { SideBar } from "../components/layout/SideBar";
import { EditorArea } from "../components/layout/EditorArea";
import { StatusBar } from "../components/layout/StatusBar";

export const MainLayout = () => {
  return (
    <div className="h-screen w-screen flex flex-col bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100 overflow-hidden font-sans">
      <div className="flex-1 flex overflow-hidden">
        <ActivityBar />
        
        <PanelGroup direction="horizontal">
          <Panel defaultSize={20} minSize={15} maxSize={40} className="flex flex-col border-r border-gray-200 dark:border-gray-800 bg-gray-50 dark:bg-gray-900">
            <SideBar />
          </Panel>
          
          <PanelResizeHandle className="w-1 bg-transparent hover:bg-blue-500 transition-colors" />
          
          <Panel minSize={30}>
             <EditorArea />
          </Panel>
        </PanelGroup>
      </div>
      
      <StatusBar />
    </div>
  );
};
