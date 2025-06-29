// Example usage in a component
// src/components/GamePreview.tsx
import React, { useRef, useEffect } from 'react';
import { useGameEngine } from '@/hooks/useGameEngine';
import { useDreamEmulator } from '@/store';

interface GamePreviewProps {
  width: number;
  height: number;
}

export function GamePreview({ width, height }: GamePreviewProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const { currentProject } = useDreamEmulator();
  const engine = useGameEngine(currentProject?.id || '');
  
  useEffect(() => {
    if (canvasRef.current && engine.engineId) {
      engine.initRenderer(canvasRef.current);
    }
  }, [engine.engineId]);
  
  return (
    <div style={{ position: 'relative' }}>
      <canvas ref={canvasRef} width={width} height={height} />
      
      <div style={{ position: 'absolute', top: 10, right: 10 }}>
        <button onClick={engine.start} disabled={engine.isRunning}>
          Play
        </button>
        <button onClick={engine.stop} disabled={!engine.isRunning}>
          Stop
        </button>
      </div>
    </div>
  );
}

// Example: Building a game
// src/components/BuildDialog.tsx
import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { save } from '@tauri-apps/api/dialog';

interface BuildDialogProps {
  projectId: string;
  onClose: () => void;
}

export function BuildDialog({ projectId, onClose }: BuildDialogProps) {
  const [building, setBuilding] = useState(false);
  const [progress, setProgress] = useState('');
  const [target, setTarget] = useState('native');
  
  const handleBuild = async () => {
    setBuilding(true);
    setProgress('Starting build...');
    
    try {
      // Build the game
      setProgress('Compiling visual scripts...');
      const buildResult = await invoke<string>('build_game', {
        projectId,
        target
      });
      
      const result = JSON.parse(buildResult);
      setProgress(`Build complete! Size: ${(result.size / 1024 / 1024).toFixed(2)}MB`);
      
      // Ask where to save
      const savePath = await save({
        defaultPath: 'MyGame',
        filters: [{
          name: 'Executable',
          extensions: target === 'windows' ? ['exe'] : ['']
        }]
      });
      
      if (savePath) {
        setProgress('Exporting game...');
        await invoke('export_game', {
          projectId,
          outputPath: savePath
        });
        setProgress('Export complete!');
      }
    } catch (error) {
      setProgress(`Build failed: ${error}`);
    } finally {
      setBuilding(false);
    }
  };
  
  return (
    <div className="build-dialog">
      <h2>Build Game</h2>
      
      <div>
        <label>Target Platform:</label>
        <select value={target} onChange={(e) => setTarget(e.target.value)}>
          <option value="native">Native (Current OS)</option>
          <option value="windows">Windows</option>
          <option value="linux">Linux</option>
          <option value="macos">macOS</option>
          <option value="web">Web (WASM)</option>
        </select>
      </div>
      
      <div>{progress}</div>
      
      <button onClick={handleBuild} disabled={building}>
        {building ? 'Building...' : 'Build'}
      </button>
      <button onClick={onClose} disabled={building}>
        Close
      </button>
    </div>
  );
}

// Example: Player Movement Visual Script
// This shows how a visual script in the editor compiles to Rust code

// Visual Script Definition (what's saved in the project)
const playerMovementScript = {
  id: "script_001",
  name: "Player Movement",
  nodes: [
    {
      id: "1",
      type: "event/update",
      position: { x: 100, y: 100 },
      data: { label: "Every Frame" }
    },
    {
      id: "2", 
      type: "query/get_entities",
      position: { x: 300, y: 100 },
      data: { 
        label: "Get Players",
        components: ["Transform", "PlayerController", "RigidBody"]
      }
    },
    {
      id: "3",
      type: "input/keyboard",
      position: { x: 300, y: 250 },
      data: {
        label: "Get Input",
        axes: ["horizontal", "vertical"]
      }
    },
    {
      id: "4",
      type: "math/multiply",
      position: { x: 500, y: 250 },
      data: { label: "Scale by Speed" }
    },
    {
      id: "5",
      type: "physics/apply_force",
      position: { x: 700, y: 200 },
      data: { label: "Move Player" }
    }
  ],
  connections: [
    {
      id: "c1",
      source: "1",
      sourceHandle: "exec",
      target: "2", 
      targetHandle: "exec"
    },
    {
      id: "c2",
      source: "2",
      sourceHandle: "entities",
      target: "5",
      targetHandle: "entity"
    },
    {
      id: "c3",
      source: "2",
      sourceHandle: "PlayerController",
      target: "4",
      targetHandle: "a"
    },
    {
      id: "c4",
      source: "3",
      sourceHandle: "vector",
      target: "4",
      targetHandle: "b"
    },
    {
      id: "c5",
      source: "4",
      sourceHandle: "result",
      target: "5",
      targetHandle: "force"
    }
  ]
};

