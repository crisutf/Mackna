import React from 'react';
import { useGameStore } from '../stores/gameStore';
import { useTranslation } from '../utils/translations';

const ErrorModal: React.FC = () => {
    const { errorMsg, setErrorMsg } = useGameStore();
    const { t } = useTranslation();

    if (!errorMsg) return null;

    return (
        <div className="fixed inset-0 z-[1000] flex items-center justify-center bg-black/98 text-white p-6 animate-fade-in">
            {/* Darker and more intense background effects */}
            <div className="absolute top-0 left-0 w-full h-full bg-[url('https://cdn.crisu.qzz.io/services/leilos/logo/logo.jpg')] bg-cover bg-center opacity-10 scale-105 blur-sm"></div>
            <div className="absolute inset-0 bg-gradient-to-t from-black via-black/95 to-transparent"></div>

            <div className="relative z-10 max-w-lg w-full bg-zinc-900/40 backdrop-blur-2xl border border-white/5 rounded-[2.5rem] p-10 shadow-[0_0_100px_rgba(0,0,0,0.8)] flex flex-col items-center text-center gap-8 animate-fade-in-up">
                {/* Refined Logo and Name */}
                <div className="flex items-center gap-5">
                    <div className="w-20 h-20 rounded-3xl bg-gradient-to-br from-red-500 to-red-600 p-0.5 shadow-[0_0_40px_rgba(220,38,38,0.25)] flex items-center justify-center">
                        <div className="w-full h-full bg-black rounded-[1.4rem] overflow-hidden flex items-center justify-center">
                            <img src="https://cdn.crisu.qzz.io/services/leilos/logo/logo.jpg" alt="Leilos Logo" className="w-full h-full object-cover" />
                        </div>
                    </div>
                    <div className="text-left">
                        <h2 className="text-3xl font-display font-black tracking-[0.15em] text-white uppercase leading-none">Leilos</h2>
                        <h2 className="text-3xl font-display font-black tracking-[0.15em] text-red-500 uppercase leading-none mt-1">Launcher</h2>
                    </div>
                </div>

                <div className="space-y-3">
                    <h1 className="text-4xl font-display font-black tracking-widest text-white uppercase">
                        {t('home.errorDetected').split(' ')[0]} <span className="text-gray-500">{t('home.errorDetected').split(' ')[1]}</span>
                    </h1>
                    <div className="h-1.5 w-24 bg-gradient-to-r from-transparent via-red-500 to-transparent mx-auto rounded-full shadow-[0_0_15px_rgba(220,38,38,0.5)]"></div>
                </div>
                
                <div className="glass-panel-dark p-6 rounded-2xl border border-white/5 bg-black/40 w-full">
                    <p className="text-gray-400 font-mono text-sm leading-relaxed break-words italic">
                        "{errorMsg}"
                    </p>
                </div>

                <button 
                    onClick={() => setErrorMsg(null)}
                    className="w-full py-5 bg-gradient-to-r from-red-600 to-red-500 text-white font-black text-lg rounded-2xl hover:from-red-500 hover:to-red-400 transition-all duration-300 font-display tracking-[0.3em] shadow-[0_10px_40px_rgba(220,38,38,0.3)] hover:shadow-[0_15px_50px_rgba(220,38,38,0.45)] transform hover:-translate-y-1 active:scale-[0.98] uppercase"
                >
                    {t('home.accept')}
                </button>
                
                <div className="flex items-center gap-3 opacity-30 group hover:opacity-100 transition-opacity">
                    <img src="https://cdn.crisu.qzz.io/services/leilos/logo/logo.jpg" alt="Logo" className="w-4 h-4 grayscale rounded-sm" />
                    <p className="text-[10px] text-gray-500 font-mono tracking-[0.2em] uppercase">
                        Leilos • Crisutf
                    </p>
                </div>
            </div>
        </div>
    );
};

export default ErrorModal;
