import React, { useState, useRef, useEffect } from 'react';
import { DndContext, DragEndEvent, useDraggable, useDroppable } from '@dnd-kit/core';
import { CSS } from '@dnd-kit/utilities';
import { 
  Move, 
  MousePointer, 
  Grid3x3, 
  Play, 
  Pause, 
  Save,
  Folder,
  Image,
  Music,
  Code,
  Plus,
  Settings,
  Layers,
  Eye,
  EyeOff
} from 'lucide-react';
import * as PIXI from 'pixi.js';
import styles from './TopDownGameCreator.module.css';

// Asset Panel Component
const AssetPanel = ({ assets, onDragStart }) => {
  const [selectedCategory, setSelectedCategory] = useState('sprites');
  
  const categories = [
    { id: 'sprites', label: 'Sprites', icon: Image },
    { id: 'tiles', label: 'Tiles', icon: Grid3x3 },
    { id: 'audio', label: 'Audio', icon: Music },
    { id: 'scripts', label: 'Scripts', icon: Code },
  ];

  return (
    <div className={styles.assetPanel}>
      <div className={styles.assetCategories}>
        {categories.map(cat => {
          const Icon = cat.icon;
          return (
            <button
              key={cat.id}
              className={`${styles.categoryBtn} ${selectedCategory === cat.id ? styles.active : ''}`}
              onClick={() => setSelectedCategory(cat.id)}
            >
              <Icon size={16} />
              <span>{cat.label}</span>
            </button>
          );
        })}
      </div>
      
      <div className={styles.assetGrid}>
        {/* Demo assets */}
        {[1, 2, 3, 4, 5, 6].map(i => (
          <DraggableAsset key={i} id={`asset-${i}`} />
        ))}
        
        <button className={styles.addAssetBtn}>
          <Plus size={24} />
          <span>Add Asset</span>
        </button>
      </div>
    </div>
  );
};

// Draggable Asset Component
const DraggableAsset = ({ id }) => {
  const { attributes, listeners, setNodeRef, transform, isDragging } = useDraggable({
    id,
  });

  const style = {
    transform: CSS.Translate.toString(transform),
    opacity: isDragging ? 0.5 : 1,
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      {...listeners}
      {...attributes}
      className={styles.draggableAsset}
    >
      <div className={styles.assetPreview}>
        🎮
      </div>
      <span className={styles.assetName}>Asset {id}</span>
    </div>
  );
};

// Game Canvas Component
const GameCanvas = ({ onDrop }) => {
  const canvasRef = useRef<HTMLDivElement>(null);
  const pixiApp = useRef<PIXI.Application | null>(null);
  const { setNodeRef } = useDroppable({ id: 'game-canvas' });

  useEffect(() => {
    if (canvasRef.current && !pixiApp.current) {
      // Create PIXI Application
      const app = new PIXI.Application({
        width: 800,
        height: 600,
        backgroundColor: 0x1a1a2e,
        antialias: true,
      });

      // Add the canvas to the DOM
      canvasRef.current.appendChild(app.view as HTMLCanvasElement);
      pixiApp.current = app;

      // Create grid graphics
      const grid = new PIXI.Graphics();
      grid.lineStyle(1, 0x333333, 0.5);
      
      // Draw vertical lines
      for (let x = 0; x <= 800; x += 32) {
        grid.moveTo(x, 0);
        grid.lineTo(x, 600);
      }
      
      // Draw horizontal lines
      for (let y = 0; y <= 600; y += 32) {
        grid.moveTo(0, y);
        grid.lineTo(800, y);
      }
      
      app.stage.addChild(grid);

      // Cleanup function
      return () => {
        if (pixiApp.current) {
          pixiApp.current.destroy(true, { children: true, texture: true, baseTexture: true });
          pixiApp.current = null;
        }
      };
    }
  }, []);

  return (
    <div ref={setNodeRef} className={styles.gameCanvasContainer}>
      <div ref={canvasRef} />
    </div>
  );
};

