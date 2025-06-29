# Plugin Development Guide

## Table of Contents
- [Introduction](#introduction)
- [Plugin Architecture](#plugin-architecture)
- [Creating Your First Plugin](#creating-your-first-plugin)
- [Plugin API](#plugin-api)
- [Extension Points](#extension-points)
- [Publishing Plugins](#publishing-plugins)
- [Best Practices](#best-practices)
- [Examples](#examples)
- [API Reference](#api-reference)

## Introduction

Dream Emulator's plugin system allows developers to extend the editor with new features, tools, nodes, and exporters. Plugins run in a sandboxed environment with access to a comprehensive API.

### Why Create Plugins?
- **Add Custom Tools**: Specialized tools for your workflow
- **Create Node Types**: Visual scripting nodes for specific games
- **Export Formats**: Support for additional platforms
- **Asset Processors**: Custom import/export pipelines
- **Editor Extensions**: New panels, inspectors, and workflows

## Plugin Architecture

### Plugin Structure
```
my-plugin/
├── manifest.json         # Plugin metadata
├── index.ts             # Main entry point
├── assets/              # Plugin assets
│   ├── icons/
│   └── templates/
├── src/                 # Source code
│   ├── tools/
│   ├── nodes/
│   └── components/
├── dist/                # Compiled output
└── README.md           # Documentation
```

### Manifest File
```json
{
  "id": "com.example.myplugin",
  "name": "My Awesome Plugin",
  "version": "1.0.0",
  "description": "Adds awesome features to Dream Emulator",
  "author": {
    "name": "Your Name",
    "email": "you@example.com",
    "url": "https://example.com"
  },
  "main": "dist/index.js",
  "dreamEmulator": {
    "minVersion": "1.0.0",
    "maxVersion": "2.0.0"
  },
  "permissions": [
    "filesystem.read",
    "filesystem.write",
    "network.fetch",
    "editor.tools",
    "editor.nodes",
    "project.assets"
  ],
  "contributes": {
    "tools": ["./tools/*.js"],
    "nodes": ["./nodes/*.js"],
    "commands": ["./commands/*.js"],
    "themes": ["./themes/*.json"]
  },
  "dependencies": {},
  "keywords": ["platformer", "tools", "animation"],
  "repository": "https://github.com/user/plugin",
  "license": "MIT"
}
```

## Creating Your First Plugin

### Step 1: Set Up Development Environment
```bash
# Install Dream Emulator Plugin SDK
npm install -g @dreamemulator/plugin-sdk

# Create new plugin
dream-plugin create my-plugin

# Navigate to plugin directory
cd my-plugin

# Install dependencies
npm install

# Start development
npm run dev
```

### Step 2: Basic Plugin Structure
```typescript
// index.ts
import { Plugin, PluginContext } from '@dreamemulator/plugin-api'

export default class MyPlugin implements Plugin {
  private context: PluginContext
  
  // Called when plugin is loaded
  async activate(context: PluginContext) {
    this.context = context
    
    // Register a command
    context.commands.register({
      id: 'myPlugin.hello',
      name: 'Hello World',
      execute: () => {
        context.ui.showMessage('Hello from my plugin!')
      }
    })
    
    // Add menu item
    context.menus.add('tools', {
      id: 'myPlugin.menu',
      label: 'My Plugin',
      command: 'myPlugin.hello'
    })
    
    console.log('My plugin activated!')
  }
  
  // Called when plugin is unloaded
  async deactivate() {
    console.log('My plugin deactivated!')
  }
}
```

### Step 3: Create a Custom Tool
```typescript
// tools/CustomBrush.ts
import { Tool, ToolContext, Vector2 } from '@dreamemulator/plugin-api'

export class CustomBrushTool implements Tool {
  id = 'myPlugin.customBrush'
  name = 'Custom Brush'
  icon = 'brush'
  cursor = 'crosshair'
  
  private isDrawing = false
  private lastPoint: Vector2 | null = null
  
  activate(context: ToolContext) {
    context.ui.showMessage('Custom Brush activated')
  }
  
  onMouseDown(context: ToolContext, event: MouseEvent) {
    this.isDrawing = true
    this.lastPoint = context.screenToWorld(event.x, event.y)
    
    // Start drawing
    this.draw(context, this.lastPoint)
  }
  
  onMouseMove(context: ToolContext, event: MouseEvent) {
    if (!this.isDrawing) return
    
    const point = context.screenToWorld(event.x, event.y)
    
    // Draw line from last point
    if (this.lastPoint) {
      this.drawLine(context, this.lastPoint, point)
    }
    
    this.lastPoint = point
  }
  
  onMouseUp(context: ToolContext, event: MouseEvent) {
    this.isDrawing = false
    this.lastPoint = null
    
    // Commit changes
    context.history.commit('Custom Brush Stroke')
  }
  
  private draw(context: ToolContext, point: Vector2) {
    // Create a sprite at position
    const sprite = context.scene.createSprite({
      texture: 'brush_mark',
      position: point,
      scale: { x: 0.1, y: 0.1 },
      tint: context.selectedColor
    })
    
    context.scene.add(sprite)
  }
  
  private drawLine(context: ToolContext, from: Vector2, to: Vector2) {
    const distance = Math.sqrt(
      (to.x - from.x) ** 2 + (to.y - from.y) ** 2
    )
    const steps = Math.ceil(distance / 5) // Draw every 5 pixels
    
    for (let i = 0; i <= steps; i++) {
      const t = i / steps
      const point = {
        x: from.x + (to.x - from.x) * t,
        y: from.y + (to.y - from.y) * t
      }
      this.draw(context, point)
    }
  }
}
```

## Plugin API

### Core API Modules

#### Context API
```typescript
interface PluginContext {
  // Plugin info
  plugin: PluginInfo
  
  // Editor APIs
  commands: CommandRegistry
  menus: MenuRegistry
  tools: ToolRegistry
  nodes: NodeRegistry
  
  // UI APIs
  ui: UIManager
  panels: PanelManager
  dialogs: DialogManager
  
  // Project APIs
  project: ProjectManager
  assets: AssetManager
  scenes: SceneManager
  
  // System APIs
  filesystem: FileSystem
  network: NetworkManager
  preferences: PreferencesManager
  
  // Events
  events: EventEmitter
}
```

#### UI Manager
```typescript
interface UIManager {
  // Notifications
  showMessage(message: string, type?: 'info' | 'warning' | 'error'): void
  showNotification(options: NotificationOptions): Notification
  
  // Progress
  showProgress(title: string, cancellable?: boolean): ProgressHandle
  
  // Input
  prompt(title: string, message: string, defaultValue?: string): Promise<string | null>
  confirm(title: string, message: string): Promise<boolean>
  
  // Selection
  selectFile(options?: FileSelectOptions): Promise<File | null>
  selectFolder(options?: FolderSelectOptions): Promise<string | null>
}
```

#### Asset Manager
```typescript
interface AssetManager {
  // Query assets
  getAll(type?: AssetType): Asset[]
  get(id: string): Asset | null
  find(predicate: (asset: Asset) => boolean): Asset[]
  
  // Import assets
  import(file: File, options?: ImportOptions): Promise<Asset>
  importBatch(files: File[], options?: ImportOptions): Promise<Asset[]>
  
  // Modify assets
  update(id: string, updates: Partial<Asset>): Promise<void>
  delete(id: string): Promise<void>
  
  // Events
  on(event: 'added' | 'updated' | 'deleted', handler: (asset: Asset) => void): void
}
```

## Extension Points

### Custom Nodes
```typescript
// nodes/MathNodes.ts
import { NodeDefinition, NodeContext } from '@dreamemulator/plugin-api'

export const MultiplyNode: NodeDefinition = {
  id: 'myPlugin.multiply',
  category: 'Math',
  name: 'Multiply',
  description: 'Multiplies two numbers',
  
  inputs: {
    a: { type: 'number', label: 'A', default: 0 },
    b: { type: 'number', label: 'B', default: 0 }
  },
  
  outputs: {
    result: { type: 'number', label: 'Result' }
  },
  
  execute(inputs: { a: number, b: number }): { result: number } {
    return {
      result: inputs.a * inputs.b
    }
  },
  
  // Optional: Custom UI
  renderNode(context: NodeContext) {
    return (
      <div className="multiply-node">
        <div className="node-header">✖️ Multiply</div>
        <div className="node-preview">
          {context.inputs.a} × {context.inputs.b} = {context.outputs.result}
        </div>
      </div>
    )
  }
}

// Register multiple nodes
export default [
  MultiplyNode,
  DivideNode,
  PowerNode,
  // ... more nodes
]
```

### Custom Exporters
```typescript
// exporters/CustomExporter.ts
import { Exporter, ExportContext } from '@dreamemulator/plugin-api'

export class CustomExporter implements Exporter {
  id = 'myPlugin.customFormat'
  name = 'Custom Game Format'
  extension = '.cgf'
  
  async canExport(context: ExportContext): Promise<boolean> {
    // Check if project is compatible
    return context.project.engineType === 'topdown'
  }
  
  async export(context: ExportContext): Promise<Blob> {
    const { project, scenes, assets } = context
    
    // Show progress
    const progress = context.ui.showProgress('Exporting...', true)
    
    try {
      // Build export data
      const exportData = {
        version: '1.0',
        project: {
          name: project.name,
          id: project.id
        },
        scenes: await this.processScenes(scenes, progress),
        assets: await this.processAssets(assets, progress)
      }
      
      // Convert to binary format
      const binary = this.encode(exportData)
      
      // Create blob
      return new Blob([binary], { type: 'application/octet-stream' })
      
    } finally {
      progress.close()
    }
  }
  
  private async processScenes(scenes: Scene[], progress: ProgressHandle) {
    const processed = []
    
    for (let i = 0; i < scenes.length; i++) {
      progress.update(i / scenes.length, `Processing ${scenes[i].name}`)
      
      processed.push({
        id: scenes[i].id,
        name: scenes[i].name,
        objects: this.serializeObjects(scenes[i].objects)
      })
    }
    
    return processed
  }
}
```

### Custom Panels
```typescript
// panels/AssetStatsPanel.ts
import { Panel, PanelContext } from '@dreamemulator/plugin-api'
import { useState, useEffect } from 'react'

export const AssetStatsPanel: Panel = {
  id: 'myPlugin.assetStats',
  title: 'Asset Statistics',
  icon: 'chart',
  location: 'right', // or 'left', 'bottom'
  
  render(context: PanelContext) {
    const [stats, setStats] = useState<AssetStats | null>(null)
    
    useEffect(() => {
      // Calculate statistics
      const assets = context.assets.getAll()
      const stats = calculateStats(assets)
      setStats(stats)
      
      // Subscribe to changes
      const unsubscribe = context.assets.on('changed', () => {
        const assets = context.assets.getAll()
        setStats(calculateStats(assets))
      })
      
      return unsubscribe
    }, [])
    
    if (!stats) return <div>Loading...</div>
    
    return (
      <div className="asset-stats-panel">
        <h3>Asset Statistics</h3>
        
        <div className="stat-group">
          <label>Total Assets</label>
          <span>{stats.total}</span>
        </div>
        
        <div className="stat-group">
          <label>Total Size</label>
          <span>{formatBytes(stats.totalSize)}</span>
        </div>
        
        <div className="stat-group">
          <label>By Type</label>
          <ul>
            {Object.entries(stats.byType).map(([type, count]) => (
              <li key={type}>{type}: {count}</li>
            ))}
          </ul>
        </div>
        
        <button onClick={() => context.commands.execute('myPlugin.optimize')}>
          Optimize Assets
        </button>
      </div>
    )
  }
}
```

## Publishing Plugins

### Build for Production
```bash
# Build plugin
npm run build

# Run tests
npm test

# Package plugin
npm run package
```

### Plugin Package Structure
```
my-plugin.dreamx
├── manifest.json
├── dist/
│   ├── index.js
│   └── index.js.map
├── assets/
└── README.md
```

### Publishing to Registry
```bash
# Login to Dream Emulator registry
dream-plugin login

# Publish plugin
dream-plugin publish

# Or publish with version bump
dream-plugin publish --bump minor
```

### Plugin Store Listing
```json
{
  "listing": {
    "featured": false,
    "category": "tools",
    "tags": ["animation", "sprites", "workflow"],
    "screenshots": [
      "https://example.com/screenshot1.png",
      "https://example.com/screenshot2.png"
    ],
    "video": "https://youtube.com/watch?v=...",
    "price": 0, // Free, or price in cents
    "requirements": {
      "dreamEmulator": ">=1.2.0",
      "memory": "100MB",
      "permissions": ["filesystem", "network"]
    }
  }
}
```

## Best Practices

### 1. Performance
```typescript
// ❌ Bad: Updating every frame
context.events.on('update', () => {
  const allAssets = context.assets.getAll() // Expensive!
  updateUI(allAssets)
})

// ✅ Good: Debounced updates
const updateDebounced = debounce(() => {
  const allAssets = context.assets.getAll()
  updateUI(allAssets)
}, 100)

context.assets.on('changed', updateDebounced)
```

### 2. Memory Management
```typescript
// ❌ Bad: Keeping references forever
class MyPlugin {
  private cache = new Map<string, LargeObject>()
  
  processAsset(id: string) {
    const obj = new LargeObject(id)
    this.cache.set(id, obj) // Memory leak!
    return obj
  }
}

// ✅ Good: Use weak references or cleanup
class MyPlugin {
  private cache = new WeakMap<Asset, LargeObject>()
  
  processAsset(asset: Asset) {
    let obj = this.cache.get(asset)
    if (!obj) {
      obj = new LargeObject(asset)
      this.cache.set(asset, obj)
    }
    return obj
  }
  
  deactivate() {
    // Cleanup not needed with WeakMap
  }
}
```

### 3. Error Handling
```typescript
// ❌ Bad: Unhandled errors crash the editor
export default class MyPlugin {
  activate(context: PluginContext) {
    context.commands.register({
      id: 'myPlugin.riskyCommand',
      execute: async () => {
        const data = await fetch('/api/data')
        const json = await data.json() // Can throw!
        processData(json)
      }
    })
  }
}

// ✅ Good: Proper error handling
export default class MyPlugin {
  activate(context: PluginContext) {
    context.commands.register({
      id: 'myPlugin.riskyCommand',
      execute: async () => {
        try {
          const data = await fetch('/api/data')
          if (!data.ok) {
            throw new Error(`HTTP ${data.status}`)
          }
          const json = await data.json()
          processData(json)
        } catch (error) {
          context.ui.showMessage(
            `Command failed: ${error.message}`,
            'error'
          )
          console.error('[MyPlugin]', error)
        }
      }
    })
  }
}
```

### 4. Compatibility
```typescript
// ✅ Good: Version checking
export default class MyPlugin {
  activate(context: PluginContext) {
    // Check API availability
    if (!context.nodes?.registerBatch) {
      context.ui.showMessage(
        'This plugin requires Dream Emulator 1.2.0 or later',
        'warning'
      )
      return
    }
    
    // Use new API
    context.nodes.registerBatch(myNodes)
  }
}
```

## Examples

### Example: Sprite Sheet Importer
```typescript
import { Plugin, PluginContext } from '@dreamemulator/plugin-api'

export default class SpriteSheetImporter implements Plugin {
  activate(context: PluginContext) {
    // Register custom importer
    context.assets.registerImporter({
      id: 'spritesheet',
      name: 'Sprite Sheet',
      extensions: ['.png', '.jpg'],
      
      canImport: async (file: File) => {
        // Check if filename contains sprite sheet indicators
        return /sheet|atlas|sprites/i.test(file.name)
      },
      
      import: async (file: File) => {
        // Show configuration dialog
        const config = await this.showConfigDialog(context)
        if (!config) return null
        
        // Process sprite sheet
        const sprites = await this.extractSprites(file, config)
        
        // Import as multiple assets
        return context.assets.importBatch(sprites)
      }
    })
  }
  
  private async showConfigDialog(context: PluginContext) {
    return context.dialogs.show({
      title: 'Import Sprite Sheet',
      fields: [
        {
          id: 'columns',
          label: 'Columns',
          type: 'number',
          default: 4
        },
        {
          id: 'rows',
          label: 'Rows',
          type: 'number',
          default: 4
        },
        {
          id: 'margin',
          label: 'Margin',
          type: 'number',
          default: 0
        }
      ]
    })
  }
  
  private async extractSprites(file: File, config: any) {
    // Load image
    const img = await loadImage(file)
    
    // Calculate sprite dimensions
    const spriteWidth = img.width / config.columns
    const spriteHeight = img.height / config.rows
    
    const sprites = []
    
    // Extract each sprite
    for (let row = 0; row < config.rows; row++) {
      for (let col = 0; col < config.columns; col++) {
        const canvas = document.createElement('canvas')
        canvas.width = spriteWidth - config.margin * 2
        canvas.height = spriteHeight - config.margin * 2
        
        const ctx = canvas.getContext('2d')!
        ctx.drawImage(
          img,
          col * spriteWidth + config.margin,
          row * spriteHeight + config.margin,
          canvas.width,
          canvas.height,
          0, 0,
          canvas.width,
          canvas.height
        )
        
        // Convert to blob
        const blob = await canvasToBlob(canvas)
        const spriteFile = new File(
          [blob],
          `${file.name}_${row}_${col}.png`,
          { type: 'image/png' }
        )
        
        sprites.push(spriteFile)
      }
    }
    
    return sprites
  }
}
```

### Example: Platformer Tools
```typescript
export default class PlatformerTools implements Plugin {
  activate(context: PluginContext) {
    // Add platformer-specific tools
    context.tools.register(new LadderTool())
    context.tools.register(new MovingPlatformTool())
    context.tools.register(new SpringTool())
    
    // Add platformer nodes
    context.nodes.registerBatch([
      GroundCheckNode,
      WallJumpNode,
      DoubleJumpNode,
      DashNode
    ])
    
    // Add menu items
    context.menus.add('create', {
      id: 'platformer',
      label: 'Platformer',
      children: [
        {
          label: 'Player Controller',
          command: 'platformer.createPlayer'
        },
        {
          label: 'Enemy Patroller',
          command: 'platformer.createEnemy'
        }
      ]
    })
    
    // Register commands
    context.commands.register({
      id: 'platformer.createPlayer',
      execute: () => this.createPlayerController(context)
    })
  }
  
  private createPlayerController(context: PluginContext) {
    // Create player with components
    const player = context.scene.createObject('Player')
    
    // Add components
    player.addComponent('Sprite', {
      texture: 'player_idle'
    })
    
    player.addComponent('RigidBody', {
      type: 'dynamic',
      gravityScale: 2
    })
    
    player.addComponent('Collider', {
      shape: 'box',
      size: { width: 32, height: 64 }
    })
    
    // Add visual script for movement
    const script = player.addComponent('VisualScript')
    this.setupPlayerScript(script)
  }
}
```

## API Reference

### Complete API documentation is available at:
- [Plugin API Docs](https://dreamemulator.dev/docs/plugins/api)
- [TypeScript Definitions](https://github.com/dreamemulator/types)
- [Example Plugins](https://github.com/dreamemulator/example-plugins)

### Getting Help
- [Plugin Development Forum](https://forum.dreamemulator.dev/plugins)
- [Discord Server](https://discord.gg/dreamemulator)
- [Video Tutorials](https://youtube.com/dreamemulator)

The plugin system is designed to be powerful yet approachable, enabling creators to extend Dream Emulator in ways we haven't even imagined yet!