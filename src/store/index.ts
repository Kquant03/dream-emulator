// src/store/index.ts
import { create } from 'zustand';
import { devtools, persist } from 'zustand/middleware';
import { immer } from 'zustand/middleware/immer';

// Types
export interface Project {
  id: string;
  name: string;
  engineType: 'topdown' | 'sidescroller' | '3d';
  createdAt: Date;
  updatedAt: Date;
  thumbnailPath?: string;
  lastOpenedAt?: Date;
}

export interface Asset {
  id: string;
  projectId: string;
  type: 'sprite' | 'tilemap' | 'audio' | 'script';
  name: string;
  path: string;
  metadata: Record<string, any>;
  tags: string[];
}

export interface GameObject {
  id: string;
  name: string;
  position: { x: number; y: number };
  rotation: number;
  scale: { x: number; y: number };
  components: Map<string, Component>;
  children: string[]; // IDs of child objects
  parentId?: string;
}

export interface Component {
  type: string;
  data: Record<string, any>;
}

export interface EditorState {
  selectedObjects: string[];
  camera: {
    x: number;
    y: number;
    zoom: number;
  };
  gridEnabled: boolean;
  gridSize: number;
  snapToGrid: boolean;
  tool: 'select' | 'move' | 'rotate' | 'scale' | 'paint' | 'erase';
}

export interface GameScene {
  id: string;
  name: string;
  objects: Map<string, GameObject>;
  width: number;
  height: number;
  backgroundColor: string;
}

export interface VisualScript {
  id: string;
  name: string;
  nodes: VisualScriptNode[];
  connections: VisualScriptConnection[];
}

export interface VisualScriptNode {
  id: string;
  type: string;
  position: { x: number; y: number };
  data: Record<string, any>;
  inputs: Record<string, any>;
  outputs: Record<string, any>;
}

export interface VisualScriptConnection {
  id: string;
  source: string;
  sourceHandle: string;
  target: string;
  targetHandle: string;
}

// Store Interface
interface DreamEmulatorStore {
  // Projects
  projects: Project[];
  currentProject: Project | null;
  createProject: (name: string, engineType: Project['engineType']) => Promise<Project>;
  loadProject: (projectId: string) => Promise<void>;
  updateProject: (projectId: string, updates: Partial<Project>) => void;
  deleteProject: (projectId: string) => Promise<void>;

  // Assets
  assets: Map<string, Asset>;
  uploadAsset: (file: File, type: Asset['type']) => Promise<Asset>;
  deleteAsset: (assetId: string) => Promise<void>;
  updateAssetTags: (assetId: string, tags: string[]) => void;
  searchAssets: (query: string, type?: Asset['type']) => Asset[];

  // Scenes
  scenes: Map<string, GameScene>;
  currentScene: GameScene | null;
  createScene: (name: string) => GameScene;
  loadScene: (sceneId: string) => void;
  updateScene: (sceneId: string, updates: Partial<GameScene>) => void;
  
  // Game Objects
  createGameObject: (name: string, parentId?: string) => GameObject;
  updateGameObject: (objectId: string, updates: Partial<GameObject>) => void;
  deleteGameObject: (objectId: string) => void;
  addComponent: (objectId: string, component: Component) => void;
  removeComponent: (objectId: string, componentType: string) => void;

  // Visual Scripting
  scripts: Map<string, VisualScript>;
  createScript: (name: string) => VisualScript;
  updateScript: (scriptId: string, updates: Partial<VisualScript>) => void;
  
  // Editor State
  editor: EditorState;
  updateEditor: (updates: Partial<EditorState>) => void;
  selectObjects: (objectIds: string[]) => void;
  
  // Undo/Redo
  history: any[];
  historyIndex: number;
  undo: () => void;
  redo: () => void;
  
  // Server Management
  servers: Server[];
  createServer: (name: string, gameId: string) => Promise<Server>;
  startServer: (serverId: string) => Promise<void>;
  stopServer: (serverId: string) => Promise<void>;
}

