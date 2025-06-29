import React, { useState } from 'react';
import { useDreamEmulator } from './store';
import MainMenu from './components/MainMenu/MainMenu';
import TopDownGameCreator from './components/GameCreator/TopDownGameCreator';
import './styles/global.css';

type ViewState = 'menu' | 'create-game' | 'game-editor' | 'server-manager' | 'game-library';

function App() {
  const [currentView, setCurrentView] = useState<ViewState>('menu');
  const [selectedEngineType, setSelectedEngineType] = useState<'topdown' | 'sidescroller' | '3d' | null>(null);
  
  const { currentProject, createProject } = useDreamEmulator();

  const handleCreateNewGame = async (engineType: 'topdown' | 'sidescroller' | '3d') => {
    setSelectedEngineType(engineType);
    
    // For now, create a default project and go straight to editor
    // In production, you'd show a project configuration dialog
    const project = await createProject(`New ${engineType} Game`, engineType);
    setCurrentView('game-editor');
  };

  const handleMenuNavigation = (destination: string) => {
    switch (destination) {
      case 'create':
        // The MainMenu component handles engine selection internally
        break;
      case 'servers':
        setCurrentView('server-manager');
        break;
      case 'manage':
        setCurrentView('game-library');
        break;
    }
  };

  const renderView = () => {
    switch (currentView) {
      case 'menu':
        return (
          <MainMenu 
            onNavigate={handleMenuNavigation}
            onEngineSelect={handleCreateNewGame}
          />
        );
      
      case 'game-editor':
        // For now, only show top-down editor
        if (selectedEngineType === 'topdown' || currentProject?.engineType === 'topdown') {
          return <TopDownGameCreator onExit={() => setCurrentView('menu')} />;
        }
        // Placeholder for other engine types
        return (
          <div style={{ 
            display: 'flex', 
            alignItems: 'center', 
            justifyContent: 'center', 
            height: '100vh',
            background: '#0f0f1e',
            color: '#fff'
          }}>
            <div style={{ textAlign: 'center' }}>
              <h2>🚧 Under Construction</h2>
              <p>{selectedEngineType} editor coming soon!</p>
              <button 
                onClick={() => setCurrentView('menu')}
                style={{
                  marginTop: '20px',
                  padding: '10px 20px',
                  background: '#8b5cf6',
                  color: '#fff',
                  borderRadius: '8px',
                  fontSize: '16px'
                }}
              >
                Back to Menu
              </button>
            </div>
          </div>
        );
      
      case 'server-manager':
        return (
          <div style={{ 
            display: 'flex', 
            alignItems: 'center', 
            justifyContent: 'center', 
            height: '100vh',
            background: '#0f0f1e',
            color: '#fff'
          }}>
            <div style={{ textAlign: 'center' }}>
              <h2>🖥️ Server Manager</h2>
              <p>Server management interface coming soon!</p>
              <button 
                onClick={() => setCurrentView('menu')}
                style={{
                  marginTop: '20px',
                  padding: '10px 20px',
                  background: '#8b5cf6',
                  color: '#fff',
                  borderRadius: '8px',
                  fontSize: '16px'
                }}
              >
                Back to Menu
              </button>
            </div>
          </div>
        );
      
      case 'game-library':
        return (
          <div style={{ 
            display: 'flex', 
            alignItems: 'center', 
            justifyContent: 'center', 
            height: '100vh',
            background: '#0f0f1e',
            color: '#fff'
          }}>
            <div style={{ textAlign: 'center' }}>
              <h2>📚 Game Library</h2>
              <p>Your games will appear here!</p>
              <button 
                onClick={() => setCurrentView('menu')}
                style={{
                  marginTop: '20px',
                  padding: '10px 20px',
                  background: '#8b5cf6',
                  color: '#fff',
                  borderRadius: '8px',
                  fontSize: '16px'
                }}
              >
                Back to Menu
              </button>
            </div>
          </div>
        );
      
      default:
        return <MainMenu onNavigate={handleMenuNavigation} onEngineSelect={handleCreateNewGame} />;
    }
  };

  return (
    <div className="app">
      {renderView()}
    </div>
  );
}

export default App;