// src/hooks/useGameEngine.ts
import { useEffect, useRef, useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import * as PIXI from 'pixi.js';

interface EngineFrame {
  commands: DrawCommand[];
}

interface DrawCommand {
  type: 'Clear' | 'DrawSprite' | 'DrawRect' | 'DrawLine' | 'DrawCircle';
  data: any;
}

export function useGameEngine(projectId: string) {
  const [engineId, setEngineId] = useState<string | null>(null);
  const [isRunning, setIsRunning] = useState(false);
  const pixiApp = useRef<PIXI.Application | null>(null);
  const sprites = useRef<Map<string, PIXI.Sprite>>(new Map());
  const graphics = useRef<PIXI.Graphics | null>(null);
  
  // Initialize engine
  useEffect(() => {
    let mounted = true;
    
    async function initEngine() {
      try {
        const id = await invoke<string>('create_preview_engine', { projectId });
        if (mounted) {
          setEngineId(id);
        }
      } catch (error) {
        console.error('Failed to create engine:', error);
      }
    }
    
    initEngine();
    
    return () => {
      mounted = false;
      if (engineId) {
        invoke('destroy_preview_engine', { engineId }).catch(console.error);
      }
    };
  }, [projectId]);
  
  // Create PIXI renderer
  const initRenderer = useCallback((canvas: HTMLCanvasElement) => {
    if (pixiApp.current) return;
    
    const app = new PIXI.Application({
      view: canvas,
      width: 800,
      height: 600,
      backgroundColor: 0x1a1a2e,
      antialias: true,
    });
    
    pixiApp.current = app;
    
    // Create graphics object for primitives
    const g = new PIXI.Graphics();
    app.stage.addChild(g);
    graphics.current = g;
  }, []);
  
  // Start/stop game loop
  const start = useCallback(() => {
    if (!engineId || isRunning) return;
    
    setIsRunning(true);
    
    let lastTime = performance.now();
    let animationId: number;
    
    const gameLoop = async (currentTime: number) => {
      const dt = (currentTime - lastTime) / 1000;
      lastTime = currentTime;
      
      try {
        // Get frame data from engine
        const frameData = await invoke<string>('render_preview_frame', {
          engineId,
          dt
        });
        
        const frame: EngineFrame = JSON.parse(frameData);
        renderFrame(frame);
      } catch (error) {
        console.error('Frame error:', error);
      }
      
      if (isRunning) {
        animationId = requestAnimationFrame(gameLoop);
      }
    };
    
    animationId = requestAnimationFrame(gameLoop);
    
    return () => {
      setIsRunning(false);
      if (animationId) {
        cancelAnimationFrame(animationId);
      }
    };
  }, [engineId, isRunning]);
  
  const stop = useCallback(() => {
    setIsRunning(false);
  }, []);
  
  // Render frame data from engine
  const renderFrame = useCallback((frame: EngineFrame) => {
    if (!pixiApp.current || !graphics.current) return;
    
    // Clear graphics
    graphics.current.clear();
    
    for (const command of frame.commands) {
      switch (command.type) {
        case 'Clear':
          // Background is already set, but we could update it here
          break;
          
        case 'DrawSprite':
          renderSprite(command.data);
          break;
          
        case 'DrawRect':
          const { position, size, color } = command.data;
          graphics.current.beginFill(rgbToHex(color));
          graphics.current.drawRect(position.x, position.y, size.x, size.y);
          graphics.current.endFill();
          break;
          
        case 'DrawLine':
          const { start, end, color: lineColor, width } = command.data;
          graphics.current.lineStyle(width, rgbToHex(lineColor));
          graphics.current.moveTo(start.x, start.y);
          graphics.current.lineTo(end.x, end.y);
          break;
          
        case 'DrawCircle':
          const { center, radius, color: circleColor } = command.data;
          graphics.current.beginFill(rgbToHex(circleColor));
          graphics.current.drawCircle(center.x, center.y, radius);
          graphics.current.endFill();
          break;
      }
    }
  }, []);
  
  const renderSprite = useCallback((data: any) => {
    if (!pixiApp.current) return;
    
    const { position, rotation, scale, texture_id, color, flip_x, flip_y } = data;
    
    // Get or create sprite
    let sprite = sprites.current.get(texture_id);
    if (!sprite) {
      // For now, create a placeholder rectangle
      const graphics = new PIXI.Graphics();
      graphics.beginFill(0xffffff);
      graphics.drawRect(-32, -32, 64, 64);
      graphics.endFill();
      
      const texture = pixiApp.current.renderer.generateTexture(graphics);
      sprite = new PIXI.Sprite(texture);
      sprite.anchor.set(0.5);
      
      pixiApp.current.stage.addChild(sprite);
      sprites.current.set(texture_id, sprite);
    }
    
    // Update sprite transform
    sprite.position.set(position.x, position.y);
    sprite.rotation = rotation;
    sprite.scale.set(scale.x * (flip_x ? -1 : 1), scale.y * (flip_y ? -1 : 1));
    sprite.tint = rgbToHex(color);
    sprite.alpha = color[3];
  }, []);
  
  // Update scene data
  const updateScene = useCallback(async (sceneData: any) => {
    if (!engineId) return;
    
    try {
      await invoke('update_preview_scene', {
        engineId,
        sceneData: new TextEncoder().encode(JSON.stringify(sceneData))
      });
    } catch (error) {
      console.error('Failed to update scene:', error);
    }
  }, [engineId]);
  
  // Compile visual script
  const compileScript = useCallback(async (script: any) => {
    try {
      const result = await invoke<string>('compile_visual_script', {
        scriptJson: JSON.stringify(script)
      });
      return result;
    } catch (error) {
      console.error('Failed to compile script:', error);
      throw error;
    }
  }, []);
  
  return {
    engineId,
    isRunning,
    initRenderer,
    start,
    stop,
    updateScene,
    compileScript,
  };
}

// Helper functions
function rgbToHex(color: number[]): number {
  const r = Math.floor(color[0] * 255);
  const g = Math.floor(color[1] * 255);
  const b = Math.floor(color[2] * 255);
  return (r << 16) | (g << 8) | b;
}