export interface Server {
  id: string;
  name: string;
  gameId: string;
  status: 'running' | 'stopped';
  port: number;
  maxPlayers: number;
  currentPlayers: number;
}

// Helper function to generate IDs
const generateId = () => Math.random().toString(36).substr(2, 9);

// Create the store
export const useDreamEmulator = create<DreamEmulatorStore>()(
  devtools(
    persist(
      immer((set, get) => ({
        // Initial state
        projects: [],
        currentProject: null,
        assets: new Map(),
        scenes: new Map(),
        currentScene: null,
        scripts: new Map(),
        editor: {
          selectedObjects: [],
          camera: { x: 0, y: 0, zoom: 1 },
          gridEnabled: true,
          gridSize: 32,
          snapToGrid: true,
          tool: 'select',
        },
        history: [],
        historyIndex: -1,
        servers: [],

        // Project methods
        createProject: async (name, engineType) => {
          const project: Project = {
            id: generateId(),
            name,
            engineType,
            createdAt: new Date(),
            updatedAt: new Date(),
          };

          set((state) => {
            state.projects.push(project);
            state.currentProject = project;
          });

          // Create default scene
          get().createScene('Main Scene');

          return project;
        },

        loadProject: async (projectId) => {
          const project = get().projects.find((p) => p.id === projectId);
          if (!project) throw new Error('Project not found');

          set((state) => {
            state.currentProject = project;
            project.lastOpenedAt = new Date();
          });

          // Load project assets and scenes from storage
          // This would interface with Tauri's file system
        },

        updateProject: (projectId, updates) => {
          set((state) => {
            const project = state.projects.find((p) => p.id === projectId);
            if (project) {
              Object.assign(project, updates);
              project.updatedAt = new Date();
            }
          });
        },

        deleteProject: async (projectId) => {
          set((state) => {
            state.projects = state.projects.filter((p) => p.id !== projectId);
            if (state.currentProject?.id === projectId) {
              state.currentProject = null;
            }
          });
        },

        // Asset methods
        uploadAsset: async (file, type) => {
          const asset: Asset = {
            id: generateId(),
            projectId: get().currentProject!.id,
            type,
            name: file.name,
            path: '', // Will be set after file is saved
            metadata: {},
            tags: [],
          };

          // Here you would save the file using Tauri's file system API
          // For now, we'll use a placeholder path
          asset.path = `/assets/${asset.id}/${file.name}`;

          set((state) => {
            state.assets.set(asset.id, asset);
          });

          return asset;
        },

        deleteAsset: async (assetId) => {
          set((state) => {
            state.assets.delete(assetId);
          });
        },

        updateAssetTags: (assetId, tags) => {
          set((state) => {
            const asset = state.assets.get(assetId);
            if (asset) {
              asset.tags = tags;
            }
          });
        },

        searchAssets: (query, type) => {
          const assets = Array.from(get().assets.values());
          return assets.filter((asset) => {
            if (type && asset.type !== type) return false;
            
            const searchStr = query.toLowerCase();
            return (
              asset.name.toLowerCase().includes(searchStr) ||
              asset.tags.some((tag) => tag.toLowerCase().includes(searchStr))
            );
          });
        },

        // Scene methods
        createScene: (name) => {
          const scene: GameScene = {
            id: generateId(),
            name,
            objects: new Map(),
            width: 1920,
            height: 1080,
            backgroundColor: '#1a1a2e',
          };

          set((state) => {
            state.scenes.set(scene.id, scene);
            state.currentScene = scene;
          });

          return scene;
        },

        loadScene: (sceneId) => {
          const scene = get().scenes.get(sceneId);
          if (!scene) throw new Error('Scene not found');

          set((state) => {
            state.currentScene = scene;
          });
        },

        updateScene: (sceneId, updates) => {
          set((state) => {
            const scene = state.scenes.get(sceneId);
            if (scene) {
              Object.assign(scene, updates);
            }
          });
        },

        // GameObject methods
        createGameObject: (name, parentId) => {
          const obj: GameObject = {
            id: generateId(),
            name,
            position: { x: 0, y: 0 },
            rotation: 0,
            scale: { x: 1, y: 1 },
            components: new Map(),
            children: [],
            parentId,
          };

          set((state) => {
            if (state.currentScene) {
              state.currentScene.objects.set(obj.id, obj);
              
              if (parentId) {
                const parent = state.currentScene.objects.get(parentId);
                if (parent) {
                  parent.children.push(obj.id);
                }
              }
            }
          });

          return obj;
        },

        updateGameObject: (objectId, updates) => {
          set((state) => {
            if (state.currentScene) {
              const obj = state.currentScene.objects.get(objectId);
              if (obj) {
                Object.assign(obj, updates);
              }
            }
          });
        },

        deleteGameObject: (objectId) => {
          set((state) => {
            if (state.currentScene) {
              const obj = state.currentScene.objects.get(objectId);
              if (obj) {
                // Remove from parent
                if (obj.parentId) {
                  const parent = state.currentScene.objects.get(obj.parentId);
                  if (parent) {
                    parent.children = parent.children.filter((id) => id !== objectId);
                  }
                }
                
                // Delete all children
                const deleteChildren = (id: string) => {
                  const child = state.currentScene!.objects.get(id);
                  if (child) {
                    child.children.forEach(deleteChildren);
                    state.currentScene!.objects.delete(id);
                  }
                };
                obj.children.forEach(deleteChildren);
                
                // Delete the object
                state.currentScene.objects.delete(objectId);
              }
            }
          });
        },

        addComponent: (objectId, component) => {
          set((state) => {
            if (state.currentScene) {
              const obj = state.currentScene.objects.get(objectId);
              if (obj) {
                obj.components.set(component.type, component);
              }
            }
          });
        },

        removeComponent: (objectId, componentType) => {
          set((state) => {
            if (state.currentScene) {
              const obj = state.currentScene.objects.get(objectId);
              if (obj) {
                obj.components.delete(componentType);
              }
            }
          });
        },

        // Visual Scripting methods
        createScript: (name) => {
          const script: VisualScript = {
            id: generateId(),
            name,
            nodes: [],
            connections: [],
          };

          set((state) => {
            state.scripts.set(script.id, script);
          });

          return script;
        },

        updateScript: (scriptId, updates) => {
          set((state) => {
            const script = state.scripts.get(scriptId);
            if (script) {
              Object.assign(script, updates);
            }
          });
        },

        // Editor methods
        updateEditor: (updates) => {
          set((state) => {
            Object.assign(state.editor, updates);
          });
        },

        selectObjects: (objectIds) => {
          set((state) => {
            state.editor.selectedObjects = objectIds;
          });
        },

        // Undo/Redo
        undo: () => {
          const { history, historyIndex } = get();
          if (historyIndex > 0) {
            // Apply undo action
            set((state) => {
              state.historyIndex--;
            });
          }
        },

        redo: () => {
          const { history, historyIndex } = get();
          if (historyIndex < history.length - 1) {
            // Apply redo action
            set((state) => {
              state.historyIndex++;
            });
          }
        },

        // Server Management
        createServer: async (name, gameId) => {
          const server: Server = {
            id: generateId(),
            name,
            gameId,
            status: 'stopped',
            port: 3000 + get().servers.length,
            maxPlayers: 8,
            currentPlayers: 0,
          };

          set((state) => {
            state.servers.push(server);
          });

          return server;
        },

        startServer: async (serverId) => {
          set((state) => {
            const server = state.servers.find((s) => s.id === serverId);
            if (server) {
              server.status = 'running';
            }
          });
        },

        stopServer: async (serverId) => {
          set((state) => {
            const server = state.servers.find((s) => s.id === serverId);
            if (server) {
              server.status = 'stopped';
              server.currentPlayers = 0;
            }
          });
        },
      })),
      {
        name: 'dream-emulator-storage',
        // Only persist certain parts of the state
        partialize: (state) => ({
          projects: state.projects,
          editor: state.editor,
        }),
      }
    )
  )
);