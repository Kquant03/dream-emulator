#!/bin/bash

echo "🚀 Setting up Git repository for Dream Emulator"

# Save the .gitignore file
echo "Creating comprehensive .gitignore..."
cat > .gitignore << 'EOF'
# Dependencies
node_modules/
.pnp
.pnp.js

# Testing
coverage/
*.lcov
.nyc_output/

# Production builds
dist/
dist-ssr/
build/
*.local

# Tauri
src-tauri/target/
src-tauri/WixTools/
src-tauri/.cargo/config.toml

# Editor directories and files
.vscode/*
!.vscode/extensions.json
!.vscode/settings.json
!.vscode/tasks.json
!.vscode/launch.json
.idea/
*.swp
*.swo
*~
.project
.classpath
.c9/
*.launch
.settings/
*.sublime-workspace
*.sublime-project

# OS generated files
.DS_Store
.DS_Store?
._*
.Spotlight-V100
.Trashes
ehthumbs.db
Thumbs.db
desktop.ini

# Logs
logs/
*.log
npm-debug.log*
yarn-debug.log*
yarn-error.log*
pnpm-debug.log*
lerna-debug.log*

# Environment variables
.env
.env.local
.env.development.local
.env.test.local
.env.production.local
.env*.local

# Package manager lock files (optional - you may want to keep these)
# Uncomment if you want to track lock files
# package-lock.json
# yarn.lock
# pnpm-lock.yaml

# Temporary files
*.tmp
*.temp
*.cache
.cache/
tmp/
temp/

# TypeScript
*.tsbuildinfo

# Optional npm cache directory
.npm

# Optional eslint cache
.eslintcache

# Optional stylelint cache
.stylelintcache

# Rust
debug/
**/*.rs.bk
*.pdb

# Project specific
run-tauri-dev.sh
run-dev.sh
projset.sh

# Game-specific
/assets/user/
/saves/
*.save
*.bak
EOF

# Initialize Git repository if not already initialized
if [ ! -d .git ]; then
    echo "Initializing Git repository..."
    git init
else
    echo "Git repository already initialized"
fi

# Create a README if it doesn't exist
if [ ! -f README.md ]; then
    echo "Creating README.md..."
    cat > README.md << 'EOF'
# Dream Emulator

A visual game creation tool that makes building games as easy as using Mario Maker. Create top-down games, side-scrollers, and 3D experiences with an intuitive drag-and-drop interface.

## Features

- 🎮 **Visual Game Creation** - Drag and drop assets to build your game world
- 🔧 **Node-Based Programming** - Connect behaviors without writing code  
- 🎨 **Multiple Game Types** - Support for top-down, side-scroller, and 3D games
- 🌐 **Multiplayer Support** - Built-in server management for online games
- 💾 **Project Management** - Save, load, and organize your game projects

## Tech Stack

- **Frontend**: React + TypeScript + Vite
- **Desktop App**: Tauri (Rust)
- **State Management**: Zustand
- **Game Engine**: PIXI.js
- **Visual Programming**: React Flow
- **UI Components**: Framer Motion, Lucide Icons

## Development

### Prerequisites

- Node.js 18+
- Rust 1.70+
- System dependencies (see Tauri prerequisites)

### Setup

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

### Building

```bash
# Build for production
npm run tauri build
```

## License

MIT

---

Built with ❤️ using Tauri and React
EOF
fi

# Add all files
echo "Adding files to Git..."
git add .

# Show status
echo ""
echo "📊 Git Status:"
git status --short

# Create initial commit
echo ""
read -p "Ready to create initial commit? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    git commit -m "Initial commit: Dream Emulator game creation tool

- Tauri + React + TypeScript setup
- Main menu with animated UI
- Top-down game creator interface  
- State management with Zustand
- Drag-and-drop asset system
- Visual scripting foundation"
fi

# GitHub setup
echo ""
echo "📦 GitHub Setup"
echo "==============="
echo ""
echo "1. Create a new repository on GitHub:"
echo "   https://github.com/new"
echo "   - Name: dream-emulator"
echo "   - Description: Visual game creation tool built with Tauri"
echo "   - Keep it Public or Private as you prefer"
echo "   - DON'T initialize with README, .gitignore, or license"
echo ""
echo "2. After creating, run these commands:"
echo ""
echo "git remote add origin https://github.com/YOUR_USERNAME/dream-emulator.git"
echo "git branch -M main"
echo "git push -u origin main"
echo ""
echo "Replace YOUR_USERNAME with your GitHub username!"
echo ""
echo "Alternative (if using SSH):"
echo "git remote add origin git@github.com:YOUR_USERNAME/dream-emulator.git"

# Optional: Check if gh CLI is installed
if command -v gh &> /dev/null; then
    echo ""
    echo "🎉 GitHub CLI detected! You can create the repo automatically:"
    echo "gh repo create dream-emulator --public --source=. --remote=origin --push"
fi