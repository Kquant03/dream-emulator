# 🎮 Dream Emulator

> **Create games as easily as you imagine them.** A revolutionary visual game creation tool that combines the accessibility of Mario Maker with the power to build any genre - from top-down RPGs to 3D adventures.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Tauri](https://img.shields.io/badge/Tauri-2.0-24C8DB.svg)
![React](https://img.shields.io/badge/React-18-61DAFB.svg)
![TypeScript](https://img.shields.io/badge/TypeScript-5.0-3178C6.svg)
![Status](https://img.shields.io/badge/status-alpha-orange.svg)

## 🌟 Vision

Dream Emulator democratizes game development by providing a visual, intuitive interface where creativity matters more than coding knowledge. Inspired by tools like Mario Maker, but without genre limitations - if you can dream it, you can build it.

## ✨ Key Features

### 🎨 Visual Game Creation
- **Drag-and-Drop World Building** - Place sprites, tiles, and objects with intuitive controls
- **Real-Time Preview** - See your changes instantly without compilation
- **Smart Grid System** - Automatic snapping and alignment for pixel-perfect placement
- **Layer Management** - Organize your game world with multiple visual layers

### 🔧 Node-Based Visual Programming
- **No Code Required** - Connect behaviors using visual nodes
- **Event-Driven Logic** - "When player touches enemy" → "Reduce health"
- **Pre-Built Components** - Library of common game mechanics
- **Custom Scripts** - Advanced users can write custom nodes

### 🎮 Multi-Genre Support
- **Top-Down** - Create games like Zelda, Stardew Valley, or Pokemon
- **Side-Scroller** - Build platformers and metroidvanias (coming soon)
- **3D First-Person** - Craft immersive worlds like Daggerfall (planned)

### 🌐 Built-in Multiplayer
- **Integrated Server Management** - Host games without technical knowledge
- **Peer-to-Peer Support** - Small group play without dedicated servers
- **Cloud Saves** - Automatic backup and sync across devices

### 📚 Project Management
- **Asset Library** - Organize sprites, sounds, and scripts
- **Version Control** - Built-in save states and project history
- **Export Options** - Build for Windows, Mac, Linux, and Web

## 🏗️ Architecture Overview

```
Dream Emulator
├── Frontend (React + TypeScript)
│   ├── Main Menu           - Animated entry point with engine selection
│   ├── Game Editors        - Engine-specific creation interfaces
│   ├── Visual Scripting    - React Flow-based node editor
│   └── Asset Management    - Drag-drop asset organization
├── State Management (Zustand)
│   ├── Projects            - Game project CRUD operations
│   ├── Scenes              - Scene and object hierarchies
│   ├── Assets              - File management and tagging
│   └── Editor State        - Tool selection, camera, undo/redo
├── Game Engine (PIXI.js)
│   ├── Renderer            - WebGL-accelerated 2D graphics
│   ├── ECS                 - Entity-Component-System architecture
│   ├── Physics             - Collision detection and response
│   └── Input               - Keyboard, mouse, gamepad support
└── Desktop App (Tauri)
    ├── File System         - Native file access for assets
    ├── Window Management   - Multi-window support
    ├── Native Menus        - OS-integrated menus
    └── Auto-Updates        - Seamless version updates
```

## 🚀 Getting Started

### Prerequisites

- **Node.js** 18+ ([Download](https://nodejs.org/))
- **Rust** 1.70+ ([Download](https://rustup.rs/))
- **System Dependencies** (Linux only):
  ```bash
  # Ubuntu/Debian
  sudo apt update
  sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
    libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev \
    libglib2.0-dev libcairo2-dev libpango1.0-dev
  ```

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/dream-emulator.git
cd dream-emulator

# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

### Common Issues & Solutions

<details>
<summary>Snap Library Conflicts (Linux)</summary>

If you encounter `symbol lookup error` with Snap packages:

```bash
# Use the provided wrapper script
./run-tauri-dev.sh

# Or manually unset problematic variables
unset LD_LIBRARY_PATH && npm run tauri dev
```
</details>

<details>
<summary>Missing glib-2.0 (Linux)</summary>

```bash
sudo apt install libglib2.0-dev pkg-config
```
</details>

## 📁 Project Structure

```
dream-emulator/
├── src/                    # React application source
│   ├── App.tsx            # Main app component with routing
│   ├── main.tsx           # Application entry point
│   ├── components/        # UI components
│   │   ├── MainMenu/      # Animated main menu
│   │   │   ├── MainMenu.tsx
│   │   │   └── MainMenu.module.css
│   │   └── GameCreator/   # Game creation interfaces
│   │       ├── TopDownGameCreator.tsx
│   │       └── TopDownGameCreator.module.css
│   ├── store/             # Zustand state management
│   │   └── index.ts       # Global state store
│   ├── engine/            # Game engine integration
│   ├── visual-scripting/  # Node-based programming
│   └── styles/            # Global styles
│       └── global.css
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── main.rs        # Tauri entry point
│   │   └── commands/      # IPC commands
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
├── public/                # Static assets
├── package.json           # Node dependencies
└── vite.config.ts         # Build configuration
```

## 🔧 Development Guide

### State Management

The application uses Zustand for state management with the following stores:

```typescript
// Main store structure
interface DreamEmulatorStore {
  // Projects
  projects: Project[]
  currentProject: Project | null
  createProject: (name: string, engineType: EngineType) => Promise<Project>
  
  // Scenes & Objects
  scenes: Map<string, GameScene>
  currentScene: GameScene | null
  createGameObject: (name: string) => GameObject
  
  // Assets
  assets: Map<string, Asset>
  uploadAsset: (file: File, type: AssetType) => Promise<Asset>
  
  // Visual Scripts
  scripts: Map<string, VisualScript>
  createScript: (name: string) => VisualScript
  
  // Editor State
  editor: EditorState
  updateEditor: (updates: Partial<EditorState>) => void
}
```

### Adding New Features

1. **New Game Engine Type**
   ```typescript
   // 1. Add to engine types in store/index.ts
   engineType: 'topdown' | 'sidescroller' | '3d' | 'your-new-type'
   
   // 2. Create editor component in components/GameCreator/
   // 3. Add case in App.tsx routing
   // 4. Update MainMenu engine selection
   ```

2. **New Visual Script Node**
   ```typescript
   // 1. Define node type in visual-scripting/nodes/
   // 2. Register in node registry
   // 3. Add to node palette UI
   ```

3. **New Asset Type**
   ```typescript
   // 1. Add to Asset type union
   // 2. Create import handler
   // 3. Add preview component
   // 4. Update asset panel UI
   ```

### Component Architecture

All major components follow this pattern:

```typescript
interface ComponentProps {
  // Props for parent communication
  onExit?: () => void
  onSave?: (data: any) => void
}

const Component: React.FC<ComponentProps> = ({ onExit, onSave }) => {
  // Local state
  const [localState, setLocalState] = useState()
  
  // Global state
  const { globalState, updateGlobal } = useDreamEmulator()
  
  // Effects for initialization
  useEffect(() => {
    // Setup code
  }, [])
  
  return (
    <div className={styles.container}>
      {/* Component UI */}
    </div>
  )
}
```

## 🎯 Roadmap

### Phase 1: Foundation (Current)
- [x] Project setup with Tauri + React
- [x] Main menu with engine selection
- [x] Basic top-down editor interface
- [x] State management system
- [ ] Asset upload and management
- [ ] Basic drag-and-drop functionality
- [ ] Simple play-testing mode

### Phase 2: Visual Programming
- [ ] React Flow integration
- [ ] Basic node types (events, actions, conditions)
- [ ] Node connection validation
- [ ] Code generation from nodes
- [ ] Node library/palette

### Phase 3: Enhanced Editors
- [ ] Side-scroller editor
- [ ] Tilemap support
- [ ] Sprite animation editor
- [ ] Sound integration
- [ ] Particle effects

### Phase 4: Multiplayer
- [ ] WebRTC peer-to-peer
- [ ] Dedicated server support
- [ ] Server browser UI
- [ ] Player synchronization
- [ ] Chat system

### Phase 5: Polish & Export
- [ ] Game export (standalone executables)
- [ ] Web export
- [ ] Asset optimization
- [ ] Performance profiling
- [ ] Tutorial system

### Future Vision
- [ ] 3D engine support
- [ ] VR/AR capabilities
- [ ] Mobile app development
- [ ] Community asset marketplace
- [ ] Cloud collaboration

## 🛠️ Building for Production

```bash
# Build for current platform
npm run tauri build

# Outputs will be in:
# - src-tauri/target/release/ (executable)
# - src-tauri/target/release/bundle/ (installers)
```

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Workflow

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Style

- **TypeScript**: Use strict mode, prefer interfaces over types
- **React**: Functional components with hooks
- **CSS**: CSS Modules for component styles
- **Rust**: Follow standard Rust conventions

## 📊 Performance Targets

- **Startup Time**: < 2 seconds
- **Frame Rate**: 60 FPS during editing
- **Memory Usage**: < 500MB for typical projects
- **Asset Loading**: < 100ms per asset
- **Save/Load**: < 1 second for average project

## 🔒 Security

- **Sandboxed Execution**: User scripts run in isolated contexts
- **Asset Validation**: All uploads are verified and sanitized
- **Network Isolation**: Multiplayer uses encrypted connections
- **Auto-Updates**: Signed releases with integrity verification

## 📚 Documentation

- [Architecture Deep Dive](docs/ARCHITECTURE.md)
- [Visual Scripting Guide](docs/VISUAL_SCRIPTING.md)
- [Asset Pipeline](docs/ASSETS.md)
- [Multiplayer Protocol](docs/MULTIPLAYER.md)
- [Plugin Development](docs/PLUGINS.md)

## 🙏 Acknowledgments

Inspired by:
- **Mario Maker** - For showing that game creation can be fun
- **GameMaker Studio** - For powerful 2D game development
- **Unity** - For comprehensive game engine architecture
- **Roblox** - For accessible multiplayer experiences

Built with:
- [Tauri](https://tauri.app/) - Desktop application framework
- [React](https://react.dev/) - UI library
- [PIXI.js](https://pixijs.com/) - 2D WebGL renderer
- [React Flow](https://reactflow.dev/) - Node-based UI
- [Zustand](https://zustand-demo.pmnd.rs/) - State management

## 📄 License

This project is licensed under the MIT License - see [LICENSE](LICENSE) for details.

---

<div align="center">

**[Website](https://dreamemulator.dev)** • **[Discord](https://discord.gg/dreamemulator)** • **[Twitter](https://twitter.com/dreamemulator)**

Made with ❤️ by developers who believe everyone should be able to create games

</div>
