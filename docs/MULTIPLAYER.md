# Multiplayer Protocol Documentation

## Table of Contents
- [Overview](#overview)
- [Architecture](#architecture)
- [Connection Types](#connection-types)
- [Protocol Design](#protocol-design)
- [State Synchronization](#state-synchronization)
- [Lag Compensation](#lag-compensation)
- [Security](#security)
- [Server Management](#server-management)
- [Implementation Guide](#implementation-guide)
- [Troubleshooting](#troubleshooting)

## Overview

Dream Emulator's multiplayer system is designed to be simple for creators while providing robust networking for various game types. It supports both peer-to-peer connections for small groups and dedicated servers for larger experiences.

### Design Goals
- **Zero Configuration**: Works out of the box for simple games
- **Scalable**: From 2-player local to 100+ player servers
- **Flexible**: Supports different network topologies
- **Secure**: Built-in protection against common exploits
- **Performant**: Optimized for real-time gameplay

## Architecture

### Network Topologies

#### 1. Peer-to-Peer (P2P)
```
Player A ←→ Player B
    ↑          ↑
    └────┬─────┘
         ↓
     Player C
```
- Best for: 2-8 players
- Advantages: No server needed, low latency
- Disadvantages: NAT traversal, host migration

#### 2. Client-Server
```
     Server
    ↗  ↑  ↖
   ↙   │   ↘
  A    B    C
```
- Best for: 8+ players
- Advantages: Authoritative, scalable
- Disadvantages: Requires hosting

#### 3. Hybrid (Relay)
```
  Relay Server
    ↗  ↑  ↖
   ↙   │   ↘
  A ←→ B ←→ C
```
- Best for: P2P with fallback
- Advantages: Works through NAT
- Disadvantages: Additional latency

### Component Architecture
```typescript
interface NetworkArchitecture {
  // Core components
  transport: Transport           // WebRTC, WebSocket, etc.
  protocol: Protocol            // Message format and routing
  synchronizer: Synchronizer    // State sync logic
  authority: Authority          // Who controls what
  
  // Features
  lobby: LobbyManager          // Matchmaking and rooms
  voice: VoiceChat            // Optional voice communication
  replay: ReplaySystem        // Record and playback
  
  // Security
  encryption: Encryption       // Message encryption
  validation: Validation      // Input validation
  anticheat: AntiCheat       // Cheat detection
}
```

## Connection Types

### WebRTC (P2P)
```typescript
class WebRTCTransport implements Transport {
  private peers = new Map<string, RTCPeerConnection>()
  private dataChannels = new Map<string, RTCDataChannel>()
  
  async connect(peerId: string, offer?: RTCSessionDescription) {
    const peer = new RTCPeerConnection({
      iceServers: [
        { urls: 'stun:stun.l.google.com:19302' },
        { 
          urls: 'turn:turn.dreamemulator.dev:3478',
          username: 'user',
          credential: 'pass'
        }
      ]
    })
    
    // Create data channel
    const channel = peer.createDataChannel('game', {
      ordered: false,      // Speed over reliability
      maxRetransmits: 2   // Limited retries
    })
    
    // Set up handlers
    channel.onopen = () => this.onConnect(peerId)
    channel.onmessage = (e) => this.onMessage(peerId, e.data)
    channel.onclose = () => this.onDisconnect(peerId)
    
    this.peers.set(peerId, peer)
    this.dataChannels.set(peerId, channel)
    
    // Handle offer/answer
    if (offer) {
      await peer.setRemoteDescription(offer)
      const answer = await peer.createAnswer()
      await peer.setLocalDescription(answer)
      return answer
    } else {
      const offer = await peer.createOffer()
      await peer.setLocalDescription(offer)
      return offer
    }
  }
  
  send(peerId: string, data: ArrayBuffer) {
    const channel = this.dataChannels.get(peerId)
    if (channel?.readyState === 'open') {
      channel.send(data)
    }
  }
  
  broadcast(data: ArrayBuffer) {
    for (const channel of this.dataChannels.values()) {
      if (channel.readyState === 'open') {
        channel.send(data)
      }
    }
  }
}
```

### WebSocket (Client-Server)
```typescript
class WebSocketTransport implements Transport {
  private socket: WebSocket
  private messageQueue: ArrayBuffer[] = []
  
  async connect(serverUrl: string) {
    this.socket = new WebSocket(serverUrl)
    this.socket.binaryType = 'arraybuffer'
    
    return new Promise((resolve, reject) => {
      this.socket.onopen = () => {
        // Send queued messages
        this.messageQueue.forEach(msg => this.socket.send(msg))
        this.messageQueue = []
        resolve()
      }
      
      this.socket.onmessage = (e) => {
        this.onMessage('server', e.data)
      }
      
      this.socket.onerror = reject
      this.socket.onclose = () => this.onDisconnect('server')
    })
  }
  
  send(target: string, data: ArrayBuffer) {
    if (this.socket.readyState === WebSocket.OPEN) {
      this.socket.send(data)
    } else {
      this.messageQueue.push(data)
    }
  }
}
```

## Protocol Design

### Message Format
```typescript
// Binary protocol for efficiency
enum MessageType {
  // Connection
  Join = 0x01,
  Leave = 0x02,
  Ping = 0x03,
  
  // Game state
  Snapshot = 0x10,
  Delta = 0x11,
  Event = 0x12,
  
  // Player input
  Input = 0x20,
  Command = 0x21,
  
  // Voice/Chat
  Voice = 0x30,
  Chat = 0x31,
  
  // System
  Error = 0xF0,
  Kick = 0xF1
}

interface Message {
  type: MessageType
  timestamp: number
  sender: string
  data: ArrayBuffer
}

class Protocol {
  encode(message: Message): ArrayBuffer {
    const encoder = new Encoder()
    
    encoder.writeUint8(message.type)
    encoder.writeUint32(message.timestamp)
    encoder.writeString(message.sender)
    encoder.writeBytes(message.data)
    
    return encoder.buffer
  }
  
  decode(buffer: ArrayBuffer): Message {
    const decoder = new Decoder(buffer)
    
    return {
      type: decoder.readUint8(),
      timestamp: decoder.readUint32(),
      sender: decoder.readString(),
      data: decoder.readBytes()
    }
  }
}
```

### Efficient Encoding
```typescript
class Encoder {
  private buffer: ArrayBuffer
  private view: DataView
  private offset = 0
  
  constructor(size = 1024) {
    this.buffer = new ArrayBuffer(size)
    this.view = new DataView(this.buffer)
  }
  
  writeUint8(value: number) {
    this.view.setUint8(this.offset, value)
    this.offset += 1
  }
  
  writeUint16(value: number) {
    this.view.setUint16(this.offset, value, true) // Little endian
    this.offset += 2
  }
  
  writeFloat32(value: number) {
    this.view.setFloat32(this.offset, value, true)
    this.offset += 4
  }
  
  writeVector2(x: number, y: number) {
    // Compress to 16-bit integers for position
    this.writeInt16(Math.round(x * 100)) // 0.01 precision
    this.writeInt16(Math.round(y * 100))
  }
  
  writeString(value: string) {
    const encoded = new TextEncoder().encode(value)
    this.writeUint16(encoded.length)
    new Uint8Array(this.buffer, this.offset).set(encoded)
    this.offset += encoded.length
  }
}
```

## State Synchronization

### Snapshot Interpolation
```typescript
class SnapshotInterpolation {
  private snapshots: Snapshot[] = []
  private interpolationDelay = 100 // ms
  
  addSnapshot(snapshot: Snapshot) {
    // Keep sorted by timestamp
    this.snapshots.push(snapshot)
    this.snapshots.sort((a, b) => a.timestamp - b.timestamp)
    
    // Keep only recent snapshots
    const cutoff = Date.now() - 1000
    this.snapshots = this.snapshots.filter(s => s.timestamp > cutoff)
  }
  
  getInterpolatedState(currentTime: number): GameState {
    const renderTime = currentTime - this.interpolationDelay
    
    // Find snapshots to interpolate between
    let before: Snapshot | null = null
    let after: Snapshot | null = null
    
    for (let i = 0; i < this.snapshots.length - 1; i++) {
      if (this.snapshots[i].timestamp <= renderTime &&
          this.snapshots[i + 1].timestamp >= renderTime) {
        before = this.snapshots[i]
        after = this.snapshots[i + 1]
        break
      }
    }
    
    if (!before || !after) {
      // Use latest snapshot if can't interpolate
      return this.snapshots[this.snapshots.length - 1]?.state || {}
    }
    
    // Interpolate between snapshots
    const progress = (renderTime - before.timestamp) / 
                    (after.timestamp - before.timestamp)
    
    return this.interpolate(before.state, after.state, progress)
  }
  
  private interpolate(a: GameState, b: GameState, t: number): GameState {
    const result: GameState = {}
    
    // Interpolate each entity
    for (const id in a.entities) {
      const entityA = a.entities[id]
      const entityB = b.entities[id]
      
      if (!entityB) continue // Entity removed
      
      result.entities[id] = {
        position: {
          x: lerp(entityA.position.x, entityB.position.x, t),
          y: lerp(entityA.position.y, entityB.position.y, t)
        },
        rotation: lerpAngle(entityA.rotation, entityB.rotation, t),
        // Non-interpolated values use latest
        health: entityB.health,
        state: entityB.state
      }
    }
    
    return result
  }
}
```

### Delta Compression
```typescript
class DeltaCompression {
  private baselineState: GameState = {}
  private stateHistory = new Map<number, GameState>()
  
  createDelta(newState: GameState, baselineId: number): Delta {
    const baseline = this.stateHistory.get(baselineId) || this.baselineState
    const delta: Delta = {
      baselineId,
      changes: {}
    }
    
    // Find changed entities
    for (const id in newState.entities) {
      const newEntity = newState.entities[id]
      const oldEntity = baseline.entities[id]
      
      if (!oldEntity) {
        // New entity
        delta.changes[id] = { type: 'create', data: newEntity }
      } else if (this.hasChanged(oldEntity, newEntity)) {
        // Changed entity
        delta.changes[id] = { 
          type: 'update', 
          data: this.getDiff(oldEntity, newEntity) 
        }
      }
    }
    
    // Find removed entities
    for (const id in baseline.entities) {
      if (!newState.entities[id]) {
        delta.changes[id] = { type: 'remove' }
      }
    }
    
    return delta
  }
  
  applyDelta(baseline: GameState, delta: Delta): GameState {
    const result = JSON.parse(JSON.stringify(baseline)) // Deep clone
    
    for (const id in delta.changes) {
      const change = delta.changes[id]
      
      switch (change.type) {
        case 'create':
          result.entities[id] = change.data
          break
        case 'update':
          Object.assign(result.entities[id], change.data)
          break
        case 'remove':
          delete result.entities[id]
          break
      }
    }
    
    return result
  }
}
```

## Lag Compensation

### Client-Side Prediction
```typescript
class ClientPrediction {
  private predictedState: GameState
  private inputBuffer: Input[] = []
  private lastAcknowledgedInput = -1
  
  processInput(input: Input) {
    // Apply input locally immediately
    this.predictedState = this.applyInput(this.predictedState, input)
    
    // Buffer input to send to server
    input.sequence = this.getNextSequence()
    this.inputBuffer.push(input)
    
    // Send to server
    this.network.send({
      type: MessageType.Input,
      data: input
    })
  }
  
  receiveServerUpdate(serverState: GameState, acknowledgedInput: number) {
    this.lastAcknowledgedInput = acknowledgedInput
    
    // Start from server state
    this.predictedState = serverState
    
    // Re-apply unacknowledged inputs
    for (const input of this.inputBuffer) {
      if (input.sequence > acknowledgedInput) {
        this.predictedState = this.applyInput(this.predictedState, input)
      }
    }
    
    // Clean old inputs
    this.inputBuffer = this.inputBuffer.filter(
      i => i.sequence > acknowledgedInput
    )
  }
}
```

### Server Reconciliation
```typescript
class ServerReconciliation {
  private clientStates = new Map<string, ClientState>()
  
  processClientInput(clientId: string, input: Input) {
    const client = this.clientStates.get(clientId)
    if (!client) return
    
    // Validate input
    if (!this.validateInput(input, client)) {
      // Reject invalid input
      this.sendCorrection(clientId, client.authoritative)
      return
    }
    
    // Apply input
    client.authoritative = this.applyInput(client.authoritative, input)
    client.lastProcessedInput = input.sequence
    
    // Send acknowledgment
    this.sendAck(clientId, input.sequence, client.authoritative)
  }
  
  private validateInput(input: Input, client: ClientState): boolean {
    // Check sequence number
    if (input.sequence <= client.lastProcessedInput) {
      return false // Old input
    }
    
    // Check timestamp (prevent speed hacks)
    const expectedTime = client.lastInputTime + (1000 / 60) // 60 FPS
    if (input.timestamp < expectedTime - 50) { // 50ms tolerance
      return false // Too fast
    }
    
    // Validate movement speed
    if (input.type === 'move') {
      const distance = Math.sqrt(
        input.dx * input.dx + input.dy * input.dy
      )
      const maxSpeed = client.character.maxSpeed * (input.deltaTime / 1000)
      if (distance > maxSpeed * 1.1) { // 10% tolerance
        return false // Too fast
      }
    }
    
    return true
  }
}