// Object Inspector Component
const ObjectInspector = ({ selectedObject }) => {
  if (!selectedObject) {
    return (
      <div className={`${styles.objectInspector} ${styles.empty}`}>
        <p>Select an object to edit its properties</p>
      </div>
    );
  }

  return (
    <div className={styles.objectInspector}>
      <h3>Object Properties</h3>
      
      <div className={styles.propertyGroup}>
        <label>Name</label>
        <input type="text" defaultValue={selectedObject.name || 'GameObject'} />
      </div>
      
      <div className={styles.propertyGroup}>
        <label>Position</label>
        <div className={styles.propertyRow}>
          <input type="number" placeholder="X" defaultValue={0} />
          <input type="number" placeholder="Y" defaultValue={0} />
        </div>
      </div>
      
      <div className={styles.propertyGroup}>
        <label>Scale</label>
        <div className={styles.propertyRow}>
          <input type="number" placeholder="X" defaultValue={1} step="0.1" />
          <input type="number" placeholder="Y" defaultValue={1} step="0.1" />
        </div>
      </div>
      
      <div className={styles.propertyGroup}>
        <label>Rotation</label>
        <input type="number" defaultValue={0} step="15" />
      </div>
      
      <button className={styles.addComponentBtn}>
        <Plus size={16} />
        Add Component
      </button>
    </div>
  );
};

// Scene Hierarchy Component
const SceneHierarchy = ({ objects }) => {
  const [expandedItems, setExpandedItems] = useState(new Set());
  
  return (
    <div className={styles.sceneHierarchy}>
      <div className={styles.hierarchyHeader}>
        <h3>Scene</h3>
        <button className={styles.iconBtn}>
          <Plus size={16} />
        </button>
      </div>
      
      <div className={styles.hierarchyTree}>
        <div className={styles.hierarchyItem}>
          <Eye size={14} />
          <span>Main Camera</span>
        </div>
        <div className={styles.hierarchyItem}>
          <Eye size={14} />
          <span>Player</span>
        </div>
        {/* More items would be mapped here */}
      </div>
    </div>
  );
};

interface TopDownGameCreatorProps {
  onExit?: () => void;
}

// Main Top-Down Game Creator Component
const TopDownGameCreator = ({ onExit }: TopDownGameCreatorProps) => {
  const [tool, setTool] = useState('select');
  const [isPlaying, setIsPlaying] = useState(false);
  const [selectedObject, setSelectedObject] = useState(null);
  const [showGrid, setShowGrid] = useState(true);

  const tools = [
    { id: 'select', icon: MousePointer, tooltip: 'Select' },
    { id: 'move', icon: Move, tooltip: 'Move' },
  ];

  const handleDragEnd = (event: DragEndEvent) => {
    if (event.over && event.over.id === 'game-canvas') {
      console.log('Dropped asset:', event.active.id);
      // Here you would add the asset to the scene
    }
  };

  return (
    <DndContext onDragEnd={handleDragEnd}>
      <div className={styles.container}>
        {/* Toolbar */}
        <div className={styles.toolbar}>
          {tools.map(t => {
            const Icon = t.icon;
            return (
              <button
                key={t.id}
                className={`${styles.toolBtn} ${t.id === tool ? styles.active : ''}`}
                onClick={() => setTool(t.id)}
                title={t.tooltip}
              >
                <Icon size={20} />
              </button>
            );
          })}
          
          <div className={styles.toolbarSeparator} />
          
          <button
            className={styles.toolBtn}
            onClick={() => setShowGrid(!showGrid)}
            title="Toggle Grid"
          >
            <Grid3x3 size={20} />
          </button>
          
          <div className={styles.toolbarSeparator} />
          
          <button
            className={`${styles.toolBtn} ${isPlaying ? styles.active : ''}`}
            onClick={() => setIsPlaying(!isPlaying)}
            title={isPlaying ? 'Stop' : 'Play'}
          >
            {isPlaying ? <Pause size={20} /> : <Play size={20} />}
          </button>
          
          <button className={styles.toolBtn} title="Save">
            <Save size={20} />
          </button>
          
          <button className={styles.toolBtn} onClick={onExit} title="Exit to Menu">
            <Folder size={20} />
          </button>
        </div>

        {/* Left Panel - Assets */}
        <div className={styles.leftPanel}>
          <AssetPanel assets={[]} onDragStart={() => {}} />
        </div>

        {/* Center - Game Canvas */}
        <div className={styles.centerArea}>
          <GameCanvas onDrop={() => {}} />
        </div>

        {/* Right Panel - Inspector & Hierarchy */}
        <div className={styles.rightPanel}>
          <SceneHierarchy objects={[]} />
          <ObjectInspector selectedObject={selectedObject} />
        </div>
      </div>
    </DndContext>
  );
};

export default TopDownGameCreator;