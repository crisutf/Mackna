import React from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useGameStore } from '../stores/gameStore';
import { useTranslation } from '../utils/translations';

const SplashScreen: React.FC = () => {
    const { splashMessage } = useGameStore();
    const { t } = useTranslation();

    if (!splashMessage) return null;

    return (
        <AnimatePresence>
            <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                className="fixed inset-0 z-[9999] flex flex-col items-center justify-center bg-black overflow-hidden"
            >
                {/* Background Image */}
                <div 
                    className="absolute inset-0 bg-cover bg-center bg-no-repeat transition-all duration-700"
                    style={{ 
                        backgroundImage: `url('/Fort_Load_SeasonKeyArt.png')`,
                    }}
                >
                    {/* Subtle gradient for text readability without hiding the art */}
                    <div className="absolute inset-0 bg-gradient-to-t from-black/60 via-transparent to-transparent"></div>
                </div>

                {/* Content Container */}
                <div className="relative z-10 w-full h-full flex flex-col">
                    {/* Loading Text Bottom-Right */}
                    <div className="absolute bottom-16 right-16 flex flex-col items-end gap-3">
                        <motion.span 
                            key={splashMessage}
                            initial={{ opacity: 0 }}
                            animate={{ opacity: 1 }}
                            className="text-white text-3xl font-black italic uppercase tracking-[0.1em] drop-shadow-[0_2px_10px_rgba(0,0,0,0.8)]"
                        >
                            {t(splashMessage)}
                        </motion.span>
                        
                        {/* Progress Line */}
                        <div className="w-56 h-[3px] bg-white/10 rounded-full overflow-hidden">
                            <motion.div 
                                className="h-full bg-white shadow-[0_0_15px_rgba(255,255,255,0.8)]"
                                animate={{ 
                                    width: ["0%", "100%"],
                                }}
                                transition={{ 
                                    duration: 2, 
                                    ease: "linear",
                                    repeat: Infinity 
                                }}
                            />
                        </div>
                    </div>
                </div>

                {/* Top-Left Small Logo */}
                <div className="absolute top-10 left-10 z-20">
                    <div className="w-16 h-16 bg-black/40 backdrop-blur-md rounded-xl border border-white/5 flex items-center justify-center overflow-hidden group">
                        <img 
                            src="/logo.jpg" 
                            alt="Leilos" 
                            className="w-10 opacity-60 group-hover:opacity-100 transition-opacity"
                        />
                    </div>
                </div>
            </motion.div>
        </AnimatePresence>
    );
};

export default SplashScreen;
