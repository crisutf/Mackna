import React from 'react';
import { appWindow } from '@tauri-apps/api/window';
import Sidebar from './Sidebar';
import { useTranslation } from '../utils/translations';

interface LayoutProps {
    children: React.ReactNode;
    currentView: string;
    onChangeView: (view: string) => void;
}

const Layout: React.FC<LayoutProps> = ({ children, currentView, onChangeView }) => {
    const { t } = useTranslation();
    return (
        <div className="flex w-full h-screen bg-bg-dark overflow-hidden text-white font-sans selection:bg-gold-primary selection:text-black">
            <Sidebar currentView={currentView} onChangeView={onChangeView} />
            <div className="flex-1 flex flex-col overflow-hidden relative">
                {/* Top Bar / Drag Region */}
                <div data-tauri-drag-region className="h-10 w-full shrink-0 flex items-center justify-between px-4 z-50 bg-transparent">
                    {/* Left: Empty space for dragging */}
                    <div className="flex-1 h-full cursor-default"></div>
                    
                    {/* Window Controls */}
                    <div className="flex items-center gap-1">
                        <button 
                            onClick={() => appWindow.minimize()}
                            className="p-2 text-gray-500 hover:text-white hover:bg-white/10 rounded-md transition-all duration-200"
                            title={t('nav.minimize')}
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><line x1="5" y1="12" x2="19" y2="12"></line></svg>
                        </button>
                        <button 
                            onClick={() => appWindow.close()}
                            className="p-2 text-gray-500 hover:text-white hover:bg-red-500/80 rounded-md transition-all duration-200"
                            title={t('nav.close')}
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
                        </button>
                    </div>
                </div>
                
                {/* Main Content Area */}
                <div className="flex-1 overflow-y-auto overflow-x-hidden p-0 relative z-10 scroll-smooth">
                    <div className="h-full">
                        {children}
                    </div>
                </div>
            </div>
        </div>
    );
};

export default Layout;
