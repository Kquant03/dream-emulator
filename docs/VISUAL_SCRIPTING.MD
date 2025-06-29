# Visual Scripting Guide

## Table of Contents
- [Introduction](#introduction)
- [Core Concepts](#core-concepts)
- [Node Types](#node-types)
- [Creating Scripts](#creating-scripts)
- [Built-in Nodes](#built-in-nodes)
- [Custom Nodes](#custom-nodes)
- [Best Practices](#best-practices)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)

## Introduction

Dream Emulator's visual scripting system allows you to create game logic without writing code. By connecting nodes in a graph, you define how your game responds to events and player input.

### Key Benefits
- **No Syntax Errors**: Visual connections prevent syntax mistakes
- **Live Preview**: See your logic flow in real-time
- **Debugging**: Visual execution flow makes debugging intuitive
- **Reusability**: Save node graphs as templates

## Core Concepts

### Nodes
Nodes are the building blocks of visual scripts. Each node represents an operation, event, or value.

```
┌─────────────────┐
│   Node Name     │
├─────────────────┤
│ ● Input Port    │
│                 │
│   Properties    │
│                 │
│ Output Port ●   │
└─────────────────┘
```

### Connections
Connections (edges) transfer data or execution flow between nodes.

- **Data Connections**: Transfer values (numbers, strings, objects)
- **Execution Connections**: Control flow order (white connections)

### Ports
- **Input Ports** (left side): Receive data or execution
- **Output Ports** (right side): Send data or execution

### Port Types
- 🟢 **Number**: Float or integer values
- 🔵 **String**: Text values
- 🟣 **Boolean**: True/false values
- 🟡 **Object**: Game objects or complex data
- ⚪ **Execution**: Flow control
- 🔴 **Any**: Accepts any data type

## Node Types

### 1. Event Nodes
Trigger script execution based on game events.

```
┌─────────────────┐
│ 🎮 Game Start   │
├─────────────────┤
│                 │
│          Next ● │──→
└─────────────────┘
```

**Common Events:**
- Game Start
- Update (every frame)
- Fixed Update (physics)
- Player Input
- Collision Enter/Exit
- Timer Complete
- Custom Events

### 2. Action Nodes
Perform operations in your game.

```
┌─────────────────┐
│ 📦 Create Object│
├─────────────────┤
│ ● Execute       │
│ ● Type: "Enemy" │
│ ● Position      │
│                 │
│     Complete ●  │──→
│     Object ●    │──→
└─────────────────┘
```

**Common Actions:**
- Create/Destroy Object
- Move/Rotate/Scale
- Play Sound/Animation
- Change Scene
- Set Variable
- Send Message

### 3. Logic Nodes
Control flow and make decisions.

```
┌─────────────────┐
│ ❓ If/Else      │
├─────────────────┤
│ ● Execute       │
│ ● Condition     │
│                 │
│      True ●     │──→
│      False ●    │──→
└─────────────────┘
```

**Logic Types:**
- If/Else
- Switch
- For Loop
- While Loop
- Sequence
- Parallel

### 4. Value Nodes
Provide or manipulate data.

```
┌─────────────────┐
│ 🔢 Number       │
├─────────────────┤
│   Value: 42     │
│                 │
│      Output ●   │──→
└─────────────────┘
```

**Value Types:**
- Constants (Number, String, Boolean)
- Variables (Get/Set)
- Math Operations
- String Operations
- Type Conversions

### 5. Object Nodes
Work with game objects and components.

```
┌─────────────────┐
│ 🎯 Get Component│
├─────────────────┤
│ ● Object        │
│   Type: Health  │
│                 │
│   Component ●   │──→
└─────────────────┘
```

## Creating Scripts

### Step 1: Open Visual Script Editor
1. Select an object in the scene
2. Click "Add Component" → "Visual Script"
3. Click "Edit Script" to open the editor

### Step 2: Add Nodes
- **Right-click** in empty space to open node menu
- **Search** for nodes by name
- **Drag** from palette to add nodes

### Step 3: Connect Nodes
1. **Click and drag** from output port to input port
2. **Compatible ports** will highlight in green
3. **Release** to create connection

### Step 4: Configure Properties
1. **Select** a node to see properties
2. **Edit** values in the inspector
3. **Preview** changes in real-time

## Built-in Nodes

### Events

#### On Start
```typescript
{
  type: "event.start",
  outputs: {
    exec: "execution"
  }
}
```

#### On Update
```typescript
{
  type: "event.update",
  outputs: {
    exec: "execution",
    deltaTime: "number"
  }
}
```

#### On Key Press
```typescript
{
  type: "event.keyPress",
  properties: {
    key: "Space"
  },
  outputs: {
    exec: "execution",
    key: "string"
  }
}
```

### Actions

#### Move Object
```typescript
{
  type: "action.move",
  inputs: {
    exec: "execution",
    object: "gameObject",
    direction: "vector2",
    speed: "number"
  },
  outputs: {
    exec: "execution"
  }
}
```

#### Spawn Object
```typescript
{
  type: "action.spawn",
  inputs: {
    exec: "execution",
    prefab: "string",
    position: "vector2",
    rotation: "number"
  },
  outputs: {
    exec: "execution",
    spawned: "gameObject"
  }
}
```

### Logic

#### Branch (If/Else)
```typescript
{
  type: "logic.branch",
  inputs: {
    exec: "execution",
    condition: "boolean"
  },
  outputs: {
    true: "execution",
    false: "execution"
  }
}
```

#### For Each
```typescript
{
  type: "logic.forEach",
  inputs: {
    exec: "execution",
    array: "array"
  },
  outputs: {
    loop: "execution",
    complete: "execution",
    item: "any",
    index: "number"
  }
}
```

### Math

#### Add
```typescript
{
  type: "math.add",
  inputs: {
    a: "number",
    b: "number"
  },
  outputs: {
    result: "number"
  }
}
```

#### Random Range
```typescript
{
  type: "math.randomRange",
  inputs: {
    min: "number",
    max: "number"
  },
  outputs: {
    value: "number"
  }
}
```

## Custom Nodes

### Creating a Custom Node

```typescript
// nodes/custom/MyCustomNode.ts
import { NodeDefinition } from '@/visual-scripting/types'

export const MyCustomNode: NodeDefinition = {
  type: 'custom.myNode',
  category: 'Custom',
  name: 'My Custom Node',
  description: 'Does something special',
  
  inputs: {
    exec: { type: 'execution', name: 'Execute' },
    value: { type: 'number', name: 'Input Value' }
  },
  
  outputs: {
    exec: { type: 'execution', name: 'Next' },
    result: { type: 'number', name: 'Result' }
  },
  
  properties: {
    multiplier: {
      type: 'number',
      default: 2,
      min: 0,
      max: 10
    }
  },
  
  execute(context, inputs, properties) {
    const result = inputs.value * properties.multiplier
    
    return {
      result
    }
  }
}
```

### Registering Custom Nodes

```typescript
// visual-scripting/registry.ts
import { MyCustomNode } from './nodes/custom/MyCustomNode'

export function registerCustomNodes(registry: NodeRegistry) {
  registry.register(MyCustomNode)
}
```

### Node UI Customization

```typescript
export const MyCustomNode: NodeDefinition = {
  // ... other properties
  
  renderNode(props) {
    return (
      <div className="custom-node">
        <div className="node-header">
          <Icon name="star" />
          <span>{props.data.name}</span>
        </div>
        <div className="node-body">
          {/* Custom UI */}
        </div>
      </div>
    )
  },
  
  renderPreview(props) {
    return (
      <div className="node-preview">
        Value: {props.properties.multiplier}x
      </div>
    )
  }
}
```

## Best Practices

### 1. Keep Graphs Simple
- Break complex logic into sub-graphs
- Use comments to document sections
- Group related nodes together

### 2. Name Everything
- Give nodes descriptive names
- Label important connections
- Use clear variable names

### 3. Optimize Performance
- Avoid Update loops for heavy operations
- Cache frequently used values
- Use event-driven logic when possible

### 4. Error Handling
- Add null checks for objects
- Provide default values
- Use Try/Catch nodes for risky operations

### 5. Reusability
- Create node groups for common patterns
- Save templates for reuse
- Build a library of utility graphs

## Examples

### Example 1: Player Movement
```
[On Update] → [Get Input Axis] → [Multiply (speed)] → [Move Object]
                                          ↑
                                    [Get Variable: moveSpeed]
```

### Example 2: Enemy Spawner
```
[On Start] → [Set Variable: spawnTimer = 0]
     ↓
[On Update] → [Add: deltaTime to spawnTimer] → [If: spawnTimer > 2]
                                                      ↓ True
                                        [Spawn Enemy] → [Reset Timer]
```

### Example 3: Health System
```
[On Collision: Bullet] → [Get Component: Health] → [Subtract: 10] → [Set Health]
                                                                          ↓
                                                            [If: Health <= 0]
                                                                   ↓ True
                                                            [Destroy Object]
```

### Example 4: Collectible Item
```
[On Trigger Enter: Player] → [Get Component: Inventory] → [Add Item: Coin]
                                    ↓
                          [Play Sound: Collect] → [Destroy Self]
```

## Troubleshooting

### Common Issues

#### 1. Nodes Not Executing
- Check execution connections (white lines)
- Ensure event node is triggering
- Verify object is active in scene

#### 2. Type Mismatch
- Port colors must match
- Use conversion nodes if needed
- Check data flow direction

#### 3. Performance Problems
- Too many nodes in Update loop
- Use events instead of polling
- Cache expensive calculations

#### 4. Null Reference Errors
- Add "Is Valid" checks before using objects
- Ensure objects exist before accessing
- Handle destroyed objects gracefully

### Debug Mode

Enable debug mode to see:
- Execution flow visualization
- Value inspection on connections
- Performance metrics per node
- Error highlighting

```typescript
// Enable in editor settings
{
  visualScripting: {
    debugMode: true,
    showExecutionFlow: true,
    showDataValues: true,
    highlightErrors: true
  }
}
```

### Performance Tips

1. **Batch Operations**: Combine multiple operations into custom nodes
2. **Event-Driven**: Use events instead of constant checking
3. **Object Pooling**: Reuse objects instead of creating/destroying
4. **LOD Scripts**: Disable complex scripts for distant objects

## Advanced Topics

### Sub-Graphs
Create reusable logic as sub-graphs:
1. Select nodes to group
2. Right-click → "Create Sub-Graph"
3. Name and save the sub-graph
4. Use like any other node

### Script Communication
Send messages between scripts:
```
[Send Message: "PlayerDied"] → [Broadcast to: All]

// In another script:
[On Message: "PlayerDied"] → [Reset Level]
```

### Custom Events
Define your own events:
```typescript
// Register custom event
EventSystem.register('inventory.full', ['item'])

// Trigger from code or nodes
EventSystem.trigger('inventory.full', { item: 'Sword' })
```

This visual scripting system empowers creators to build complex game logic without coding, while maintaining the flexibility for advanced users to extend with custom nodes.