// This compiles to the following Rust code:
const compiledRustCode = `
use dream_engine::{World, PhysicsWorld, System, EntityId};
use dream_engine::{Transform, PlayerController, RigidBody, Vec2, Vec3};

pub struct PlayerMovementSystem {
    // System state
}

impl System for PlayerMovementSystem {
    fn execute(&mut self, world: &mut World, physics: &mut PhysicsWorld, dt: f32) {
        // Every Frame
        
        // Get Players
        for (entity, (transform, player_controller, rigid_body)) in world.query::<(&Transform, &PlayerController, &RigidBody)>().iter() {
            
            // Get Input
            let horizontal = if input::is_key_pressed(Key::D) { 1.0 } 
                           else if input::is_key_pressed(Key::A) { -1.0 } 
                           else { 0.0 };
            let vertical = if input::is_key_pressed(Key::W) { 1.0 } 
                         else if input::is_key_pressed(Key::S) { -1.0 } 
                         else { 0.0 };
            let input_vector = Vec2::new(horizontal, vertical).normalize();
            
            // Scale by Speed
            let movement_force = input_vector * player_controller.move_speed;
            
            // Move Player
            if let Some(body) = physics.get_body_mut(entity) {
                body.apply_force(movement_force);
            }
        }
    }
}
`;

// Another Example: Enemy AI Script
const enemyAIScript = {
  id: "script_002",
  name: "Enemy Chase AI",
  nodes: [
    {
      id: "1",
      type: "event/update",
      position: { x: 100, y: 100 },
      data: { label: "Every Frame" }
    },
    {
      id: "2",
      type: "query/get_entities",
      position: { x: 300, y: 100 },
      data: {
        label: "Get Enemies",
        components: ["Transform", "Enemy", "RigidBody"]
      }
    },
    {
      id: "3",
      type: "query/find_nearest",
      position: { x: 500, y: 150 },
      data: {
        label: "Find Nearest Player",
        targetTag: "Player"
      }
    },
    {
      id: "4",
      type: "math/distance",
      position: { x: 700, y: 150 },
      data: { label: "Get Distance" }
    },
    {
      id: "5",
      type: "flow/if",
      position: { x: 900, y: 150 },
      data: {
        label: "If In Range",
        comparison: "<",
        value: 200.0
      }
    },
    {
      id: "6",
      type: "math/direction",
      position: { x: 700, y: 300 },
      data: { label: "Get Direction" }
    },
    {
      id: "7",
      type: "physics/move_towards",
      position: { x: 900, y: 300 },
      data: { label: "Chase Player" }
    }
  ],
  connections: [
    // ... connections between nodes
  ]
};

// Example: How to create these scripts in the editor
export function VisualScriptEditor() {
  const [nodes, setNodes] = useState([]);
  const [connections, setConnections] = useState([]);
  
  // Node palette - what users can drag into the editor
  const nodeTypes = [
    // Events
    { category: "Events", type: "event/update", label: "On Update" },
    { category: "Events", type: "event/start", label: "On Start" },
    { category: "Events", type: "event/collision", label: "On Collision" },
    
    // Queries
    { category: "Queries", type: "query/get_entities", label: "Get Entities" },
    { category: "Queries", type: "query/find_nearest", label: "Find Nearest" },
    { category: "Queries", type: "query/raycast", label: "Raycast" },
    
    // Components
    { category: "Components", type: "component/get", label: "Get Component" },
    { category: "Components", type: "component/set", label: "Set Component" },
    
    // Math
    { category: "Math", type: "math/add", label: "Add" },
    { category: "Math", type: "math/multiply", label: "Multiply" },
    { category: "Math", type: "math/distance", label: "Distance" },
    { category: "Math", type: "math/lerp", label: "Lerp" },
    
    // Physics
    { category: "Physics", type: "physics/apply_force", label: "Apply Force" },
    { category: "Physics", type: "physics/set_velocity", label: "Set Velocity" },
    { category: "Physics", type: "physics/raycast", label: "Physics Raycast" },
    
    // Flow Control
    { category: "Flow", type: "flow/if", label: "If" },
    { category: "Flow", type: "flow/foreach", label: "For Each" },
    { category: "Flow", type: "flow/while", label: "While" },
    
    // Actions
    { category: "Actions", type: "action/spawn", label: "Spawn Object" },
    { category: "Actions", type: "action/destroy", label: "Destroy" },
    { category: "Actions", type: "action/play_sound", label: "Play Sound" },
  ];
  
  return (
    <div className="visual-script-editor">
      {/* Node palette */}
      <div className="node-palette">
        {nodeTypes.map(node => (
          <div 
            key={node.type}
            draggable
            onDragStart={(e) => {
              e.dataTransfer.setData('nodeType', node.type);
            }}
          >
            {node.label}
          </div>
        ))}
      </div>
      
      {/* Canvas with React Flow or similar */}
      <div className="script-canvas">
        {/* Visual node graph editor here */}
      </div>
    </div>
  );
}

// Example: Complete game project structure
const exampleProject = {
  id: "proj_001",
  name: "Space Shooter",
  engineType: "topdown",
  scenes: [
    {
      id: "scene_001",
      name: "Main Level",
      objects: [
        {
          id: "obj_001",
          name: "Player",
          position: { x: 400, y: 500 },
          rotation: 0,
          scale: { x: 1, y: 1 },
          components: [
            {
              component_type: "Sprite",
              data: { texture_id: "player_ship" }
            },
            {
              component_type: "PlayerController",
              data: { move_speed: 300.0 }
            },
            {
              component_type: "RigidBody",
              data: { body_type: "Dynamic", mass: 1.0 }
            },
            {
              component_type: "Collider",
              data: { type: "circle", radius: 16 }
            }
          ]
        }
      ]
    }
  ],
  scripts: [
    playerMovementScript,
    enemyAIScript
  ],
  assets: [
    {
      id: "player_ship",
      name: "player_ship.png",
      path: "/assets/sprites/player_ship.png",
      asset_type: "sprite"
    }
  ]
};

// When built, this creates a ~3MB executable instead of a 150MB Electron app!