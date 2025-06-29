# Dream Emulator Architecture Deep Dive

## Table of Contents
- [Overview](#overview)
- [Core Design Principles](#core-design-principles)
- [System Architecture](#system-architecture)
- [Data Flow](#data-flow)
- [Component Details](#component-details)
- [Performance Considerations](#performance-considerations)
- [Security Model](#security-model)
- [Extension Points](#extension-points)

## Overview

Dream Emulator is built on a modular, event-driven architecture that separates concerns across multiple layers:

```
┌─────────────────────────────────────────────────────────────┐
│                      User Interface Layer                     │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │  Main Menu  │  │ Game Editors │  │ Visual Scripting │   │
│  └─────────────┘  └──────────────┘  └──────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                    State Management Layer                     │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │   Projects  │  │    Scenes    │  │     Assets       │   │
│  └─────────────┘  └──────────────┘  └──────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                     Game Engine Layer                         │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │  Renderer   │  │     ECS      │  │    Physics       │   │
│  └─────────────┘  └──────────────┘  └──────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                    Platform Layer (Tauri)                     │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │ File System │  │   Native UI  │  │   Networking     │   │
│  └─────────────┘  └──────────────┘  └──────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## Core Design Principles

### 1. **Separation of Concerns**
Each layer has a specific responsibility and communicates through well-defined interfaces.

### 2. **Event-Driven Communication**
Components communicate through events, enabling loose coupling and extensibility.

### 3. **Immutable State Updates**
Using Zustand with Immer ensures predictable state mutations and enables time-travel debugging.

### 4. **Progressive Disclosure**
The UI reveals complexity gradually, keeping beginners comfortable while providing power users with advanced features.

### 5. **Performance First**
Critical paths are optimized for 60 FPS operation with large game worlds.

## System Architecture

### Frontend Architecture (React)

```typescript
// Component hierarchy
App
├── RouteManager
│   ├── MainMenu
│   ├── EngineSelector
│   └── GameEditor
│       ├── Toolbar
│       ├── AssetPanel
│       ├── GameCanvas
│       ├── Inspector
│       └── Hierarchy
└── GlobalProviders
    ├── ThemeProvider
    ├── ErrorBoundary
    └── ShortcutProvider
```

### State Management (Zustand)

The state is organized into slices for better maintainability:

```typescript
interface StoreSlices {
  projectSlice: ProjectSlice
  sceneSlice: SceneSlice
  assetSlice: AssetSlice
  editorSlice: EditorSlice
  scriptSlice: ScriptSlice
  serverSlice: ServerSlice
}

// Each slice manages its domain
interface ProjectSlice {
  projects: Map<string, Project>
  currentProjectId: string | null
  actions: {
    createProject: (options: ProjectOptions) => Promise<Project>
    loadProject: (id: string) => Promise<void>
    saveProject: () => Promise<void>
    exportProject: (format: ExportFormat) => Promise<void>
  }
}
```

### Game Engine Architecture

The game engine uses an Entity-Component-System (ECS) pattern:

```typescript
// Entity: Just an ID
type EntityId = string

// Component: Pure data
interface Transform {
  type: 'transform'
  position: Vector2
  rotation: number
  scale: Vector2
}

interface Sprite {
  type: 'sprite'
  textureId: string
  tint: number
  alpha: number
}

// System: Logic that operates on components
class RenderSystem {
  query = ['transform', 'sprite'] as const
  
  update(entities: EntityView[]) {
    for (const entity of entities) {
      const transform = entity.get('transform')
      const sprite = entity.get('sprite')
      this.renderSprite(transform, sprite)
    }
  }
}

// World: Manages everything
class World {
  entities: Map<EntityId, Entity>
  systems: System[]
  
  update(deltaTime: number) {
    for (const system of this.systems) {
      const entities = this.query(system.query)
      system.update(entities, deltaTime)
    }
  }
}
```

### Rendering Pipeline

```typescript
class RenderPipeline {
  stages = [
    new ClearStage(),
    new BackgroundStage(),
    new TilemapStage(),
    new SpriteStage(),
    new ParticleStage(),
    new UIStage(),
    new DebugStage()
  ]
  
  render(world: World, camera: Camera) {
    for (const stage of this.stages) {
      stage.render(world, camera)
    }
  }
}
```

## Data Flow

### 1. User Input Flow
```
User Input → UI Component → Action Dispatch → State Update → Re-render
     ↓
Input System → Game Engine → Physics/Logic → State Update
```

### 2. Asset Loading Flow
```
File Selection → Tauri File API → Validation → Processing → Storage
       ↓
Asset Manager → Texture Cache → GPU Upload → Ready for Use
```

### 3. Visual Script Execution
```
Node Graph → Compilation → Bytecode → Interpreter → Game State
      ↓
Hot Reload ← File Watch ← Script Editor
```

## Component Details

### Main Menu Component
- **Purpose**: Entry point with engine selection
- **State**: Local animation state only
- **Features**: Particle effects, smooth transitions
- **Performance**: Uses CSS transforms for 60 FPS animations

### Game Editor Components

#### Toolbar
- **Purpose**: Tool selection and quick actions
- **State**: Connected to editor.tool slice
- **Shortcuts**: Keyboard mappings for all tools
- **Extensible**: Plugins can add tools

#### Asset Panel
- **Purpose**: Organize and browse game assets
- **Features**: 
  - Drag-and-drop to canvas
  - Smart search and filtering
  - Thumbnail generation
  - Batch operations
- **Performance**: Virtual scrolling for large libraries

#### Game Canvas
- **Renderer**: PIXI.js with WebGL
- **Features**:
  - Multi-selection with box select
  - Snap-to-grid with visual guides
  - Real-time preview
  - Zoom/pan with limits
- **Optimizations**:
  - Culling off-screen objects
  - Batched rendering
  - Texture atlasing

#### Inspector
- **Purpose**: Edit selected object properties
- **Features**:
  - Dynamic property fields
  - Component system
  - Undo/redo integration
- **Extensible**: Plugins can add property types

### State Management Details

#### Persistence Strategy
```typescript
// Only persist necessary data
const persistConfig = {
  name: 'dream-emulator',
  version: 1,
  partialize: (state) => ({
    projects: Array.from(state.projects.entries()),
    preferences: state.preferences,
    recentFiles: state.recentFiles
  })
}
```

#### Undo/Redo System
```typescript
class UndoManager {
  private history: Command[] = []
  private cursor = -1
  
  execute(command: Command) {
    // Remove future history
    this.history = this.history.slice(0, this.cursor + 1)
    
    // Add and execute
    this.history.push(command)
    command.execute()
    this.cursor++
    
    // Limit history size
    if (this.history.length > 100) {
      this.history.shift()
      this.cursor--
    }
  }
  
  undo() {
    if (this.cursor >= 0) {
      this.history[this.cursor].undo()
      this.cursor--
    }
  }
}
```

## Performance Considerations

### Rendering Performance
1. **Batch Rendering**: Group similar draw calls
2. **Texture Atlasing**: Combine small textures
3. **Object Pooling**: Reuse objects to reduce GC
4. **Culling**: Only render visible objects
5. **LOD System**: Reduce detail for distant objects

### State Performance
1. **Immutable Updates**: Use Immer for efficient updates
2. **Selective Re-renders**: React.memo for expensive components
3. **Virtualization**: Large lists use virtual scrolling
4. **Web Workers**: Heavy computations off main thread

### Memory Management
```typescript
class AssetCache {
  private cache = new Map<string, WeakRef<Asset>>()
  private memory = 0
  private limit = 500 * 1024 * 1024 // 500MB
  
  add(id: string, asset: Asset) {
    if (this.memory + asset.size > this.limit) {
      this.evictLRU()
    }
    this.cache.set(id, new WeakRef(asset))
    this.memory += asset.size
  }
}
```

## Security Model

### Script Sandboxing
```typescript
class ScriptSandbox {
  private worker: Worker
  private timeout = 16 // ms per frame
  
  async execute(script: string, context: GameContext) {
    return new Promise((resolve, reject) => {
      const timeoutId = setTimeout(() => {
        this.worker.terminate()
        reject(new Error('Script timeout'))
      }, this.timeout)
      
      this.worker.postMessage({ script, context })
      this.worker.onmessage = (e) => {
        clearTimeout(timeoutId)
        resolve(e.data)
      }
    })
  }
}
```

### Asset Validation
- File type validation
- Size limits
- Content scanning
- Sandboxed preview generation

## Extension Points

### Plugin System
```typescript
interface Plugin {
  id: string
  name: string
  version: string
  
  // Lifecycle hooks
  activate(context: PluginContext): void
  deactivate(): void
  
  // Extension points
  tools?: ToolDefinition[]
  nodes?: NodeDefinition[]
  components?: ComponentDefinition[]
  exporters?: ExporterDefinition[]
}

class PluginManager {
  async loadPlugin(path: string) {
    const module = await import(path)
    const plugin = new module.default() as Plugin
    
    // Validate plugin
    this.validatePlugin(plugin)
    
    // Register extensions
    plugin.activate(this.createContext(plugin))
    
    this.plugins.set(plugin.id, plugin)
  }
}
```

### Custom Nodes
```typescript
interface NodeDefinition {
  type: string
  category: string
  inputs: PortDefinition[]
  outputs: PortDefinition[]
  
  // Execution
  execute(inputs: InputValues): OutputValues
  
  // UI
  renderPreview(): ReactElement
  renderProperties(): ReactElement
}
```

### Custom Exporters
```typescript
interface ExporterDefinition {
  format: string
  name: string
  
  canExport(project: Project): boolean
  export(project: Project, options: ExportOptions): Promise<Blob>
}
```

## Future Considerations

### Multi-Window Support
- Separate windows for different editors
- Shared state across windows
- Drag-and-drop between windows

### Collaborative Editing
- CRDT-based state synchronization
- Presence awareness
- Conflict resolution

### Cloud Integration
- Asset CDN
- Project sharing
- Real-time collaboration
- Version control

This architecture is designed to scale from simple 2D games to complex multiplayer experiences while maintaining performance and usability.