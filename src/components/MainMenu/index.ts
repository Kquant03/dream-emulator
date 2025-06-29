import React, { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Sparkles, Plus, Server, Folder, ChevronRight, Gamepad2, Cpu, Box } from 'lucide-react';

const MainMenu = () => {
  const [hoveredButton, setHoveredButton] = useState(null);
  const [currentView, setCurrentView] = useState('menu');
  const [particles, setParticles] = useState([]);

  // Generate floating particles for ambiance
  useEffect(() => {
    const newParticles = Array.from({ length: 20 }, (_, i) => ({
      id: i,
      x: Math.random() * 100,
      y: Math.random() * 100,
      size: Math.random() * 4 + 2,
      duration: Math.random() * 20 + 20
    }));
    setParticles(newParticles);
  }, []);

  const menuItems = [
    {
      id: 'create',
      label: 'Create New Game',
      icon: Plus,
      description: 'Start building your dream game from scratch',
      gradient: 'from-purple-500 to-pink-500'
    },
    {
      id: 'servers',
      label: 'Join Games/Manage Servers',
      icon: Server,
      description: 'Host and join multiplayer experiences',
      gradient: 'from-blue-500 to-cyan-500'
    },
    {
      id: 'manage',
      label: 'Manage Existing Games',
      icon: Folder,
      description: 'Continue working on your projects',
      gradient: 'from-green-500 to-emerald-500'
    }
  ];

  const engineTypes = [
    {
      id: 'topdown',
      label: 'Top-Down',
      icon: Gamepad2,
      description: 'Create games like Zelda, Stardew Valley, or Pokemon',
      preview: '🎮'
    },
    {
      id: 'sidescroller',
      label: 'Side-Scroller',
      icon: ChevronRight,
      description: 'Build platformers and metroidvanias',
      preview: '🏃'
    },
    {
      id: '3d',
      label: '3D First-Person',
      icon: Box,
      description: 'Craft immersive 3D worlds like Daggerfall',
      preview: '🎯'
    }
  ];

  const handleMenuClick = (id) => {
    if (id === 'create') {
      setCurrentView('engineSelect');
    } else {
      // Handle other menu items
      console.log(`Clicked: ${id}`);
    }
  };

  const handleEngineSelect = (engineType) => {
    console.log(`Selected engine: ${engineType}`);
    // This is where you'd transition to the game creation interface
  };

  return (
    <div className="dream-emulator-container">
      <style jsx>{`
        .dream-emulator-container {
          min-height: 100vh;
          background: linear-gradient(135deg, #0f0f1e 0%, #1a1a2e 100%);
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          position: relative;
          overflow: hidden;
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        }

        .particle {
          position: absolute;
          background: radial-gradient(circle, rgba(255,255,255,0.8) 0%, rgba(255,255,255,0) 70%);
          border-radius: 50%;
          pointer-events: none;
        }

        .menu-button {
          background: rgba(255, 255, 255, 0.05);
          border: 1px solid rgba(255, 255, 255, 0.1);
          backdrop-filter: blur(10px);
          border-radius: 16px;
          padding: 24px 32px;
          cursor: pointer;
          transition: all 0.3s ease;
          width: 400px;
          position: relative;
          overflow: hidden;
        }

        .menu-button:hover {
          background: rgba(255, 255, 255, 0.1);
          border-color: rgba(255, 255, 255, 0.2);
          transform: translateY(-2px);
          box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
        }

        .gradient-overlay {
          position: absolute;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          opacity: 0;
          transition: opacity 0.3s ease;
          pointer-events: none;
        }

        .menu-button:hover .gradient-overlay {
          opacity: 0.1;
        }

        .engine-card {
          background: rgba(255, 255, 255, 0.05);
          border: 1px solid rgba(255, 255, 255, 0.1);
          backdrop-filter: blur(10px);
          border-radius: 24px;
          padding: 32px;
          cursor: pointer;
          transition: all 0.3s ease;
          text-align: center;
          position: relative;
          overflow: hidden;
        }

        .engine-card:hover {
          background: rgba(255, 255, 255, 0.1);
          border-color: rgba(255, 255, 255, 0.3);
          transform: translateY(-4px) scale(1.02);
          box-shadow: 0 20px 40px rgba(0, 0, 0, 0.4);
        }

        .preview-emoji {
          font-size: 64px;
          margin-bottom: 16px;
          filter: grayscale(0.5);
          transition: filter 0.3s ease;
        }

        .engine-card:hover .preview-emoji {
          filter: grayscale(0);
        }
      `}</style>

      {/* Floating particles */}
      {particles.map((particle) => (
        <motion.div
          key={particle.id}
          className="particle"
          style={{
            width: particle.size,
            height: particle.size,
            left: `${particle.x}%`,
            top: `${particle.y}%`,
          }}
          animate={{
            y: [0, -1000],
            opacity: [0, 1, 0],
          }}
          transition={{
            duration: particle.duration,
            repeat: Infinity,
            ease: "linear",
          }}
        />
      ))}

      <AnimatePresence mode="wait">
        {currentView === 'menu' && (
          <motion.div
            key="menu"
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -20 }}
            transition={{ duration: 0.5 }}
            style={{ width: '100%', maxWidth: '600px' }}
          >
            {/* Logo */}
            <motion.div
              initial={{ opacity: 0, scale: 0.9 }}
              animate={{ opacity: 1, scale: 1 }}
              transition={{ delay: 0.2, duration: 0.5 }}
              style={{ textAlign: 'center', marginBottom: '60px' }}
            >
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '16px', marginBottom: '8px' }}>
                <Sparkles size={40} color="#fff" />
                <h1 style={{ fontSize: '48px', fontWeight: '700', color: '#fff', margin: 0 }}>
                  Dream Emulator
                </h1>
              </div>
              <p style={{ color: 'rgba(255,255,255,0.6)', fontSize: '18px' }}>
                Create games as easily as you imagine them
              </p>
            </motion.div>

            {/* Menu buttons */}
            <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
              {menuItems.map((item, index) => {
                const Icon = item.icon;
                return (
                  <motion.div
                    key={item.id}
                    initial={{ opacity: 0, x: -20 }}
                    animate={{ opacity: 1, x: 0 }}
                    transition={{ delay: 0.3 + index * 0.1, duration: 0.5 }}
                  >
                    <div
                      className="menu-button"
                      onMouseEnter={() => setHoveredButton(item.id)}
                      onMouseLeave={() => setHoveredButton(null)}
                      onClick={() => handleMenuClick(item.id)}
                    >
                      <div
                        className="gradient-overlay"
                        style={{
                          background: `linear-gradient(135deg, ${item.gradient.split(' ')[1]} 0%, ${item.gradient.split(' ')[3]} 100%)`,
                        }}
                      />
                      <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
                        <Icon size={24} color="#fff" />
                        <div style={{ textAlign: 'left' }}>
                          <h3 style={{ margin: 0, color: '#fff', fontSize: '20px', fontWeight: '600' }}>
                            {item.label}
                          </h3>
                          <p style={{ margin: '4px 0 0 0', color: 'rgba(255,255,255,0.6)', fontSize: '14px' }}>
                            {item.description}
                          </p>
                        </div>
                      </div>
                    </div>
                  </motion.div>
                );
              })}
            </div>
          </motion.div>
        )}

        {currentView === 'engineSelect' && (
          <motion.div
            key="engineSelect"
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -20 }}
            transition={{ duration: 0.5 }}
            style={{ width: '100%', maxWidth: '1000px' }}
          >
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{ delay: 0.2 }}
              style={{ textAlign: 'center', marginBottom: '60px' }}
            >
              <h2 style={{ fontSize: '36px', fontWeight: '600', color: '#fff', margin: '0 0 16px 0' }}>
                Choose Your Game Type
              </h2>
              <p style={{ color: 'rgba(255,255,255,0.6)', fontSize: '18px' }}>
                Select the perspective that best fits your vision
              </p>
            </motion.div>

            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '24px' }}>
              {engineTypes.map((engine, index) => {
                const Icon = engine.icon;
                return (
                  <motion.div
                    key={engine.id}
                    initial={{ opacity: 0, y: 20 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ delay: 0.3 + index * 0.1 }}
                    className="engine-card"
                    onClick={() => handleEngineSelect(engine.id)}
                  >
                    <div className="preview-emoji">{engine.preview}</div>
                    <Icon size={32} color="#fff" style={{ marginBottom: '16px' }} />
                    <h3 style={{ fontSize: '24px', fontWeight: '600', color: '#fff', margin: '0 0 8px 0' }}>
                      {engine.label}
                    </h3>
                    <p style={{ color: 'rgba(255,255,255,0.7)', fontSize: '14px', margin: 0 }}>
                      {engine.description}
                    </p>
                  </motion.div>
                );
              })}
            </div>

            <motion.button
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{ delay: 0.6 }}
              onClick={() => setCurrentView('menu')}
              style={{
                background: 'none',
                border: '1px solid rgba(255,255,255,0.2)',
                color: 'rgba(255,255,255,0.8)',
                padding: '12px 24px',
                borderRadius: '8px',
                marginTop: '40px',
                cursor: 'pointer',
                fontSize: '16px',
                transition: 'all 0.3s ease',
              }}
              whileHover={{ borderColor: 'rgba(255,255,255,0.4)', color: '#fff' }}
            >
              ← Back to Menu
            </motion.button>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
};

export default MainMenu;